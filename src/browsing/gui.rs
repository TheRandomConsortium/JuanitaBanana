use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Entry, HeaderBar, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{WebContext, WebView, WebViewExt};

use crate::browsing::browser::SharedBanList;
use crate::browsing::tab::{create_tab, Tab};
use crate::log;
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

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Juanita Banana 🍌")
            .default_width(1280)
            .default_height(800)
            .build();

        let global_window: Rc<RefCell<Option<ApplicationWindow>>> = Rc::new(RefCell::new(None));
        *global_window.borrow_mut() = Some(window.clone());

        let header = HeaderBar::new();
        header.set_show_close_button(true);
        window.set_titlebar(Some(&header));

        let current_uri = Rc::new(RefCell::new(String::new()));

        let url_entry = Entry::builder()
            .placeholder_text("Search or enter address...")
            .hexpand(true)
            .build();

        let ban_button = Button::with_label("BAN");
        ban_button.style_context().add_class("destructive-action");

        header.set_custom_title(Some(&url_entry));
        header.pack_end(&ban_button);

        let key_button = Button::with_label("🔑");
        key_button.set_no_show_all(true);
        key_button.set_visible(false);
        header.pack_start(&key_button);

        // Autofill button click handler
        let gw_key = global_webview.clone();
        let window_key = window.clone();
        key_button.connect_clicked(move |_| {
            if let Some(wv) = gw_key.borrow().as_ref() {
                crate::browsing::credentials_ui::try_autofill(wv, &window_key);
            }
        });

        let vbox = GtkBox::new(Orientation::Vertical, 0);
        let notebook = gtk::Notebook::new();
        notebook.set_show_tabs(true);
        notebook.set_scrollable(true);
        vbox.pack_start(&notebook, true, true, 0);
        window.add(&vbox);

        let tabs: Rc<RefCell<Vec<Tab>>> = Rc::new(RefCell::new(Vec::new()));
        let is_cleaning = Rc::new(RefCell::new(false));

        let web_context = WebContext::default().unwrap();
        let downloads = Rc::new(RefCell::new(crate::util::downloads::DownloadManager::new()));
        crate::util::downloads::setup_downloads(&web_context, &downloads, &tx_activate);

        let expected_unban: Rc<RefCell<Option<(String, i32)>>> = Rc::new(RefCell::new(None));
        let internal_pages = Rc::new(crate::browsing::internal::get_internal_pages());

        // Shared noise provider and tab channels
        let config = AppConfig::load();
        let noise_provider = Rc::new(crate::search::noise::RssNoiseProvider::new(&config));
        let (tab_tx, tab_rx) = async_channel::unbounded::<String>();

        // Register window action for right-click context menu "Open Link in New Tab"
        let open_in_new_tab_action = gtk::gio::SimpleAction::new(
            "open-in-new-tab",
            Some(gtk::glib::VariantTy::new("s").unwrap()),
        );
        let tab_tx_act = tab_tx.clone();
        open_in_new_tab_action.connect_activate(move |_, parameter| {
            if let Some(p) = parameter {
                if let Some(url) = p.str() {
                    let _ = tab_tx_act.send_blocking(url.to_string());
                }
            }
        });
        window.add_action(&open_in_new_tab_action);

        // Helper to add a tab
        let notebook_c = notebook.clone();
        let tabs_c = tabs.clone();
        let window_c = window.clone();
        let global_window_c = global_window.clone();
        let web_context_c = web_context.clone();
        let banlist_c = banlist.clone();
        let downloads_c = downloads.clone();
        let expected_unban_c = expected_unban.clone();
        let internal_pages_c = internal_pages.clone();
        let tx_c = tx_activate.clone();
        let url_entry_c = url_entry.clone();
        let current_uri_c = current_uri.clone();
        let key_button_c = key_button.clone();
        let is_cleaning_c = is_cleaning.clone();
        let gw_c = gw_activate.clone();
        let noise_provider_c = noise_provider.clone();
        let tab_tx_c = tab_tx.clone();

        let add_new_tab = move |url: &str| {
            let tab = create_tab(
                &notebook_c,
                &tabs_c,
                url,
                &window_c,
                &global_window_c,
                &web_context_c,
                &banlist_c,
                &downloads_c,
                &expected_unban_c,
                &internal_pages_c,
                &tx_c,
                &url_entry_c,
                &current_uri_c,
                &key_button_c,
                &is_cleaning_c,
                &gw_c,
                &noise_provider_c,
                &tab_tx_c,
            );
            let new_index = {
                let mut tabs_borrow = tabs_c.borrow_mut();
                tabs_borrow.push(tab);
                tabs_borrow.len() - 1
            };
            notebook_c.set_current_page(Some(new_index as u32));
        };

        // Listen for new tab creation requests
        let add_tab_channel = add_new_tab.clone();
        gtk::glib::spawn_future_local(async move {
            while let Ok(url) = tab_rx.recv().await {
                add_tab_channel(&url);
            }
        });

        // Add New Tab button "+" to headerbar
        let new_tab_btn = Button::with_label("+");
        header.pack_start(&new_tab_btn);
        let add_tab_btn_clone = add_new_tab.clone();
        new_tab_btn.connect_clicked(move |_| {
            add_tab_btn_clone("juanita://home");
        });

        // Set up active tab change / switch page event
        let tabs_switch = tabs.clone();
        let url_entry_switch = url_entry.clone();
        let key_button_switch = key_button.clone();
        let global_webview_switch = gw_activate.clone();
        let current_uri_switch = current_uri.clone();
        let is_cleaning_switch = is_cleaning.clone();
        let notebook_switch = notebook.clone();

        notebook.connect_switch_page(move |_, _page, page_num| {
            if *is_cleaning_switch.borrow() {
                return;
            }
            let tabs_borrow = tabs_switch.borrow();
            if let Some(tab) = tabs_borrow.get(page_num as usize) {
                let uri = tab.webview.uri().map(|s| s.to_string()).unwrap_or_default();
                let restored_uri = crate::resolver::restore_original_domain_in_uri(&uri);
                *current_uri_switch.borrow_mut() = restored_uri.clone();
                let display_uri = if let Some((base, _)) = restored_uri.split_once('?') {
                    base.to_string()
                } else {
                    restored_uri
                };
                url_entry_switch.set_text(&display_uri);
                *global_webview_switch.borrow_mut() = Some(tab.webview.clone());

                crate::browsing::gui_plugin::ACTIVE_TAB.with(|at| {
                    *at.borrow_mut() = Some((tab.webview.clone(), tab.ad_intox_engine.clone()));
                });

                let has_creds = if !uri.is_empty()
                    && !uri.starts_with("juanita://")
                    && !uri.starts_with("about:")
                {
                    let domain = crate::browsing::browser::extract_domain(&uri);
                    crate::util::credentials::CredentialIndex::load().has_credentials(&domain)
                } else {
                    false
                };
                key_button_switch.set_visible(has_creds);
            }

            // Clean up killed tabs on idle so it doesn't interrupt page switching
            let tabs_idle = tabs_switch.clone();
            let notebook_idle = notebook_switch.clone();
            let url_entry_idle = url_entry_switch.clone();
            let gw_idle = global_webview_switch.clone();
            let cur_idle = current_uri_switch.clone();
            let key_idle = key_button_switch.clone();
            let is_cleaning_idle = is_cleaning_switch.clone();
            gtk::glib::idle_add_local(move || {
                crate::browsing::tab_cleanup::cleanup_killed_tabs(
                    &notebook_idle,
                    &tabs_idle,
                    &url_entry_idle,
                    &gw_idle,
                    &cur_idle,
                    &key_idle,
                    &is_cleaning_idle,
                );
                gtk::glib::ControlFlow::Break
            });
        });

        // Focus in on url_entry and notebook to trigger cleanup
        let tabs_cleanup1 = tabs.clone();
        let notebook_cleanup1 = notebook.clone();
        let url_entry_cleanup1 = url_entry.clone();
        let gw_cleanup1 = gw_activate.clone();
        let cur_cleanup1 = current_uri.clone();
        let key_cleanup1 = key_button.clone();
        let is_cleaning_cleanup1 = is_cleaning.clone();
        url_entry.connect_focus_in_event(move |entry, _| {
            crate::browsing::tab_cleanup::cleanup_killed_tabs(
                &notebook_cleanup1,
                &tabs_cleanup1,
                &url_entry_cleanup1,
                &gw_cleanup1,
                &cur_cleanup1,
                &key_cleanup1,
                &is_cleaning_cleanup1,
            );
            let uri = cur_cleanup1.borrow();
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

        let tabs_cleanup2 = tabs.clone();
        let notebook_cleanup2 = notebook.clone();
        let url_entry_cleanup2 = url_entry.clone();
        let gw_cleanup2 = gw_activate.clone();
        let cur_cleanup2 = current_uri.clone();
        let key_cleanup2 = key_button.clone();
        let is_cleaning_cleanup2 = is_cleaning.clone();
        notebook.connect_button_press_event(move |_, _| {
            crate::browsing::tab_cleanup::cleanup_killed_tabs(
                &notebook_cleanup2,
                &tabs_cleanup2,
                &url_entry_cleanup2,
                &gw_cleanup2,
                &cur_cleanup2,
                &key_cleanup2,
                &is_cleaning_cleanup2,
            );
            gtk::glib::Propagation::Proceed
        });

        let tabs_cleanup3 = tabs.clone();
        let notebook_cleanup3 = notebook.clone();
        let url_entry_cleanup3 = url_entry.clone();
        let gw_cleanup3 = gw_activate.clone();
        let cur_cleanup3 = current_uri.clone();
        let key_cleanup3 = key_button.clone();
        let is_cleaning_cleanup3 = is_cleaning.clone();
        notebook.connect_enter_notify_event(move |_, _| {
            crate::browsing::tab_cleanup::cleanup_killed_tabs(
                &notebook_cleanup3,
                &tabs_cleanup3,
                &url_entry_cleanup3,
                &gw_cleanup3,
                &cur_cleanup3,
                &key_cleanup3,
                &is_cleaning_cleanup3,
            );
            gtk::glib::Propagation::Proceed
        });

        let tabs_cleanup4 = tabs.clone();
        let notebook_cleanup4 = notebook.clone();
        let url_entry_cleanup4 = url_entry.clone();
        let gw_cleanup4 = gw_activate.clone();
        let cur_cleanup4 = current_uri.clone();
        let key_cleanup4 = key_button.clone();
        let is_cleaning_cleanup4 = is_cleaning.clone();
        window.connect_focus_in_event(move |_, _| {
            crate::browsing::tab_cleanup::cleanup_killed_tabs(
                &notebook_cleanup4,
                &tabs_cleanup4,
                &url_entry_cleanup4,
                &gw_cleanup4,
                &cur_cleanup4,
                &key_cleanup4,
                &is_cleaning_cleanup4,
            );
            gtk::glib::Propagation::Proceed
        });

        // URL entry activate handler navigates the active webview
        let gw_act_entry = gw_activate.clone();
        let downloads_act_entry = downloads.clone();
        let banlist_act_entry = banlist.clone();
        let expected_unban_act_entry = expected_unban.clone();
        let internal_pages_act_entry = internal_pages.clone();
        let tabs_act_entry = tabs.clone();
        let notebook_act_entry = notebook.clone();
        url_entry.connect_activate(move |entry| {
            let text = entry.text();
            let text_str = text.as_str();

            // Cancel pending on the active tab's intox engine
            let active_idx = notebook_act_entry.current_page().unwrap_or(0) as usize;
            if let Some(tab) = tabs_act_entry.borrow().get(active_idx) {
                tab.intox_engine.cancel_pending();
            }

            if let Some(wv) = gw_act_entry.borrow().as_ref() {
                let ctx = crate::browsing::internal::PageContext {
                    webview: wv.clone(),
                    downloads: downloads_act_entry.clone(),
                    banlist: banlist_act_entry.clone(),
                    expected_unban: expected_unban_act_entry.clone(),
                    config: AppConfig::load(),
                };

                let mut handled = false;
                for page in internal_pages_act_entry.iter() {
                    if page.matches_input(text_str) {
                        page.handle_input(text_str, &ctx);
                        handled = true;
                        break;
                    }
                }
                if !handled {
                    let url = crate::browsing::browser::normalize_url(text_str);
                    wv.load_uri(&url);
                }
            }
        });

        // Ban button acts on active webview
        let gw_ban = gw_activate.clone();
        let banlist_btn = banlist.clone();
        ban_button.connect_clicked(move |_| {
            if let Some(wv) = gw_ban.borrow().as_ref() {
                if let Some(uri) = wv.uri() {
                    let domain = crate::browsing::browser::extract_domain(uri.as_str());
                    {
                        let mut bl = banlist_btn.borrow_mut();
                        bl.ban(&domain);
                        bl.save();
                    }
                    log!(Warn, GUI, "Banned domain: {}", domain);
                    let banned_html = crate::util::ban::banned_page(uri.as_str());
                    wv.load_html(&banned_html, Some("juanita://banned/"));
                }
            }
        });

        // Inactivity timeout loop checking for dead tabs
        let tabs_timer = tabs.clone();
        let window_timer = window.clone();
        gtk::glib::timeout_add_seconds_local(5, move || {
            if window_timer.in_destruction() {
                return gtk::glib::ControlFlow::Break;
            }
            crate::browsing::tab_cleanup::check_tab_inactivity(&tabs_timer);
            gtk::glib::ControlFlow::Continue
        });

        // Load the initial/pending tabs
        let has_pending = !pending_activate.borrow().is_empty();
        log!(raw:Debug, GUI, "connect_activate: has_pending = {}", has_pending);

        let mut initial_loaded = false;
        {
            let mut pending = pending_activate.borrow_mut();
            for (url, _is_external) in pending.drain(..) {
                add_new_tab(&url);
                initial_loaded = true;
            }
        }

        if !initial_loaded {
            log!(raw:Debug, GUI, "No pending activation, loading home page");
            add_new_tab("juanita://home");
        }

        window.show_all();
    });

    app.run();
}
