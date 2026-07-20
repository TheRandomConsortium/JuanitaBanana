use crate::browsing::tab::Tab;
use gtk::prelude::*;
use gtk::{Button, Entry, Notebook};
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{WebView, WebViewExt};

pub fn sync_active_tab_ui(
    notebook: &Notebook,
    tabs: &[Tab],
    url_entry: &Entry,
    global_webview: &Rc<RefCell<Option<WebView>>>,
    current_uri: &Rc<RefCell<String>>,
    key_button: &Button,
) {
    if notebook.in_destruction() || url_entry.in_destruction() || key_button.in_destruction() {
        return;
    }
    if let Some(page_num) = notebook.current_page() {
        if let Some(tab) = tabs.get(page_num as usize) {
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

pub fn cleanup_killed_tabs(
    notebook: &Notebook,
    tabs: &Rc<RefCell<Vec<Tab>>>,
    url_entry: &Entry,
    global_webview: &Rc<RefCell<Option<WebView>>>,
    current_uri: &Rc<RefCell<String>>,
    key_button: &Button,
    is_cleaning: &Rc<RefCell<bool>>,
) {
    if notebook.in_destruction() || url_entry.in_destruction() || key_button.in_destruction() {
        return;
    }
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

    {
        let mut tabs_borrow = tabs.borrow_mut();
        for idx in tabs_to_remove {
            notebook.remove_page(Some(idx as u32));
            tabs_borrow.remove(idx);
        }
    }

    *is_cleaning.borrow_mut() = false;

    // Refresh active tab UI state
    let tabs_borrow = tabs.borrow();
    sync_active_tab_ui(
        notebook,
        &tabs_borrow,
        url_entry,
        global_webview,
        current_uri,
        key_button,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn manual_close_tab(
    notebook: &Notebook,
    tabs: &Rc<RefCell<Vec<Tab>>>,
    webview: &WebView,
    is_cleaning: &Rc<RefCell<bool>>,
    url_entry: &Entry,
    global_webview: &Rc<RefCell<Option<WebView>>>,
    current_uri: &Rc<RefCell<String>>,
    key_button: &Button,
) {
    if notebook.in_destruction() || url_entry.in_destruction() || key_button.in_destruction() {
        return;
    }
    if *is_cleaning.borrow() {
        return;
    }
    *is_cleaning.borrow_mut() = true;

    let mut idx_to_remove = None;
    {
        let tabs_borrow = tabs.borrow();
        for (i, tab) in tabs_borrow.iter().enumerate() {
            if tab.webview == *webview {
                idx_to_remove = Some(i);
                break;
            }
        }
    }

    if let Some(idx) = idx_to_remove {
        notebook.remove_page(Some(idx as u32));
        tabs.borrow_mut().remove(idx);
    }

    *is_cleaning.borrow_mut() = false;

    // Refresh active tab UI state
    let tabs_borrow = tabs.borrow();
    sync_active_tab_ui(
        notebook,
        &tabs_borrow,
        url_entry,
        global_webview,
        current_uri,
        key_button,
    );
}

pub fn check_tab_inactivity(tabs: &Rc<RefCell<Vec<Tab>>>) {
    let config = crate::util::config::AppConfig::load();
    let ttl_mins = config.tab_inactivity_ttl.clamp(1, 60);
    let nuke_action = config.last_tab_nuke_action.clone();

    let now = std::time::Instant::now();
    let mut tabs_borrow = tabs.borrow_mut();

    let mut active_count = tabs_borrow
        .iter()
        .filter(|t| !*t.is_killed.borrow())
        .count();

    for tab in tabs_borrow.iter_mut() {
        if *tab.is_killed.borrow() {
            continue;
        }

        let last_inter = *tab.last_interaction.borrow();
        if now.duration_since(last_inter) >= std::time::Duration::from_secs(ttl_mins as u64 * 60) {
            if active_count == 1 {
                if nuke_action == "survive" {
                    *tab.last_interaction.borrow_mut() = now;
                    continue;
                } else if nuke_action == "home" {
                    tab.webview.load_uri("juanita://home");
                    *tab.last_interaction.borrow_mut() = now;
                    continue;
                }
            }

            *tab.is_killed.borrow_mut() = true;
            active_count -= 1;

            let shared_css = crate::browsing::internal::SHARED_CSS;
            let cleanup_html = format!(
                r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8">
<style>
  {shared_css}
</style>
</head>
<body class="jb-page">
<div class="jb-container">
  <div class="jb-subtitle" style="font-size: 1.1em; color: var(--jb-text-secondary);">tab closed due to inactivity in 0 frames</div>
</div></body></html>"#,
                shared_css = shared_css
            );
            tab.webview.load_html(&cleanup_html, Some("about:blank"));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn setup_cleanup_triggers(
    window: &gtk::ApplicationWindow,
    notebook: &Notebook,
    tabs: &Rc<RefCell<Vec<Tab>>>,
    url_entry: &Entry,
    global_webview: &Rc<RefCell<Option<WebView>>>,
    current_uri: &Rc<RefCell<String>>,
    key_button: &Button,
    is_cleaning: &Rc<RefCell<bool>>,
) {
    // 1. url_entry Focus In
    let tabs_c = tabs.clone();
    let notebook_c = notebook.clone();
    let url_entry_c = url_entry.clone();
    let gw_c = global_webview.clone();
    let cur_c = current_uri.clone();
    let key_c = key_button.clone();
    let is_clean_c = is_cleaning.clone();
    url_entry.connect_focus_in_event(move |entry, _| {
        cleanup_killed_tabs(
            &notebook_c,
            &tabs_c,
            &url_entry_c,
            &gw_c,
            &cur_c,
            &key_c,
            &is_clean_c,
        );
        let uri = cur_c.borrow();
        entry.set_text(&crate::util::debug::redact_uri(&uri));
        gtk::glib::Propagation::Proceed
    });

    // 2. url_entry Focus Out
    let cur_blur = current_uri.clone();
    url_entry.connect_focus_out_event(move |entry, _| {
        let uri = cur_blur.borrow();
        let display_uri = if let Some((base, _)) = uri.split_once('?') {
            base.to_string()
        } else {
            uri.to_string()
        };
        entry.set_text(&display_uri);
        gtk::glib::Propagation::Proceed
    });

    // 3. notebook Button Press
    let tabs_c = tabs.clone();
    let notebook_c = notebook.clone();
    let url_entry_c = url_entry.clone();
    let gw_c = global_webview.clone();
    let cur_c = current_uri.clone();
    let key_c = key_button.clone();
    let is_clean_c = is_cleaning.clone();
    notebook.connect_button_press_event(move |_, _| {
        cleanup_killed_tabs(
            &notebook_c,
            &tabs_c,
            &url_entry_c,
            &gw_c,
            &cur_c,
            &key_c,
            &is_clean_c,
        );
        gtk::glib::Propagation::Proceed
    });

    // 4. notebook Enter Notify
    let tabs_c = tabs.clone();
    let notebook_c = notebook.clone();
    let url_entry_c = url_entry.clone();
    let gw_c = global_webview.clone();
    let cur_c = current_uri.clone();
    let key_c = key_button.clone();
    let is_clean_c = is_cleaning.clone();
    notebook.connect_enter_notify_event(move |_, _| {
        cleanup_killed_tabs(
            &notebook_c,
            &tabs_c,
            &url_entry_c,
            &gw_c,
            &cur_c,
            &key_c,
            &is_clean_c,
        );
        gtk::glib::Propagation::Proceed
    });

    // 5. window Focus In
    let tabs_c = tabs.clone();
    let notebook_c = notebook.clone();
    let url_entry_c = url_entry.clone();
    let gw_c = global_webview.clone();
    let cur_c = current_uri.clone();
    let key_c = key_button.clone();
    let is_clean_c = is_cleaning.clone();
    window.connect_focus_in_event(move |_, _| {
        cleanup_killed_tabs(
            &notebook_c,
            &tabs_c,
            &url_entry_c,
            &gw_c,
            &cur_c,
            &key_c,
            &is_clean_c,
        );
        gtk::glib::Propagation::Proceed
    });
}
