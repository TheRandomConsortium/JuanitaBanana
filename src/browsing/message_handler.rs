use crate::ad_intoxication::AdIntoxicationEngine;
use crate::browsing::browser::SharedBanList;
use gtk::ApplicationWindow;
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{JavascriptResult, WebView, WebViewExt};

pub fn handle_script_message(
    js_result: &JavascriptResult,
    webview: &WebView,
    window: &Rc<RefCell<Option<ApplicationWindow>>>,
    banlist: &SharedBanList,
    ad_engine: &AdIntoxicationEngine,
) {
    if let Some(val) = js_result.js_value() {
        let json_str = val.to_string();
        if let Ok(msg_val) = serde_json::from_str::<serde_json::Value>(&json_str) {
            if msg_val["type"] == "ad_detected" {
                let page_url = msg_val["page_url"].as_str().unwrap_or("").to_string();
                let selector = msg_val["selector"].as_str().unwrap_or("").to_string();
                let ad_url = msg_val["ad_url"].as_str().unwrap_or("").to_string();
                ad_engine.queue_ad(crate::ad_intoxication::AdTask {
                    page_url,
                    selector,
                    ad_url,
                });
            } else if msg_val["type"] == "right_click_target" {
                if let Ok(info) = serde_json::from_value::<
                    crate::browsing::gui_plugin::RightClickInfo,
                >(msg_val.clone())
                {
                    crate::browsing::gui_plugin::LAST_RIGHT_CLICK.with(|rc| {
                        *rc.borrow_mut() = Some(info);
                    });
                }
            } else if msg_val["type"] == "ban_domain" {
                if let Some(domain) = msg_val["domain"].as_str() {
                    {
                        let mut bl = banlist.borrow_mut();
                        bl.ban(domain);
                        bl.save();
                    }
                    println!("[BAN] Banned domain: {}", domain);
                    let banned_html = crate::util::ban::banned_page(&format!("https://{}", domain));
                    webview.load_html(&banned_html, Some("juanita://banned/"));
                }
            } else if msg_val["type"] == "save_local_html" {
                if let (Some(uri), Some(content)) =
                    (msg_val["uri"].as_str(), msg_val["content"].as_str())
                {
                    let path = uri.strip_prefix("file://").unwrap_or(uri);
                    match std::fs::write(path, content) {
                        Ok(_) => println!("[HTML_VIEWER] Saved: {}", path),
                        Err(e) => println!("[HTML_VIEWER] Save error: {}", e),
                    }
                }
            } else if msg_val["type"] == "form_interact" {
                if let Some(ref win) = *window.borrow() {
                    crate::browsing::credentials_ui::handle_form_interact(webview, win);
                }
            } else if msg_val["type"] == "credential_capture" {
                if let (Some(domain), Some(username), Some(password)) = (
                    msg_val["domain"].as_str(),
                    msg_val["username"].as_str(),
                    msg_val["password"].as_str(),
                ) {
                    let idx = crate::util::credentials::CredentialIndex::load();
                    if !idx.has_credentials(domain) {
                        if let Some(ref win) = *window.borrow() {
                            crate::browsing::credentials_ui::handle_save_suggest(
                                win, domain, username, password,
                            );
                        }
                    }
                }
            } else if msg_val["type"] == "delete_credential" {
                if let Some(domain) = msg_val["domain"].as_str() {
                    if let Some(ref win) = *window.borrow() {
                        let title = format!("❌ Delete Credentials — {}", domain);
                        let body = format!(
                            "Enter your master password to confirm deleting saved credentials for {}.",
                            domain
                        );
                        if let Some(pass) =
                            crate::browsing::credentials_ui::ask_master_password(win, &title, &body)
                        {
                            match crate::unsubscribe::db::SecureDbManager::new_responsive(&pass) {
                                Ok(mut mgr) => {
                                    match mgr.open_connection() {
                                        Ok(conn) => {
                                            let deleted = crate::unsubscribe::db::delete_credentials_for_domain(&conn, domain);
                                            let _ = mgr.save_and_close(conn);
                                            match deleted {
                                                Ok(_) => {
                                                    let mut idx = crate::util::credentials::CredentialIndex::load();
                                                    idx.remove(domain);
                                                    println!(
                                                        "[CREDS] Deleted credentials for: {}",
                                                        domain
                                                    );
                                                }
                                                Err(e) => {
                                                    println!("[CREDS] Failed to delete: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => println!("[CREDS] DB open error: {}", e),
                                    }
                                }
                                Err(_) => println!("[CREDS] Wrong master password."),
                            }
                        }
                    }
                }
            }
        }
    }
}
