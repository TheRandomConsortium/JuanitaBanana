use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Entry, HeaderBar, Orientation};
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{
    DownloadExt, NavigationPolicyDecision, NavigationPolicyDecisionExt, PolicyDecisionType,
    URIRequestExt, UserContentInjectedFrames, UserContentManager, UserContentManagerExt,
    UserScript, UserScriptInjectionTime, WebContext, WebContextExt, WebView, WebViewExt,
};

use crate::browsing::browser::SharedBanList;
use crate::fingerprint::spoof;
use crate::search::intoxication::IntoxicationEngine;
use crate::search::noise::RssNoiseProvider;
use crate::util::config::AppConfig;

pub fn run(banlist: SharedBanList) {
    let app = Application::builder()
        .application_id("org.juanitabanana.Browser")
        .flags(gtk::gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    #[allow(deprecated)]
    let (tx, rx) = gtk::glib::MainContext::channel::<(String, bool)>(gtk::glib::Priority::DEFAULT);

    let tx_open = tx.clone();
    app.connect_open(move |app, files, _hint| {
        app.activate();
        for file in files {
            let uri = file.uri();
            let _ = tx_open.send((uri.to_string(), true));
        }
    });

    let global_webview: Rc<RefCell<Option<WebView>>> = Rc::new(RefCell::new(None));
    let gw_rx = global_webview.clone();
    let rx_banlist = banlist.clone();
    let app_rx = app.clone();

    let pending_urls: Rc<RefCell<Vec<(String, bool)>>> = Rc::new(RefCell::new(Vec::new()));
    let pending_rx = pending_urls.clone();

    rx.attach(None, move |(url, is_external)| {
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
                    return gtk::glib::ControlFlow::Continue;
                }
            }
            wv.load_uri(&url);
        } else {
            pending_rx.borrow_mut().push((url, is_external));
        }
        gtk::glib::ControlFlow::Continue
    });

    let banlist_clone = banlist.clone();
    let gw_activate = global_webview.clone();
    let tx_activate = tx.clone();
    let pending_activate = pending_urls.clone();

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

        *gw_activate.borrow_mut() = Some(webview.clone());

        let mut pending = pending_activate.borrow_mut();
        for (url, is_external) in pending.drain(..) {
            let _ = tx_activate.send((url, is_external));
        }

        let downloads = Rc::new(RefCell::new(crate::util::downloads::DownloadManager::new()));
        let downloads_ctx = downloads.clone();

        let tx_download = tx_activate.clone();
        web_context.connect_download_started(move |_context, download| {
            let id = format!("{}", rand::thread_rng().gen::<u64>());

            let mut origin_domain = String::from("unknown");
            #[allow(deprecated)]
            if let Some(req) = download.request() {
                if let Some(uri) = req.uri() {
                    origin_domain = crate::browsing::browser::extract_domain(uri.as_str());
                }
            }

            let downloads_ctx_dest = downloads_ctx.clone();
            let id_dest = id.clone();
            download.connect_decide_destination(move |dl, suggested_filename| {
                let filename = suggested_filename.to_string();
                let dest_dir = format!("/tmp/juanita-sandbox-{}", id_dest);
                std::fs::create_dir_all(&dest_dir).ok();

                let dest_path = format!("{}/{}", dest_dir, filename);
                dl.set_destination(&format!("file://{}", dest_path));

                downloads_ctx_dest.borrow_mut().active_downloads.insert(id_dest.clone(), (dest_path, filename.clone(), false, 0.0, origin_domain.clone()));

                std::process::Command::new("notify-send")
                    .arg("Juanita Banana 🍌")
                    .arg(format!("Downloading: {}", filename))
                    .spawn().ok();

                true
            });

            let downloads_ctx_prog = downloads_ctx.clone();
            let id_prog = id.clone();
            download.connect_received_data(move |dl, _data_length| {
                if let Some(entry) = downloads_ctx_prog.borrow_mut().active_downloads.get_mut(&id_prog) {
                    entry.3 = dl.estimated_progress();
                }
            });

            let downloads_fin = downloads_ctx.clone();
            let id_fin = id.clone();
            let tx_clone = tx_download.clone();
            download.connect_finished(move |_| {
                if let Some(entry) = downloads_fin.borrow_mut().active_downloads.get_mut(&id_fin) {
                    entry.2 = true;
                    let filename = entry.1.clone();
                    println!("[SANDBOX] Download finished: {}", filename);

                    let tx_thread = tx_clone.clone();
                    std::thread::spawn(move || {
                        if let Ok(out) = std::process::Command::new("notify-send")
                            .arg("--action=open=View Downloads")
                            .arg("Juanita Banana 🍌")
                            .arg(format!("Ready in Sandbox: {}", filename))
                            .output() {
                            if String::from_utf8_lossy(&out.stdout).trim() == "open" {
                                let _ = tx_thread.send(("juanita://downloads".to_string(), false));
                            }
                        }
                    });
                }
            });
        });

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
            if text_str.starts_with("juanita:unban") || text_str.starts_with("juanita://unban") || text_str.starts_with("juanita://submit-unban") {
                webview_clone.load_uri(text_str);
                return;
            }
            if text_str.starts_with("juanita:downloads") || text_str.starts_with("juanita://downloads") {
                webview_clone.load_uri(text_str);
                return;
            }
            if text_str.starts_with("juanita:contribute") || text_str.starts_with("juanita://contribute") {
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
        let expected_unban: Rc<RefCell<Option<(String, i32)>>> = Rc::new(RefCell::new(None));

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
                                let is_default = crate::util::config::is_default_browser();
                                let config_html = crate::util::config::config_page_html(&config, is_default);
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
                                webview_nav.load_uri("juanita://config?saved=true");
                                return true;
                            }
                            if uri_str.starts_with("juanita://make-default") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();

                                // Check if running from a system install (e.g. /usr/bin)
                                let exe_path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("juanita-banana"));
                                let is_system_install = exe_path.starts_with("/usr/");

                                let desktop_filename = if is_system_install {
                                    "juanita-banana.desktop".to_string()
                                } else {
                                    // Create a local dev desktop file so we don't shadow the system RPM one
                                    let base = std::env::var("XDG_DATA_HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share"));
                                    let apps_dir = base.join("applications");
                                    std::fs::create_dir_all(&apps_dir).ok();

                                    let desktop_path = apps_dir.join("juanita-banana-local.desktop");
                                    let desktop_content = format!(
                                        "[Desktop Entry]\nVersion=1.0\nName=Juanita Banana (Local)\nGenericName=Web Browser\nComment=Weaponized Privacy Browser\nExec={} %U\nTerminal=false\nX-MultipleArgs=false\nType=Application\nIcon=web-browser\nCategories=Network;WebBrowser;\nMimeType=text/html;text/xml;application/xhtml+xml;x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/juanita;\nStartupNotify=true",
                                        exe_path.display()
                                    );
                                    std::fs::write(&desktop_path, desktop_content).ok();
                                    "juanita-banana-local.desktop".to_string()
                                };

                                std::process::Command::new("xdg-settings")
                                    .arg("set")
                                    .arg("default-web-browser")
                                    .arg(&desktop_filename)
                                    .spawn()
                                    .ok();

                                println!("[CONFIG] Set as default browser!");
                                webview_nav.load_uri("juanita://config");
                                return true;
                            }

                            if uri_str.starts_with("juanita://contribute-page") {
                                return false; // allow loading the local HTML
                            }

                            if uri_str.starts_with("juanita://contribute") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();

                                // Inyectamos la imagen en el binario y la pasamos a base64
                                // Nota: Asegúrate de tener la dependencia `base64` en tu Cargo.toml
                                let image_data = include_bytes!("../../assets/monerowallet.png"); // Ajusta el path relativo a main.rs
                                let b64_image = base64::encode(image_data);

                                // ATENCIÓN: Las llaves de CSS están escapadas como {{ }} para que format! no colapse
                                let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Contribute to Juanita</title>
    <style>
        body {{
            background: #1e1e1e;
            color: #d4d4d4;
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
            text-align: center;
        }}
        h1 {{ color: #ffcc00; font-size: 3em; margin-bottom: 10px; }}
        p {{ font-size: 1.5em; margin-bottom: 30px; }}
        a {{
            color: #0098ff;
            text-decoration: none;
            font-weight: bold;
            font-size: 1.2em;
        }}
        a:hover {{ text-decoration: underline; }}
        .monero {{
            margin-top: 50px;
            background: #2d2d30;
            padding: 20px;
            border-radius: 10px;
            border: 1px solid #444;
            display: flex;
            flex-direction: column;
            align-items: center;
        }}
        .monero-title {{ color: #f26822; font-size: 1.5em; font-weight: bold; margin-bottom: 15px; }}
        .monero-qr {{ margin-bottom: 15px; border-radius: 8px; border: 2px solid #f26822; max-width: 250px; }}
        .monero-address {{
            font-family: monospace;
            background: #000;
            padding: 10px;
            border-radius: 5px;
            color: #0f0;
            word-break: break-all;
            max-width: 600px;
            user-select: all;
        }}
    </style>
</head>
<body>
    <h1>🍌 Contribute to Juanita Banana</h1>
    <p>Please contribute with code!</p>
    <a href="https://github.com/TheRandomConsortium/JuanitaBanana" target="_blank">View on GitHub</a>
    
    <div class="monero">
        <div class="monero-title">However, we also accept Monero...</div>
        <img class="monero-qr" src="data:image/png;base64,{}" alt="Monero Wallet QR">
        <div class="monero-address">48VpnekPQCmSF6QMM6FZFufcAaDSEM8mN7zzGeFuPqqZgYgv3p5V4DFFPJN6vVNgGeD2e4yXmADxcHJiSDbNHJwr3K7vBm6</div>
    </div>
</body>
</html>
                                "#, b64_image);

                                webview_nav.load_html(&html, Some("juanita://contribute-page/"));
                                return true;
                            }

                            if uri_str.starts_with("juanita://choose-competitor") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                let html = crate::util::competitors::competitors_page_html();
                                webview_nav.load_html(&html, Some("juanita://competitors-page/"));
                                return true;
                            }

                            if uri_str.starts_with("juanita://competitors-page") {
                                return false; // allow loading the local HTML
                            }

                            if let Some(desktop_str) = uri_str.strip_prefix("juanita://set-competitor?desktop=") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                let _ = std::process::Command::new("xdg-settings")
                                    .arg("set")
                                    .arg("default-web-browser")
                                    .arg(desktop_str)
                                    .output();
                                webview_nav.load_uri("juanita://config");
                                return true;
                            }

                            if uri_str.starts_with("juanita://downloads-page") {
                                return false; // allow loading the local HTML
                            }

                            if uri_str.starts_with("juanita://downloads/open?id=") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                let id = uri_str.trim_start_matches("juanita://downloads/open?id=");
                                downloads.borrow().open_sandboxed(id);
                                webview_nav.load_uri("juanita://downloads");
                                return true;
                            }

                            if uri_str.starts_with("juanita://downloads/persist?id=") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                let id = uri_str.trim_start_matches("juanita://downloads/persist?id=");
                                downloads.borrow_mut().make_permanent(id);
                                webview_nav.load_uri("juanita://downloads");
                                return true;
                            }

                            if uri_str.starts_with("juanita://downloads/delete?id=") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                let id = uri_str.trim_start_matches("juanita://downloads/delete?id=");
                                downloads.borrow_mut().shred(id);
                                webview_nav.load_uri("juanita://downloads");
                                return true;
                            }

                            if uri_str.starts_with("juanita://downloads") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                let html = downloads.borrow().generate_html();
                                webview_nav.load_html(&html, Some("juanita://downloads-page/"));
                                return true;
                            }

                            if uri_str.starts_with("juanita://unban-page") {
                                return false; // allow loading the local HTML
                            }

                            if let Some(domain_query) = uri_str.strip_prefix("juanita://unban?domain=") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                use crate::util::ban::{EquationProvider, BasicIntegralEquationProvider};
                                let provider = BasicIntegralEquationProvider;
                                let (equation, answer) = provider.generate_challenge();

                                let domain = domain_query.to_string();
                                *expected_unban.borrow_mut() = Some((domain.clone(), answer));

                                let unban_html = crate::util::ban::unban_page(&domain, &equation);
                                let base_uri = uri_str.replace("juanita://unban", "juanita://unban-page");
                                webview_nav.load_html(&unban_html, Some(&base_uri));
                                return true;
                            } else if uri_str.starts_with("juanita://unban") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();
                                let domains = banlist_nav.borrow().banned_domains.clone();
                                let list_html = crate::util::ban::unban_list_page(&domains);
                                webview_nav.load_html(&list_html, Some("juanita://unban-page/"));
                                return true;
                            }

                            if let Some(query) = uri_str.strip_prefix("juanita://submit-unban?") {
                                use webkit2gtk::PolicyDecisionExt;
                                decision.ignore();

                                // parse domain=X&answer=Y
                                let parts: Vec<&str> = query.split('&').collect();
                                let mut domain = String::new();
                                let mut answer = String::new();
                                for p in parts {
                                    if let Some(d) = p.strip_prefix("domain=") {
                                        domain = d.to_string();
                                    }
                                    if let Some(a) = p.strip_prefix("answer=") {
                                        answer = a.to_string();
                                    }
                                }

                                if let Some((expected_domain, expected_ans)) = expected_unban.borrow().as_ref() {
                                    if *expected_domain == domain && answer == expected_ans.to_string() {
                                        println!("[UNBAN] User solved the math! Unbanning {}", domain);
                                        let mut bl = banlist_nav.borrow_mut();
                                        bl.unban(&domain);
                                        bl.save();
                                        webview_nav.load_uri(&format!("https://{}", domain));
                                        return true;
                                    }
                                }

                                println!("[UNBAN] Incorrect math or tampered domain. Access denied.");
                                let banned_html = crate::util::ban::banned_page(&domain);
                                webview_nav.load_html(&banned_html, Some("juanita://banned"));
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
