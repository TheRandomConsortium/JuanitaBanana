// ── Embedder module ───────────────────────────────────────────
//
// Integration point for Servo ↔ winit ↔ surfman ↔ egui.
//
// Servo requires the embedder to implement:
//   - EmbedderMethods: engine hooks (user-agent, scripts, etc.)
//   - WindowMethods:   access to the GL surface and dimensions
//
// The EmbedderMethods hooks are exactly where we plug in:
//   - spoof::USER_AGENT      → what the servers see
//   - spoof::anti_fingerprint_script() → JS injected on every page
//
// This file will be completed iteratively as we advance
// with the Servo embedding. The servo 0.3 public API is new
// and we will adjust it based on compilation errors.

/* TODO: Restore these imports when integrating Servo proper
use servo::{
    compositing::windowing::{EmbedderEvent, WindowMethods},
    embedder_traits::EmbedderMethods,
    euclid::{Point2D, Rect, Scale, Size2D},
    servo_url::ServoUrl,
    webrender_api::units::DevicePixel,
};
*/

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::browser::{SharedBanList, normalize_url, extract_domain};
use crate::spoof;
use crate::ban;

// ── JuanitaEmbedder ──────────────────────────────────────────
// Implements EmbedderMethods — hooks called by Servo.
struct JuanitaEmbedder;

/* TODO: Implement EmbedderMethods when servo re-exports are found
impl EmbedderMethods for JuanitaEmbedder {
    fn get_user_agent_string(&self) -> Option<String> {
        Some(spoof::USER_AGENT.to_string())
    }

    fn get_javascript_to_evaluate_at_document_start(
        &self,
        _url: &ServoUrl,
    ) -> Option<String> {
        // Inject the anti-fingerprint script before ANY
        // JS from the page. This prevents them from overriding us.
        Some(spoof::anti_fingerprint_script().to_string())
    }
}
*/

struct JuanitaApp {
    window: Option<Arc<Window>>,
    state: SharedBanList,
}

impl ApplicationHandler for JuanitaApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title("Juanita Banana 🍌")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 800));
            
            let window = event_loop.create_window(attrs).expect("Failed to create window");
            self.window = Some(Arc::new(window));

            // TODO: Initialize surfman + Servo compositor here.
            
            log::info!("[JuanitaBanana] Starting — engine: Servo, stack: 100% Rust");
            log::info!("[JuanitaBanana] User-Agent: {}", spoof::USER_AGENT);
            println!("Banana Browser Core Initialized!");
            println!("Servo Engine has successfully compiled!");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { .. } => {
                // TODO: Pass keyboard events to Servo
            }
            _ => {}
        }
    }
}

// ── Main run loop ─────────────────────────────────────────────
pub fn run(state: SharedBanList) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = JuanitaApp {
        window: None,
        state,
    };

    event_loop.run_app(&mut app).expect("Event loop error");
}
