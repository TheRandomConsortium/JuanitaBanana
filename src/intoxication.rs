use gtk::glib;
use rand::Rng;
use regex::Regex;
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;
use std::time::Duration;
use webkit2gtk::{LoadEvent, WebContext, WebView, WebViewExt};
use gtk::prelude::WidgetExtManual;

use crate::config::AppConfig;
use crate::noise::NoiseProvider;

pub enum IntoxicationTask {
    FakeSearch(String),
    RealSearch(String, u64),
}

pub struct IntoxicationEngine {
    queue: Rc<RefCell<VecDeque<IntoxicationTask>>>,
    active_count: Rc<RefCell<usize>>,
    active_views: Rc<RefCell<Vec<WebView>>>,
    allowed_urls: Rc<RefCell<HashSet<String>>>,
    search_session_id: Rc<RefCell<u64>>,
    context: WebContext,
    main_webview: WebView,
    min_delay_ms: u64,
    max_delay_ms: u64,
    max_concurrent_searches: usize,
}

impl IntoxicationEngine {
    pub fn new(context: &WebContext, main_webview: &WebView, config: &AppConfig) -> Self {
        Self {
            queue: Rc::new(RefCell::new(VecDeque::new())),
            active_count: Rc::new(RefCell::new(0)),
            active_views: Rc::new(RefCell::new(Vec::new())),
            allowed_urls: Rc::new(RefCell::new(HashSet::new())),
            search_session_id: Rc::new(RefCell::new(0)),
            context: context.clone(),
            main_webview: main_webview.clone(),
            min_delay_ms: config.min_delay_ms,
            max_delay_ms: config.max_delay_ms,
            max_concurrent_searches: config.max_concurrent_searches,
        }
    }

    pub fn check_and_poison_search(
        &self,
        real_uri: &str,
        config: &AppConfig,
        noise: &impl NoiseProvider,
    ) -> bool {
        // Prevent infinite loop when rendering the real search
        if self.allowed_urls.borrow().contains(real_uri) {
            self.allowed_urls.borrow_mut().remove(real_uri);
            return false;
        }

        for rule in &config.search_engines {
            if let Ok(re) = Regex::new(&rule.domain_regex) {
                if re.is_match(real_uri) {
                    // Make sure the URL actually contains the query param! 
                    // Otherwise we might intercept the homepage (e.g. duckduckgo.com/)
                    let mut has_params = false;
                    for param in &rule.query_params {
                        let param_re_str = format!(r"([?&]{}=)[^&]+", regex::escape(param));
                        if let Ok(param_re) = Regex::new(&param_re_str) {
                            if param_re.is_match(real_uri) {
                                has_params = true;
                                break;
                            }
                        }
                    }
                    
                    if !has_params {
                        continue;
                    }

                    println!("[INTOX] Intercepted search on {}", rule.name);
                    
                    let mut tasks = Vec::new();
                    let fake_terms = noise.get_keywords(20);
                    
                    for term in fake_terms {
                        // Replace the search terms in the URL
                        let mut fake_uri = real_uri.to_string();
                        for param in &rule.query_params {
                            // Match param=value where value is anything until & or end of string
                            let param_re_str = format!(r"([?&]{}=)[^&]+", regex::escape(param));
                            if let Ok(param_re) = Regex::new(&param_re_str) {
                                let replacement = format!("${{1}}{}", urlencoding::encode(&term));
                                fake_uri = param_re.replace_all(&fake_uri, replacement.as_str()).to_string();
                            }
                        }
                        tasks.push(IntoxicationTask::FakeSearch(fake_uri));
                    }
                    
                    // The Camouflage Shuffle: Insert the real search dynamically
                    let mut rng = rand::thread_rng();
                    let max_idx = (config.max_concurrent_searches * 2).saturating_sub(1);
                    let insert_idx = rng.gen_range(0..=max_idx.min(tasks.len()));
                    
                    *self.search_session_id.borrow_mut() += 1;
                    let session_id = *self.search_session_id.borrow();
                    tasks.insert(insert_idx, IntoxicationTask::RealSearch(real_uri.to_string(), session_id));
                    
                    // Add all to queue
                    self.queue.borrow_mut().extend(tasks);
                    
                    // Start processing (kickstart up to max concurrency)
                    for _ in 0..self.max_concurrent_searches {
                        self.process_queue();
                    }
                    
                    return true;
                }
            }
        }
        false
    }

    pub fn cancel_pending(&self) {
        *self.search_session_id.borrow_mut() += 1;
    }

    fn process_queue(&self) {
        let active = *self.active_count.borrow();
        if active >= self.max_concurrent_searches {
            return; // Max concurrency reached
        }
        
        let task = self.queue.borrow_mut().pop_front();
        if let Some(task) = task {
            *self.active_count.borrow_mut() += 1;
            
            let engine = self.clone();
            
            // Humanized delay before executing
            let mut rng = rand::thread_rng();
            let delay = rng.gen_range(self.min_delay_ms..=self.max_delay_ms);
            
            glib::timeout_add_local(Duration::from_millis(delay), move || {
                match &task {
                    IntoxicationTask::FakeSearch(uri) => {
                        println!("[INTOX] Firing background noise: {}", uri);
                        let settings = webkit2gtk::Settings::builder()
                            .user_agent("JuanitaBanana/0.1 (FOSS; Not-Google; Linux)")
                            .build();
                        let hidden_wv = webkit2gtk::WebView::builder()
                            .web_context(&engine.context)
                            .settings(&settings)
                            .build();
                        engine.active_views.borrow_mut().push(hidden_wv.clone());
                        
                        let engine_clone = engine.clone();
                        hidden_wv.connect_load_changed(move |wv, load_event| {
                            if load_event == LoadEvent::Committed || load_event == LoadEvent::Finished {
                                // Only process once per webview! (Finished might fire after Committed)
                                // We remove it from active_views. If it was already removed, we ignore it to prevent double-decrement.
                                let mut views = engine_clone.active_views.borrow_mut();
                                if views.contains(wv) {
                                    views.retain(|v| v != wv);
                                    let wv_to_destroy = wv.clone();
                                    glib::idle_add_local(move || {
                                        unsafe { wv_to_destroy.destroy(); }
                                        glib::ControlFlow::Break
                                    });
                                    *engine_clone.active_count.borrow_mut() -= 1;
                                    engine_clone.process_queue(); // Trigger next
                                }
                            }
                        });
                        
                        hidden_wv.load_uri(uri);
                    },
                    IntoxicationTask::RealSearch(uri, session_id) => {
                        if *engine.search_session_id.borrow() == *session_id {
                            println!("[INTOX] Rending REAL search after camouflage delay: {}", uri);
                            *engine.active_count.borrow_mut() -= 1;
                            engine.allowed_urls.borrow_mut().insert(uri.clone());
                            engine.main_webview.load_uri(uri);
                        } else {
                            println!("[INTOX] Discarding stale search (user navigated away): {}", uri);
                            *engine.active_count.borrow_mut() -= 1;
                        }
                        engine.process_queue(); // Trigger next
                    }
                }
                gtk::glib::ControlFlow::Break
            });
        }
    }
}

// Implement Clone to allow passing engine into closures easily
impl Clone for IntoxicationEngine {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            active_count: self.active_count.clone(),
            active_views: self.active_views.clone(),
            allowed_urls: self.allowed_urls.clone(),
            search_session_id: self.search_session_id.clone(),
            context: self.context.clone(),
            main_webview: self.main_webview.clone(),
            min_delay_ms: self.min_delay_ms,
            max_delay_ms: self.max_delay_ms,
            max_concurrent_searches: self.max_concurrent_searches,
        }
    }
}
