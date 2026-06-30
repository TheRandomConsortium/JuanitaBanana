use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, HeaderBar, Entry, Button, Box as GtkBox, Orientation};
use webkit2gtk::{WebView, WebViewExt, WebViewExtManual, WebContext, UserContentManager, UserContentManagerExt, UserScript, UserScriptInjectionTime, UserContentInjectedFrames, PolicyDecisionType, NavigationPolicyDecision, NavigationPolicyDecisionExt, URIRequestExt};
use std::rc::Rc;
use std::cell::RefCell;

use crate::browser::SharedBanList;
use crate::spoof;

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
            UserContentInjectedFrames::TopFrame,
            UserScriptInjectionTime::Start,
            &[],
            &[],
        );
        ucm.add_script(&script);

        let webview = WebView::new_with_context_and_user_content_manager(&web_context, &ucm);

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

        let webview_clone = webview.clone();
        let url_entry_clone = url_entry.clone();
        
        url_entry.connect_activate(move |entry| {
            let text = entry.text();
            let url = crate::browser::normalize_url(&text);
            webview_clone.load_uri(&url);
        });

        let webview_clone2 = webview.clone();
        let banlist_btn = banlist.clone();
        ban_button.connect_clicked(move |_| {
            if let Some(uri) = webview_clone2.uri() {
                let domain = crate::browser::extract_domain(uri.as_str());
                let mut bl = banlist_btn.borrow_mut();
                bl.ban(&domain);
                bl.save();
                println!("[BAN] Banned domain: {}", domain);
                let banned_html = crate::ban::banned_page(uri.as_str());
                webview_clone2.load_html(&banned_html, Some("juanita://banned"));
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
                    if let Some(req) = nav_decision.request() {
                        if let Some(uri) = req.uri() {
                            if banlist_nav.borrow().is_banned(uri.as_str()) {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                let banned_html = crate::ban::banned_page(uri.as_str());
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
