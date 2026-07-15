use gtk::prelude::*;
use gtk::{ApplicationWindow, Button, Entry};
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

#[allow(dead_code)]
#[derive(Clone)]
pub struct Tab {
    pub webview: WebView,
    pub label: gtk::Label,
    pub last_interaction: Rc<RefCell<std::time::Instant>>,
    pub is_killed: Rc<RefCell<bool>>,
    pub original_title: Rc<RefCell<String>>,
    pub intox_engine: IntoxicationEngine,
    pub ad_intox_engine: Rc<crate::ad_intoxication::AdIntoxicationEngine>,
}

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
) -> Tab {
    let config = AppConfig::load();
    let ucm = UserContentManager::new();

    let script = UserScript::new(
        spoof::anti_fingerprint_script(),
        UserContentInjectedFrames::AllFrames,
        UserScriptInjectionTime::Start,
        &[],
        &["juanita://*"],
    );
    ucm.add_script(&script);

    ucm.register_script_message_handler("juanita");
    let ad_script = UserScript::new(
        &crate::ad_intoxication::ad_intoxication_script(&config),
        UserContentInjectedFrames::AllFrames,
        UserScriptInjectionTime::Start,
        &[],
        &["juanita://*"],
    );
    ucm.add_script(&ad_script);

    let toxic_script = UserScript::new(
        &crate::util::ban::toxic_warning_script(&config),
        UserContentInjectedFrames::TopFrame,
        UserScriptInjectionTime::Start,
        &[],
        &["juanita://*"],
    );
    ucm.add_script(&toxic_script);

    if config.guilt_trip_enabled {
        let guilt_script = UserScript::new(
            &crate::browsing::guilt::guilt_trip_script(&config),
            UserContentInjectedFrames::TopFrame,
            UserScriptInjectionTime::Start,
            &[],
            &["juanita://*"],
        );
        ucm.add_script(&guilt_script);
    }

    let form_mon_script = UserScript::new(
        crate::browsing::credentials_ui::form_monitor_script(),
        UserContentInjectedFrames::TopFrame,
        UserScriptInjectionTime::End,
        &[],
        &[],
    );
    ucm.add_script(&form_mon_script);

    let console_override = UserScript::new(
        crate::util::debug::console_override_script(),
        UserContentInjectedFrames::AllFrames,
        UserScriptInjectionTime::Start,
        &[],
        &[],
    );
    ucm.add_script(&console_override);

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

    let noise_provider = Rc::new(RssNoiseProvider::new(&config));
    let intox_engine = IntoxicationEngine::new(web_context, &webview, &config);

    let last_interaction = Rc::new(RefCell::new(std::time::Instant::now()));
    let is_killed = Rc::new(RefCell::new(false));
    let original_title = Rc::new(RefCell::new(String::from("New Tab")));

    let label = gtk::Label::new(Some("New Tab"));

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
    let label_clone = label.clone();

    webview.connect_load_changed(move |wv, load_event| {
        if *is_killed_clone.borrow() {
            if matches!(load_event, webkit2gtk::LoadEvent::Started) {
                *is_killed_clone.borrow_mut() = false;
                label_clone.set_text("Loading...");
            } else {
                return;
            }
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

    let webview_create = webview.clone();
    webview.connect_create(move |_wv, nav_action| {
        #[allow(deprecated)]
        if let Some(req) = nav_action.request() {
            if let Some(uri) = req.uri() {
                webview_create.load_uri(uri.as_str());
            }
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
                            &intox_engine_policy,
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

    let original_title_clone = original_title.clone();
    let is_killed_clone2 = is_killed.clone();
    let label_clone2 = label.clone();
    webview.connect_title_notify(move |wv| {
        if *is_killed_clone2.borrow() {
            return;
        }
        let title = wv.title().unwrap_or_else(|| "New Tab".into()).to_string();
        *original_title_clone.borrow_mut() = title.clone();
        label_clone2.set_text(&title);
    });

    let ctx = crate::browsing::internal::PageContext {
        webview: webview.clone(),
        downloads: downloads.clone(),
        banlist: banlist.clone(),
        expected_unban: expected_unban.clone(),
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

    notebook.append_page(&webview, Some(&label));
    webview.show_all();
    label.show_all();

    Tab {
        webview,
        label,
        last_interaction,
        is_killed,
        original_title,
        intox_engine,
        ad_intox_engine,
    }
}

pub fn cleanup_killed_tabs(
    notebook: &gtk::Notebook,
    tabs: &Rc<RefCell<Vec<Tab>>>,
    url_entry: &Entry,
    global_webview: &Rc<RefCell<Option<WebView>>>,
    current_uri: &Rc<RefCell<String>>,
    key_button: &Button,
    _config: &AppConfig,
    is_cleaning: &Rc<RefCell<bool>>,
) {
    if *is_cleaning.borrow() {
        return;
    }

    let mut tabs_to_remove = Vec::new();
    {
        let tabs_borrow = tabs.borrow();
        for (i, tab) in tabs_borrow.iter().enumerate() {
            if *tab.is_killed.borrow() {
                tabs_to_remove.push(i);
            }
        }
    }

    if tabs_to_remove.is_empty() {
        return;
    }

    *is_cleaning.borrow_mut() = true;

    tabs_to_remove.sort_by(|a, b| b.cmp(a));

    let mut tabs_borrow = tabs.borrow_mut();
    for idx in tabs_to_remove {
        notebook.remove_page(Some(idx as u32));
        tabs_borrow.remove(idx);
    }

    *is_cleaning.borrow_mut() = false;

    // Refresh active tab UI state
    if let Some(page_num) = notebook.current_page() {
        if let Some(tab) = tabs_borrow.get(page_num as usize) {
            let uri = tab.webview.uri().map(|s| s.to_string()).unwrap_or_default();
            let restored_uri = crate::resolver::restore_original_domain_in_uri(&uri);
            *current_uri.borrow_mut() = restored_uri.clone();
            let display_uri = if let Some((base, _)) = restored_uri.split_once('?') {
                base.to_string()
            } else {
                restored_uri
            };
            url_entry.set_text(&display_uri);
            *global_webview.borrow_mut() = Some(tab.webview.clone());

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
            key_button.set_visible(has_creds);
        }
    }
}
