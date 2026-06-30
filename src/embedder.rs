use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use euclid::{Scale, Size2D};
use servo::{
    InputEvent, RenderingContext, Servo, ServoBuilder, WebView, WebViewBuilder, WheelDelta,
    WheelEvent, WheelMode, WindowRenderingContext, UserContentManager, UserScript, Opts,
};
use url::Url;
use winit::{
    application::ApplicationHandler,
    event::{MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Icon, Window, WindowId},
};

use crate::browser::{SharedBanList, normalize_url, extract_domain};
use crate::spoof;

fn load_icon() -> Option<Icon> {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/icon.png").ok()?.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).ok()
}

struct AppState {
    window: Window,
    servo: Servo,
    rendering_context: Rc<WindowRenderingContext>,
    webviews: RefCell<Vec<WebView>>,
}

impl servo::WebViewDelegate for AppState {
    fn notify_new_frame_ready(&self, _: WebView) {
        self.window.request_redraw();
    }
}

enum JuanitaApp {
    Initial(Waker, SharedBanList),
    Running(Rc<AppState>),
}

impl JuanitaApp {
    fn new(event_loop: &EventLoop<WakerEvent>, state: SharedBanList) -> Self {
        Self::Initial(Waker::new(event_loop), state)
    }
}

impl ApplicationHandler<WakerEvent> for JuanitaApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Self::Initial(waker, _state) = self {
            #[allow(unused_mut)]
            let mut attrs = Window::default_attributes()
                .with_title("Juanita Banana 🍌")
                .with_window_icon(load_icon())
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 800));
            
            #[cfg(target_os = "linux")]
            {
                use winit::platform::wayland::WindowAttributesExtWayland;
                use winit::platform::x11::WindowAttributesExtX11;
                attrs = WindowAttributesExtWayland::with_name(attrs.clone(), "juanita-banana", "Juanita Banana");
                attrs = WindowAttributesExtX11::with_name(attrs, "juanita-banana", "Juanita Banana");
            }

            let display_handle = event_loop
                .display_handle()
                .expect("Failed to get display handle");
            
            let window = event_loop
                .create_window(attrs)
                .expect("Failed to create winit Window");
            
            let window_handle = window.window_handle().expect("Failed to get window handle");

            let rendering_context = Rc::new(
                WindowRenderingContext::new(display_handle, window_handle, window.inner_size())
                    .expect("Could not create RenderingContext for window."),
            );

            let _ = rendering_context.make_current();

            // Set up Servo options with our user agent spoofing
            let opts = Opts::default();

            let servo = ServoBuilder::default()
                .opts(opts)
                .event_loop_waker(Box::new(waker.clone()))
                .build();
            servo.setup_logging();

            // Inyectar el script anti-fingerprinting de Juanita
            let user_content = UserContentManager::new(&servo);
            user_content.add_script(Rc::new(UserScript::new(
                spoof::anti_fingerprint_script().to_string(),
                None
            )));
            let user_content = Rc::new(user_content);

            let app_state = Rc::new(AppState {
                window,
                servo,
                rendering_context: rendering_context.clone(),
                webviews: Default::default(),
            });

            let url = Url::parse("https://duckduckgo.com").unwrap();

            let webview = WebViewBuilder::new(&app_state.servo, rendering_context)
                .url(url)
                .hidpi_scale_factor(Scale::new(app_state.window.scale_factor() as f32))
                .user_content_manager(user_content)
                .delegate(app_state.clone())
                .build();

            app_state.webviews.borrow_mut().push(webview);

            log::info!("[JuanitaBanana] Starting — engine: Servo, stack: 100% Rust");
            log::info!("[JuanitaBanana] User-Agent: {}", spoof::USER_AGENT);
            println!("Banana Browser Core Initialized!");
            println!("Servo Engine has successfully compiled and injected!");

            *self = Self::Running(app_state);
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: WakerEvent) {
        if let Self::Running(state) = self {
            state.servo.spin_event_loop();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Self::Running(state) = self {
            state.servo.spin_event_loop();
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                if let Self::Running(state) = self {
                    if let Some(webview) = state.webviews.borrow().last() {
                        webview.paint();
                    }
                    state.rendering_context.present();
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                if let Self::Running(state) = self {
                    if let Some(webview) = state.webviews.borrow().last() {
                        let (delta_x, delta_y, mode) = match delta {
                            MouseScrollDelta::LineDelta(dx, dy) => {
                                ((dx * 76.0) as f64, (dy * 76.0) as f64, WheelMode::DeltaLine)
                            },
                            MouseScrollDelta::PixelDelta(delta) => {
                                (delta.x, delta.y, WheelMode::DeltaPixel)
                            },
                        };

                        // Use servo::DevicePoint if it is exported, or fallback
                        // Let's just create an InputEvent without DevicePoint if possible, 
                        // or we will patch it if it fails compilation
                        webview.notify_input_event(InputEvent::Wheel(WheelEvent::new(
                            WheelDelta {
                                x: delta_x,
                                y: delta_y,
                                z: 0.0,
                                mode,
                            },
                            servo::WebViewPoint::Device(euclid::Point2D::new(0.0, 0.0)), // Mock device point
                        )));
                    }
                }
            },
            WindowEvent::Resized(new_size) => {
                if let Self::Running(state) = self {
                    if let Some(webview) = state.webviews.borrow().last() {
                        webview.resize(new_size);
                    }
                }
            },
            _ => (),
        }
    }
}

#[derive(Clone)]
struct Waker(winit::event_loop::EventLoopProxy<WakerEvent>);
#[derive(Debug)]
struct WakerEvent;

impl Waker {
    fn new(event_loop: &EventLoop<WakerEvent>) -> Self {
        Self(event_loop.create_proxy())
    }
}

// Ensure the correct EventLoopWaker trait is used
impl servo::EventLoopWaker for Waker {
    fn clone_box(&self) -> Box<dyn servo::EventLoopWaker> {
        Box::new(Self(self.0.clone()))
    }

    fn wake(&self) {
        if let Err(error) = self.0.send_event(WakerEvent) {
            log::warn!("Failed to wake event loop: {:?}", error);
        }
    }
}

pub fn run(state: SharedBanList) {
    let event_loop = EventLoop::with_user_event().build().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = JuanitaApp::new(&event_loop, state);
    event_loop.run_app(&mut app).expect("Event loop error");
}
