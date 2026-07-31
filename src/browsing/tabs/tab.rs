use gtk::prelude::*;
use gtk::{ApplicationWindow, Button, Entry};
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{
    HitTestResultExt, NavigationPolicyDecision, NavigationPolicyDecisionExt, PolicyDecisionExt,
    PolicyDecisionType, URIRequestExt, UserContentManager, UserContentManagerExt, WebContext,
    WebView, WebViewExt,
};

use crate::browsing::browser::SharedBanList;
use crate::log;
use crate::search::intoxication::IntoxicationEngine;
use crate::search::noise::RssNoiseProvider;
use crate::util::config::AppConfig;

#[derive(Clone)]
pub struct Tab {
    pub webview: WebView,
    pub _label: gtk::Label,
    pub last_interaction: Rc<RefCell<std::time::Instant>>,
    pub is_killed: Rc<RefCell<bool>>,
    pub _original_title: Rc<RefCell<String>>,
    pub intox_engine: IntoxicationEngine,
    pub ad_intox_engine: Rc<crate::ad_intoxication::AdIntoxicationEngine>,
}

#[allow(clippy::too_many_arguments)]
pub fn create_tab(
    notebook: &gtk::Notebook,
    _tabs: &Rc<RefCell<Vec<Tab>>>,
    url: &str,
    window: &ApplicationWindow,
    global_window: &Rc<RefCell<Option<ApplicationWindow>>>,
    web_context: &WebContext,
    banlist: &SharedBanList,
    downloads: &Rc<RefCell<crate::util::downloads::DownloadManager>>,
    expected_unban: &Rc<RefCell<Option<(String, i32)>>>,
    internal_pages: &Rc<Vec<Box<dyn crate::browsing::internal::InternalPage>>>,
    _tx: &async_channel::Sender<(String, bool)>,
    url_entry: &Entry,
    current_uri: &Rc<RefCell<String>>,
    key_button: &Button,
    _is_cleaning: &Rc<RefCell<bool>>,
    _global_webview: &Rc<RefCell<Option<WebView>>>,
    noise_provider: &Rc<RssNoiseProvider>,
    tab_tx: &async_channel::Sender<String>,
) -> Tab {
    let config = AppConfig::load();
    let ucm = UserContentManager::new();
    super::handlers::setup_user_content_manager(&ucm, &config);

    let active_ua = config.active_user_agent();
    let settings = webkit2gtk::Settings::builder()
        .user_agent(&active_ua)
        .build();
    let webview = WebView::builder()
        .web_context(web_context)
        .user_content_manager(&ucm)
        .settings(&settings)
        .build();

    let ad_intox_engine = Rc::new(crate::ad_intoxication::AdIntoxicationEngine::new(
        web_context,
        &webview,
        &config,
    ));

    let window_msg = global_window.clone();
    let ad_engine_msg = ad_intox_engine.clone();
    let banlist_msg = banlist.clone();
    let webview_msg = webview.clone();
    ucm.register_script_message_handler("console");
    ucm.connect_script_message_received(Some("console"), move |_manager, js_result| {
        if let Some(val) = js_result.js_value() {
            let msg = val.to_string();
            log!(raw:Debug, CONSOLE, "{}", msg);
        }
    });

    ucm.connect_script_message_received(Some("juanita"), move |_manager, js_result| {
        crate::browsing::message_handler::handle_script_message(
            js_result,
            &webview_msg,
            &window_msg,
            &banlist_msg,
            &ad_engine_msg,
        );
    });

    use crate::browsing::gui_plugin::{AdIntoxicationPlugin, GuiPlugin};
    use crate::plugins::unsubscribe::AggressiveUnsubscribePlugin;
    let plugin = AdIntoxicationPlugin;
    plugin.setup(window, &webview, &ad_intox_engine);

    let unsub_plugin = AggressiveUnsubscribePlugin;
    unsub_plugin.setup(window, &webview, &ad_intox_engine);

    let intox_engine = IntoxicationEngine::new(web_context, &webview, &config);

    let last_interaction = Rc::new(RefCell::new(std::time::Instant::now()));
    let is_killed = Rc::new(RefCell::new(false));
    let original_title = Rc::new(RefCell::new(String::from("New Tab")));

    let label = gtk::Label::new(Some("New Tab"));

    // Custom EventBox + Box for close buttons and middle click
    let event_box = gtk::EventBox::new();
    let tab_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    tab_box.pack_start(&label, true, true, 0);

    let close_btn = gtk::Button::builder()
        .relief(gtk::ReliefStyle::None)
        .focus_on_click(false)
        .build();
    let close_img = gtk::Image::from_icon_name(Some("window-close-symbolic"), gtk::IconSize::Menu);
    close_btn.set_image(Some(&close_img));
    tab_box.pack_start(&close_btn, false, false, 0);

    event_box.add(&tab_box);
    event_box.show_all();

    let notebook_close = notebook.clone();
    let tabs_close = _tabs.clone();
    let webview_close = webview.clone();
    let is_cleaning_close = _is_cleaning.clone();
    let url_entry_close = url_entry.clone();
    let gw_close = _global_webview.clone();
    let cur_close = current_uri.clone();
    let key_close = key_button.clone();

    close_btn.connect_clicked(move |_| {
        crate::browsing::tab_cleanup::manual_close_tab(
            &notebook_close,
            &tabs_close,
            &webview_close,
            &is_cleaning_close,
            &url_entry_close,
            &gw_close,
            &cur_close,
            &key_close,
        );
    });

    let notebook_mc = notebook.clone();
    let tabs_mc = _tabs.clone();
    let webview_mc = webview.clone();
    let is_cleaning_mc = _is_cleaning.clone();
    let url_entry_mc = url_entry.clone();
    let gw_mc = _global_webview.clone();
    let cur_mc = current_uri.clone();
    let key_mc = key_button.clone();

    event_box.connect_button_press_event(move |_, event| {
        if event.button() == 2 {
            crate::browsing::tab_cleanup::manual_close_tab(
                &notebook_mc,
                &tabs_mc,
                &webview_mc,
                &is_cleaning_mc,
                &url_entry_mc,
                &gw_mc,
                &cur_mc,
                &key_mc,
            );
            gtk::glib::Propagation::Stop
        } else {
            gtk::glib::Propagation::Proceed
        }
    });

    let li_clone = last_interaction.clone();
    webview.connect_button_press_event(move |_, _| {
        *li_clone.borrow_mut() = std::time::Instant::now();
        gtk::glib::Propagation::Proceed
    });

    let li_clone = last_interaction.clone();
    webview.connect_key_press_event(move |_, _| {
        *li_clone.borrow_mut() = std::time::Instant::now();
        gtk::glib::Propagation::Proceed
    });

    let url_entry_nav = url_entry.clone();
    let key_button_clone = key_button.clone();
    let intox_engine_load = intox_engine.clone();
    let current_uri_nav = current_uri.clone();
    let li_clone = last_interaction.clone();
    let is_killed_clone = is_killed.clone();

    webview.connect_load_changed(move |wv, load_event| {
        if *is_killed_clone.borrow() {
            return;
        }
        *li_clone.borrow_mut() = std::time::Instant::now();

        match load_event {
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
        }
    });

    webview.connect_load_failed_with_tls_errors(move |wv, failing_uri, _cert, _errors| {
        let domain = crate::browsing::browser::extract_domain(failing_uri);
        let host = crate::resolver::clean_host(&domain);
        if !host.is_empty() && !crate::resolver::is_system_resolvable(&host) {
            super::handlers::render_tls_error(wv, failing_uri);
            true
        } else {
            false
        }
    });

    webview.connect_load_failed(move |wv, _load_event, failing_uri, error| {
        if let Some(network_error) = error.kind::<webkit2gtk::NetworkError>() {
            if matches!(network_error, webkit2gtk::NetworkError::Cancelled) {
                return false;
            }
        }
        let error_message = error.message();
        let msg_lower = error_message.to_lowercase();
        let is_tls_error = msg_lower.contains("tls")
            || msg_lower.contains("ssl")
            || msg_lower.contains("handshake")
            || msg_lower.contains("certificate");

        if is_tls_error && failing_uri.starts_with("https://") {
            super::handlers::render_tls_error(wv, failing_uri);
            return true;
        }

        super::handlers::render_proxy_error(wv, failing_uri, error_message);
        true
    });

    let webview_create = webview.clone();
    webview.connect_create(move |_wv, nav_action| {
        #[allow(deprecated)]
        if let Some(uri) = nav_action.request().and_then(|r| r.uri()) {
            webview_create.load_uri(uri.as_str());
        }
        None
    });

    let webview_nav = webview.clone();
    let expected_unban_policy = expected_unban.clone();
    let downloads_policy = downloads.clone();
    let banlist_policy = banlist.clone();
    let internal_pages_policy = internal_pages.clone();
    let ad_intox_engine_policy = ad_intox_engine.clone();
    let intox_engine_policy = intox_engine.clone();
    let noise_provider_policy = noise_provider.clone();

    let tab_tx_policy = tab_tx.clone();
    webview.connect_decide_policy(move |_, decision, decision_type| {
        if decision_type == PolicyDecisionType::NavigationAction
            || decision_type == PolicyDecisionType::NewWindowAction
        {
            if let Some(nav_decision) = decision.downcast_ref::<NavigationPolicyDecision>() {
                #[allow(deprecated)]
                if let Some(req) = nav_decision.request() {
                    if let Some(uri) = req.uri() {
                        let uri_str = uri.as_str();
                        if decision_type == PolicyDecisionType::NewWindowAction
                            || nav_decision.mouse_button() == 2
                        {
                            decision.ignore();
                            let _ = tab_tx_policy.send_blocking(uri_str.to_string());
                            return true;
                        }
                        if crate::browsing::policy::handle_decide_policy(
                            decision,
                            uri_str,
                            &webview_nav,
                            &downloads_policy,
                            &banlist_policy,
                            &expected_unban_policy,
                            &internal_pages_policy,
                            &ad_intox_engine_policy,
                            &intox_engine_policy,
                            &noise_provider_policy,
                        ) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    });

    let window_menu = window.clone();
    webview.connect_context_menu(move |_wv, menu, _event, hit_test| {
        use webkit2gtk::{ContextMenuExt, ContextMenuItemExt};
        if let Some(uri) = hit_test.link_uri() {
            for item in &menu.items() {
                if item.stock_action() == webkit2gtk::ContextMenuAction::OpenLinkInNewWindow {
                    menu.remove(item);
                }
            }
            if let Some(open_act) = window_menu.lookup_action("open-in-new-tab") {
                let item = webkit2gtk::ContextMenuItem::from_gaction(
                    &open_act,
                    "Open Link in New Tab",
                    Some(&uri.to_string().to_variant()),
                );
                menu.insert(&item, 0);
            }
        }
        false
    });

    let original_title_clone = original_title.clone();
    let is_killed_clone2 = is_killed.clone();
    let label_clone2 = label.clone();
    webview.connect_title_notify(move |wv| {
        if !*is_killed_clone2.borrow() {
            let title = wv.title().unwrap_or_else(|| "New Tab".into()).to_string();
            *original_title_clone.borrow_mut() = title.clone();
            label_clone2.set_text(&title);
        }
    });

    let ctx = crate::browsing::internal::PageContext {
        webview: webview.clone(),
        downloads: downloads.clone(),
        banlist: banlist.clone(),
        expected_unban: expected_unban.clone(),
        noise_pool: std::rc::Rc::new(std::cell::RefCell::new(crate::privacy::search::gossip::SearchNoisePool::new())),
        config: AppConfig::load(),
    };

    let mut handled = false;
    for page in internal_pages.iter() {
        if page.matches_input(url) {
            page.handle_input(url, &ctx);
            handled = true;
            break;
        }
    }
    if !handled {
        let normalized = crate::browsing::browser::normalize_url(url);
        webview.load_uri(&normalized);
    }

    notebook.append_page(&webview, Some(&event_box));
    webview.show_all();

    Tab {
        webview,
        _label: label,
        last_interaction,
        is_killed,
        _original_title: original_title,
        intox_engine,
        ad_intox_engine,
    }
}
