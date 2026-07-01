use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Entry, HeaderBar, Orientation};
use std::rc::Rc;
use webkit2gtk::{
    NavigationPolicyDecision, NavigationPolicyDecisionExt, PolicyDecisionType, URIRequestExt,
    UserContentInjectedFrames, UserContentManager, UserContentManagerExt, UserScript,
    UserScriptInjectionTime, WebContext, WebView, WebViewExt,
};

use crate::browsing::browser::SharedBanList;
use crate::fingerprint::spoof;
use crate::search::intoxication::IntoxicationEngine;
use crate::search::noise::RssNoiseProvider;
use crate::util::config::AppConfig;

pub fn run(banlist: SharedBanList) {
    let app = Application::builder()
        .application_id("org.juanitabanana.Browser")
        .build();

    let banlist_clone = banlist.clone();

    app.connect_activate(move |app| {
        let banlist = banlist_clone.clone();

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

        let settings = webkit2gtk::Settings::builder()
            .user_agent("JuanitaBanana/0.1 (FOSS; Not-Google; Linux)")
            .build();
        let webview = WebView::builder()
            .web_context(&web_context)
            .user_content_manager(&ucm)
            .settings(&settings)
            .build();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Juanita Banana 🍌")
            .default_width(1280)
            .default_height(800)
            .build();

        let header = HeaderBar::new();
        header.set_show_close_button(true);
        window.set_titlebar(Some(&header));

        let url_entry = Entry::builder()
            .placeholder_text("Search or enter address...")
            .hexpand(true)
            .build();

        let ban_button = Button::with_label("BAN");
        ban_button.style_context().add_class("destructive-action");

        header.set_custom_title(Some(&url_entry));
        header.pack_end(&ban_button);

        let vbox = GtkBox::new(Orientation::Vertical, 0);
        vbox.pack_start(&webview, true, true, 0);
        window.add(&vbox);

        let config = AppConfig::load();
        let noise_provider = Rc::new(RssNoiseProvider::new(&config));
        let intox_engine = IntoxicationEngine::new(&web_context, &webview, &config);

        let webview_clone = webview.clone();
        let url_entry_clone = url_entry.clone();
        let intox_engine_entry = intox_engine.clone(); // Clone for entry closure

        url_entry.connect_activate(move |entry| {
            let text = entry.text();
            let text_str = text.as_str();
            intox_engine_entry.cancel_pending(); // User is initiating a new navigation!
            if text_str == "juanita:history" || text_str == "juanita://history" {
                let history_html = "<html><head><style>body { background: #000; color: #fff; display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; margin: 0; font-family: monospace; font-size: 3rem; }</style></head><body><div style=\"font-size: 10rem\">🖕</div><div>history? what history?</div></body></html>";
                webview_clone.load_html(history_html, Some("juanita://history-page/"));
                return;
            }
            if text_str.starts_with("juanita:config") || text_str.starts_with("juanita://config") {
                webview_clone.load_uri(text_str);
                return;
            }
            let url = crate::browsing::browser::normalize_url(text_str);
            webview_clone.load_uri(&url);
        });

        let webview_clone2 = webview.clone();
        let banlist_btn = banlist.clone();
        ban_button.connect_clicked(move |_| {
            if let Some(uri) = webview_clone2.uri() {
                let domain = crate::browsing::browser::extract_domain(uri.as_str());
                let mut bl = banlist_btn.borrow_mut();
                bl.ban(&domain);
                bl.save();
                println!("[BAN] Banned domain: {}", domain);
                let banned_html = crate::util::ban::banned_page(uri.as_str());
                webview_clone2.load_html(&banned_html, Some("juanita://banned/"));
            }
        });

        let banlist_nav = banlist.clone();
        let url_entry_nav = url_entry_clone.clone();
        webview.connect_load_changed(move |wv, load_event| {
            if load_event == webkit2gtk::LoadEvent::Started || load_event == webkit2gtk::LoadEvent::Committed {
                if let Some(uri) = wv.uri() {
                    url_entry_nav.set_text(uri.as_str());
                }
            }
        });

        let webview_nav = webview.clone();

        webview.connect_decide_policy(move |_, decision, decision_type| {
            if decision_type == PolicyDecisionType::NavigationAction {
                if let Some(nav_decision) = decision.downcast_ref::<NavigationPolicyDecision>() {
                    #[allow(deprecated)]
                    if let Some(req) = nav_decision.request() {
                        if let Some(uri) = req.uri() {
                            let uri_str = uri.as_str();

                            intox_engine.cancel_pending();

                            if uri_str == "juanita:history" || uri_str == "juanita://history" || uri_str == "juanita://history/" {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                let history_html = "<html><head><style>body { background: #000; color: #fff; display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; margin: 0; font-family: monospace; font-size: 3rem; }</style></head><body><div style=\"font-size: 10rem\">🖕</div><div>history? what history?</div></body></html>";
                                webview_nav.load_html(history_html, Some("juanita://history-page/"));
                                return true;
                            }
                            if uri_str.starts_with("juanita://config-page") {
                                return false; // Prevent infinite loop: let load_html apply the base URI
                            }

                            if uri_str.starts_with("juanita://config") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                let config_html = crate::util::config::config_page_html(&config);
                                let base_uri = uri_str.replace("juanita://config", "juanita://config-page");
                                webview_nav.load_html(&config_html, Some(&base_uri));
                                return true;
                            }
                            if let Some(data_str) = uri_str.strip_prefix("juanita://save-config?data=") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                if let Ok(decoded) = urlencoding::decode(data_str) {
                                    if let Ok(new_config) = serde_json::from_str::<crate::util::config::AppConfig>(&decoded) {
                                        new_config.save();
                                        println!("[CONFIG] Configuration saved successfully.");
                                    }
                                }
                                webview_nav.load_uri("juanita://config");
                                return true;
                            }

                            // Check for Search Intoxication
                            if intox_engine.check_and_poison_search(uri_str, &config, &*noise_provider) {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                return true;
                            }
                            if banlist_nav.borrow().is_banned(uri_str) {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                let banned_html = crate::util::ban::banned_page(uri_str);
                                webview_nav.load_html(&banned_html, Some("juanita://banned"));
                                return true;
                            }
                        }
                    }
                }
            }
            false
        });

        webview.load_uri("https://duckduckgo.com");
        window.show_all();
    });

    app.run();
}
