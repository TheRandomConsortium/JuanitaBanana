use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Entry, HeaderBar, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{
    NavigationPolicyDecision, NavigationPolicyDecisionExt, PolicyDecisionType, URIRequestExt,
    UserContentInjectedFrames, UserContentManager, UserContentManagerExt, UserScript,
    UserScriptInjectionTime, WebContext, WebView, WebViewExt,
};

use crate::browsing::browser::SharedBanList;
use crate::fingerprint::spoof;
use crate::log;
use crate::search::intoxication::IntoxicationEngine;
use crate::search::noise::RssNoiseProvider;
use crate::util::config::AppConfig;

pub fn run(banlist: SharedBanList) {
    let app = Application::builder()
        .application_id("org.juanitabanana.Browser")
        .flags(gtk::gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    let (tx, rx) = async_channel::unbounded::<(String, bool)>();

    let pending_urls: Rc<RefCell<Vec<(String, bool)>>> = Rc::new(RefCell::new(Vec::new()));
    let pending_open = pending_urls.clone();

    app.connect_open(move |app, files, _hint| {
        log!(raw:Debug, GUI, "connect_open: processing {} files", files.len());
        for file in files {
            let uri = file.uri();
            log!(raw:Debug, GUI, "connect_open file URI: {}", uri);
            pending_open.borrow_mut().push((uri.to_string(), true));
        }
        app.activate();
    });

    let global_webview: Rc<RefCell<Option<WebView>>> = Rc::new(RefCell::new(None));
    let gw_rx = global_webview.clone();
    let rx_banlist = banlist.clone();
    let app_rx = app.clone();

    let pending_rx = pending_urls.clone();

    gtk::glib::spawn_future_local(async move {
        while let Ok((url, is_external)) = rx.recv().await {
            log!(
                Debug,
                GUI,
                "rx channel received URL: {}, is_external = {}",
                url,
                is_external
            );
            if let Some(wv) = gw_rx.borrow().as_ref() {
                if is_external {
                    let domain = crate::browsing::browser::extract_domain(&url);
                    if rx_banlist.borrow().is_banned(&domain) {
                        let refuse_html = "<html><head><style>
                            body { background: #000; color: #ff3333; display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; margin: 0; font-family: monospace; font-size: 2rem; text-align: center; }
                            </style></head><body>
                            <div style=\"font-size: 8rem; margin-bottom: 20px;\">🛑</div>
                            <div>We politely refuse on your behalf to open this shithole.</div>
                            <div style=\"margin-top: 20px; font-size: 1.5rem; color: #888;\">Closing window in 5 seconds...</div>
                            </body></html>";
                        wv.load_html(refuse_html, Some("juanita://refuse/"));

                        let app_clone = app_rx.clone();
                        gtk::glib::timeout_add_seconds_local(5, move || {
                            app_clone.quit();
                            gtk::glib::ControlFlow::Break
                        });
                        continue;
                    }
                }
                log!(Debug, GUI, "webview loading URL: {}", url);
                wv.load_uri(&url);
            } else {
                log!(
                    Debug,
                    GUI,
                    "webview not yet ready, pushing to pending: {}",
                    url
                );
                pending_rx.borrow_mut().push((url, is_external));
            }
        }
    });

    let banlist_clone = banlist.clone();
    let gw_activate = global_webview.clone();
    let tx_activate = tx.clone();
    let pending_activate = pending_urls.clone();

    app.connect_activate(move |app| {
        let banlist = banlist_clone.clone();

        let config = AppConfig::load();
        let web_context = WebContext::default().unwrap();
        let ucm = UserContentManager::new();

        let script = UserScript::new(
            spoof::anti_fingerprint_script(),
            UserContentInjectedFrames::AllFrames,
            UserScriptInjectionTime::Start,
            &[],
            &[],
        );
        ucm.add_script(&script);

        ucm.register_script_message_handler("juanita");
        let ad_script = UserScript::new(
            &crate::ad_intoxication::ad_intoxication_script(&config),
            UserContentInjectedFrames::AllFrames,
            UserScriptInjectionTime::Start,
            &[],
            &[],
        );
        ucm.add_script(&ad_script);

        let toxic_script = UserScript::new(
            &crate::util::ban::toxic_warning_script(&config),
            UserContentInjectedFrames::TopFrame,
            UserScriptInjectionTime::Start,
            &[],
            &[],
        );
        ucm.add_script(&toxic_script);

        let form_mon_script = UserScript::new(
            crate::browsing::credentials_ui::form_monitor_script(),
            UserContentInjectedFrames::TopFrame,
            UserScriptInjectionTime::End,
            &[],
            &[],
        );
        ucm.add_script(&form_mon_script);

        let form_interact_script = UserScript::new(
            crate::browsing::credentials_ui::form_interact_script(),
            UserContentInjectedFrames::TopFrame,
            UserScriptInjectionTime::End,
            &[],
            &[],
        );
        ucm.add_script(&form_interact_script);

        let settings = webkit2gtk::Settings::builder()
            .user_agent("JuanitaBanana/0.1 (FOSS; Not-Google; Linux)")
            .build();
        let webview = WebView::builder()
            .web_context(&web_context)
            .user_content_manager(&ucm)
            .settings(&settings)
            .build();

        let ad_intox_engine = Rc::new(crate::ad_intoxication::AdIntoxicationEngine::new(
            &web_context,
            &webview,
            &config,
        ));

        let global_window: Rc<RefCell<Option<ApplicationWindow>>> = Rc::new(RefCell::new(None));
        let window_msg = global_window.clone();
        let ad_engine_msg = ad_intox_engine.clone();
        let banlist_msg = banlist.clone();
        let webview_msg = webview.clone();
        ucm.connect_script_message_received(Some("juanita"), move |_manager, js_result| {
            crate::browsing::message_handler::handle_script_message(
                js_result,
                &webview_msg,
                &window_msg,
                &banlist_msg,
                &ad_engine_msg,
            );
        });

        *gw_activate.borrow_mut() = Some(webview.clone());

        let has_pending = !pending_activate.borrow().is_empty();
        log!(raw:Debug, GUI, "connect_activate: has_pending = {}", has_pending);
        {
            let mut pending = pending_activate.borrow_mut();
            for (url, is_external) in pending.drain(..) {
                log!(Debug, GUI, "draining pending URL: {}", url);
                let _ = tx_activate.send_blocking((url, is_external));
            }
        }

        let downloads = Rc::new(RefCell::new(crate::util::downloads::DownloadManager::new()));
        crate::util::downloads::setup_downloads(&web_context, &downloads, &tx_activate);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Juanita Banana 🍌")
            .default_width(1280)
            .default_height(800)
            .build();

        // Resolve the deferred window reference used by the JS message handler
        *global_window.borrow_mut() = Some(window.clone());

        let header = HeaderBar::new();
        header.set_show_close_button(true);
        window.set_titlebar(Some(&header));

        let current_uri = Rc::new(RefCell::new(String::new()));

        let url_entry = Entry::builder()
            .placeholder_text("Search or enter address...")
            .hexpand(true)
            .build();

        let current_uri_focus = current_uri.clone();
        url_entry.connect_focus_in_event(move |entry, _| {
            let uri = current_uri_focus.borrow();
            entry.set_text(&crate::util::debug::redact_uri(&uri));
            gtk::glib::Propagation::Proceed
        });

        let current_uri_blur = current_uri.clone();
        url_entry.connect_focus_out_event(move |entry, _| {
            let uri = current_uri_blur.borrow();
            let display_uri = if let Some((base, _)) = uri.split_once('?') {
                base.to_string()
            } else {
                uri.to_string()
            };
            entry.set_text(&display_uri);
            gtk::glib::Propagation::Proceed
        });

        let ban_button = Button::with_label("BAN");
        ban_button.style_context().add_class("destructive-action");

        header.set_custom_title(Some(&url_entry));
        header.pack_end(&ban_button);

        let key_button = Button::with_label("🔑");
        key_button.set_no_show_all(true);
        key_button.set_visible(false);
        header.pack_start(&key_button);

        let webview_key = webview.clone();
        let window_key = window.clone();
        key_button.connect_clicked(move |_| {
            crate::browsing::credentials_ui::try_autofill(&webview_key, &window_key);
        });

        let vbox = GtkBox::new(Orientation::Vertical, 0);
        vbox.pack_start(&webview, true, true, 0);
        window.add(&vbox);

        // Set up custom actions & context menu plugin
        use crate::browsing::gui_plugin::{AdIntoxicationPlugin, GuiPlugin};
        use crate::plugins::unsubscribe::AggressiveUnsubscribePlugin;
        let plugin = AdIntoxicationPlugin;
        plugin.setup(&window, &webview, &ad_intox_engine);

        let unsub_plugin = AggressiveUnsubscribePlugin;
        unsub_plugin.setup(&window, &webview, &ad_intox_engine);

        let config = AppConfig::load();
        let noise_provider = Rc::new(RssNoiseProvider::new(&config));
        let intox_engine = IntoxicationEngine::new(&web_context, &webview, &config);

        let expected_unban: Rc<RefCell<Option<(String, i32)>>> = Rc::new(RefCell::new(None));
        let internal_pages = Rc::new(crate::browsing::internal::get_internal_pages());

        let webview_clone = webview.clone();
        let url_entry_clone = url_entry.clone();
        let intox_engine_entry = intox_engine.clone(); // Clone for entry closure
        let expected_unban_activate = expected_unban.clone();
        let downloads_activate = downloads.clone();
        let banlist_activate = banlist.clone();
        let internal_pages_activate = internal_pages.clone();

        url_entry.connect_activate(move |entry| {
            let text = entry.text();
            let text_str = text.as_str();
            intox_engine_entry.cancel_pending(); // User is initiating a new navigation!

            let ctx = crate::browsing::internal::PageContext {
                webview: webview_clone.clone(),
                downloads: downloads_activate.clone(),
                banlist: banlist_activate.clone(),
                expected_unban: expected_unban_activate.clone(),
                config: AppConfig::load(),
            };

            let mut handled = false;
            for page in internal_pages_activate.iter() {
                if page.matches_input(text_str) {
                    page.handle_input(text_str, &ctx);
                    handled = true;
                    break;
                }
            }
            if !handled {
                let url = crate::browsing::browser::normalize_url(text_str);
                webview_clone.load_uri(&url);
            }
        });

        let webview_clone2 = webview.clone();
        let banlist_btn = banlist.clone();
        ban_button.connect_clicked(move |_| {
            if let Some(uri) = webview_clone2.uri() {
                let domain = crate::browsing::browser::extract_domain(uri.as_str());
                {
                    let mut bl = banlist_btn.borrow_mut();
                    bl.ban(&domain);
                    bl.save();
                }
                println!("[BAN] Banned domain: {}", domain);
                let banned_html = crate::util::ban::banned_page(uri.as_str());
                webview_clone2.load_html(&banned_html, Some("juanita://banned/"));
            }
        });

        let url_entry_nav = url_entry_clone.clone();
        let key_button_clone = key_button.clone();
        let intox_engine_load = intox_engine.clone();
        let current_uri_nav = current_uri.clone();
        webview.connect_load_changed(move |wv, load_event| match load_event {
            webkit2gtk::LoadEvent::Started => {
                intox_engine_load.cancel_pending();
                key_button_clone.set_visible(false);
                if let Some(uri) = wv.uri() {
                    let uri_str = uri.as_str();
                    let restored_uri = crate::resolver::restore_original_domain_in_uri(uri_str);
                    *current_uri_nav.borrow_mut() = restored_uri.clone();
                    if !url_entry_nav.has_focus() {
                        let display_uri = if let Some((base, _)) = restored_uri.split_once('?') {
                            base.to_string()
                        } else {
                            restored_uri.clone()
                        };
                        url_entry_nav.set_text(&display_uri);
                    }
                }
            }
            webkit2gtk::LoadEvent::Committed => {
                if let Some(uri) = wv.uri() {
                    let uri_str = uri.as_str();
                    let restored_uri = crate::resolver::restore_original_domain_in_uri(uri_str);
                    *current_uri_nav.borrow_mut() = restored_uri.clone();
                    if !url_entry_nav.has_focus() {
                        let display_uri = if let Some((base, _)) = restored_uri.split_once('?') {
                            base.to_string()
                        } else {
                            restored_uri.clone()
                        };
                        url_entry_nav.set_text(&display_uri);
                    }
                }
            }
            webkit2gtk::LoadEvent::Finished => {
                let has_creds = if let Some(uri) = wv.uri() {
                    let domain = crate::browsing::browser::extract_domain(uri.as_str());
                    if !domain.is_empty()
                        && !uri.starts_with("juanita://")
                        && !uri.starts_with("about:")
                    {
                        let idx = crate::util::credentials::CredentialIndex::load();
                        idx.has_credentials(&domain)
                    } else {
                        false
                    }
                } else {
                    false
                };
                key_button_clone.set_visible(has_creds);
            }
            _ => {}
        });

        let webview_nav = webview.clone();
        let expected_unban_policy = expected_unban.clone();
        let downloads_policy = downloads.clone();
        let banlist_policy = banlist.clone();
        let internal_pages_policy = internal_pages.clone();
        let ad_intox_engine_policy = ad_intox_engine.clone();

        let webview_create = webview.clone();
        webview.connect_create(move |_wv, nav_action| {
            #[allow(deprecated)]
            if let Some(req) = nav_action.request() {
                if let Some(uri) = req.uri() {
                    webview_create.load_uri(uri.as_str());
                }
            }
            None // Deny new window, open in same tab
        });

        webview.connect_decide_policy(move |_, decision, decision_type| {
            if decision_type == PolicyDecisionType::NavigationAction {
                if let Some(nav_decision) = decision.downcast_ref::<NavigationPolicyDecision>() {
                    #[allow(deprecated)]
                    if let Some(req) = nav_decision.request() {
                        if let Some(uri) = req.uri() {
                            let uri_str = uri.as_str();
                            if crate::browsing::policy::handle_decide_policy(
                                decision,
                                uri_str,
                                &webview_nav,
                                &downloads_policy,
                                &banlist_policy,
                                &expected_unban_policy,
                                &internal_pages_policy,
                                &ad_intox_engine_policy,
                                &intox_engine,
                                &noise_provider,
                            ) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        });

        if !has_pending {
            log!(raw:Debug, GUI, "No pending activation, loading home page");
            webview.load_uri("juanita://home");
        }
        window.show_all();
    });

    app.run();
}
