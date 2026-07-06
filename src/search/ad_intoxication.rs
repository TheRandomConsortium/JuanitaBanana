use crate::util::config::AppConfig;
use gtk::glib;
use gtk::prelude::Cast;
use gtk::prelude::WidgetExtManual;
use rand::Rng;
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;
use std::time::Duration;
use webkit2gtk::{
    LoadEvent, UserContentInjectedFrames, UserContentManager, UserContentManagerExt, UserScript,
    UserScriptInjectionTime, WebContext, WebView, WebViewExt,
};

#[derive(Clone)]
pub struct AdTask {
    pub page_url: String,
    pub selector: String,
    pub ad_url: String,
}

pub struct AdIntoxicationEngine {
    queue: Rc<RefCell<VecDeque<AdTask>>>,
    processed: Rc<RefCell<HashSet<String>>>,
    active_view: Rc<RefCell<Option<WebView>>>,
    context: WebContext,
    main_webview: WebView,
    click_probability: f64,
    jitter_min_secs: u64,
    jitter_max_secs: u64,
}

impl AdIntoxicationEngine {
    pub fn new(context: &WebContext, main_webview: &WebView, config: &AppConfig) -> Self {
        let engine = Self {
            queue: Rc::new(RefCell::new(VecDeque::new())),
            processed: Rc::new(RefCell::new(HashSet::new())),
            active_view: Rc::new(RefCell::new(None)),
            context: context.clone(),
            main_webview: main_webview.clone(),
            click_probability: config.ad_click_probability,
            jitter_min_secs: config.ad_jitter_min_secs,
            jitter_max_secs: config.ad_jitter_max_secs,
        };

        let engine_clone = engine.clone();
        glib::timeout_add_local(Duration::from_millis(5000), move || {
            engine_clone.process_next();
            glib::ControlFlow::Continue
        });

        engine
    }

    pub fn queue_ad(&self, task: AdTask) {
        let key = format!("{}|{}", task.page_url, task.ad_url);
        let mut processed = self.processed.borrow_mut();
        if processed.contains(&key) {
            return;
        }
        processed.insert(key);

        println!(
            "[AD_INTOX] Queued ad from {}: {}",
            task.page_url, task.ad_url
        );
        self.queue.borrow_mut().push_back(task);
    }

    pub fn learn_ad_domain(&self, domain: String) {
        if domain.is_empty() || domain == "unknown" {
            return;
        }
        let mut config = AppConfig::load();
        if !config.ad_domains.contains(&domain) {
            println!("[AD_INTOX] Learning new ad domain: {}", domain);
            config.ad_domains.push(domain);
            config.save();
        }
    }

    pub fn is_ad_domain(&self, url: &str) -> bool {
        if url.starts_with("juanita://") {
            return false;
        }
        let config = AppConfig::load();
        let domain = crate::browsing::browser::extract_domain(url);
        config
            .ad_domains
            .iter()
            .any(|d| domain.contains(d) || url.contains(d))
    }

    fn process_next(&self) {
        if self.active_view.borrow().is_some() {
            return;
        }

        let main_uri = self.main_webview.uri().unwrap_or_default().to_string();
        let main_domain = crate::browsing::browser::extract_domain(&main_uri);
        {
            let mut queue = self.queue.borrow_mut();
            queue.retain(|task| {
                let task_domain = crate::browsing::browser::extract_domain(&task.page_url);
                task_domain == main_domain
            });
        }

        let task = self.queue.borrow_mut().pop_front();
        if let Some(task) = task {
            let mut rng = rand::thread_rng();
            let click = rng.gen_bool(self.click_probability);
            let jitter_secs = rng.gen_range(self.jitter_min_secs..=self.jitter_max_secs);
            let engine_clone = self.clone();

            println!(
                "[AD_INTOX] Processing ad. Click dice result: {}. Waiting jitter of {}s...",
                click, jitter_secs
            );

            glib::timeout_add_local(Duration::from_secs(jitter_secs), move || {
                if click {
                    engine_clone.execute_click(task.clone());
                }
                glib::ControlFlow::Break
            });
        }
    }

    fn execute_click(&self, task: AdTask) {
        println!(
            "[AD_INTOX] Launching headless clone for page: {}",
            task.page_url
        );

        let settings = self.main_webview.settings();

        // Create a new headless UserContentManager that only has the anti-fingerprint script,
        // specifically omitting the ad-intoxication script and script message handlers to prevent loops.
        let headless_ucm = UserContentManager::new();
        let anti_fp_script = UserScript::new(
            crate::fingerprint::spoof::anti_fingerprint_script(),
            UserContentInjectedFrames::AllFrames,
            UserScriptInjectionTime::Start,
            &[],
            &[],
        );
        headless_ucm.add_script(&anti_fp_script);

        let mut builder = WebView::builder()
            .web_context(&self.context)
            .user_content_manager(&headless_ucm);
        if let Some(ref sets) = settings {
            builder = builder.settings(sets);
        }
        let hidden_wv = builder.build();
        *self.active_view.borrow_mut() = Some(hidden_wv.clone());

        hidden_wv.connect_create(move |wv, _action| Some(wv.clone().upcast::<gtk::Widget>()));

        let engine_clone = self.clone();
        let task_clone = task.clone();

        let hidden_wv_timeout = hidden_wv.clone();
        let engine_timeout = self.clone();
        glib::timeout_add_local(Duration::from_secs(15), move || {
            println!("[AD_INTOX] Headless session timed out. Cleaning up.");
            engine_timeout.cleanup_session(&hidden_wv_timeout);
            glib::ControlFlow::Break
        });

        let clicked = Rc::new(RefCell::new(false));
        let clicked_clone = clicked.clone();
        hidden_wv.connect_load_changed(move |wv, load_event| {
            if load_event == LoadEvent::Finished {
                let current_uri = wv.uri().unwrap_or_default().to_string();
                println!("[AD_INTOX] Finished loading: {}", current_uri);

                if !*clicked_clone.borrow() {
                    *clicked_clone.borrow_mut() = true;
                    println!(
                        "[AD_INTOX] Simulating ghost mouse on: {}",
                        task_clone.selector
                    );

                    let ghost_mouse_js = format!(
                        r#"
                            (function() {{
                                const selector = `{}`;
                                const adUrl = `{}`;
                                let el = null;
                                try {{
                                    if (selector) {{
                                        el = document.querySelector(selector);
                                    }}
                                }} catch(e) {{
                                    console.log("[AD_INTOX] Invalid query selector: " + selector);
                                }}

                                // Fallback 1: Try to search for elements matching the adUrl
                                if (!el && adUrl) {{
                                    const all = document.querySelectorAll('iframe, img, a, script');
                                    for (let candidate of all) {{
                                        const src = candidate.getAttribute('src') || '';
                                        const href = candidate.getAttribute('href') || '';
                                        if (src.includes(adUrl) || adUrl.includes(src) || href.includes(adUrl) || adUrl.includes(href)) {{
                                            el = candidate;
                                            break;
                                        }}
                                    }}
                                }}

                                // Fallback 2: Synthesize a target element so we can click it and trigger destination navigation
                                if (!el && adUrl) {{
                                    console.log("[AD_INTOX] Ad element not found. Synthesizing target link.");
                                    el = document.createElement('a');
                                    el.href = adUrl;
                                    el.style.position = 'absolute';
                                    el.style.left = '-9999px';
                                    el.style.top = '-9999px';
                                    el.style.width = '10px';
                                    el.style.height = '10px';
                                    document.body.appendChild(el);
                                }}

                                if (!el) {{
                                    return;
                                }}

                                el.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                                
                                const hoverEvents = ['mouseover', 'mouseenter', 'mousemove'];
                                hoverEvents.forEach(evt => {{
                                    el.dispatchEvent(new MouseEvent(evt, {{
                                        bubbles: true,
                                        cancelable: true,
                                        view: window
                                    }}));
                                }});

                                setTimeout(() => {{
                                    const clickEvents = ['mousedown', 'mouseup', 'click'];
                                    clickEvents.forEach(evt => {{
                                        el.dispatchEvent(new MouseEvent(evt, {{
                                            bubbles: true,
                                            cancelable: true,
                                            view: window
                                        }}));
                                    }});
                                    if (el.click) {{
                                        el.click();
                                    }}
                                }}, 1000 + Math.random() * 1000);
                            }})();
                            "#,
                        task_clone.selector, task_clone.ad_url
                    );

                    #[allow(deprecated)]
                    wv.run_javascript(
                        &ghost_mouse_js,
                        None::<&webkit2gtk::gio::Cancellable>,
                        |res| {
                            if let Err(e) = res {
                                println!("[AD_INTOX] Ghost mouse JS error: {:?}", e);
                            }
                        },
                    );
                } else {
                    let is_real_dest = current_uri != task_clone.page_url
                        && !current_uri.contains("googleads")
                        && !current_uri.contains("doubleclick");
                    if is_real_dest {
                        println!(
                            "[AD_INTOX] Successfully reached final landing destination: {}",
                            current_uri
                        );
                        engine_clone.cleanup_session(wv);
                    }
                }
            }
        });

        hidden_wv.load_uri(&task.page_url);
    }

    fn cleanup_session(&self, wv: &WebView) {
        let mut active = self.active_view.borrow_mut();
        if let Some(ref active_wv) = *active {
            if active_wv == wv {
                println!("[AD_INTOX] Destroying headless WebView session");
                let wv_to_destroy = wv.clone();
                glib::idle_add_local(move || {
                    unsafe {
                        wv_to_destroy.destroy();
                    }
                    glib::ControlFlow::Break
                });
                *active = None;
            }
        }
    }
}

impl Clone for AdIntoxicationEngine {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            processed: self.processed.clone(),
            active_view: self.active_view.clone(),
            context: self.context.clone(),
            main_webview: self.main_webview.clone(),
            click_probability: self.click_probability,
            jitter_min_secs: self.jitter_min_secs,
            jitter_max_secs: self.jitter_max_secs,
        }
    }
}
