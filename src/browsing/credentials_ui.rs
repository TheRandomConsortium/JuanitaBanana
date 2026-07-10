/// Credential UI helpers: autofill, save-suggest, and master password prompt.
///
/// All three user-facing flows live here to keep gui.rs under the line limit.
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box as GtkBox, Button, ButtonsType, Entry, Label, MessageDialog,
    MessageType, Orientation, ResponseType,
};
use javascriptcore::ValueExt;
use webkit2gtk::{WebView, WebViewExt};

// ─── Master Password Dialog ──────────────────────────────────────────────────

/// Show a modal dialog asking for the master password.
/// Returns `Some(password)` if the user confirmed, `None` if cancelled.
pub fn ask_master_password(parent: &ApplicationWindow, title: &str, body: &str) -> Option<String> {
    let dialog = MessageDialog::new(
        Some(parent),
        gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
        MessageType::Question,
        ButtonsType::OkCancel,
        title,
    );
    dialog.set_secondary_text(Some(body));

    let content = dialog.content_area();
    content.set_spacing(6);
    let pass_entry = Entry::new();
    pass_entry.set_visibility(false);
    pass_entry.set_placeholder_text(Some("Master password"));
    pass_entry.set_activates_default(true);
    content.pack_end(&pass_entry, false, false, 4);
    content.show_all();

    let result = if dialog.run() == ResponseType::Ok {
        let pass = pass_entry.text().to_string();
        if pass.is_empty() {
            None
        } else {
            Some(pass)
        }
    } else {
        None
    };
    dialog.close();
    result
}

// ─── Autofill ────────────────────────────────────────────────────────────────

/// Called when the user clicks the Key button in the header bar.
/// Prompts for the master password and immediately displays the copy-fill dialog
/// to prevent detection/erasure by pages.
pub fn try_autofill(webview: &WebView, window: &ApplicationWindow) {
    let uri = match webview.uri() {
        Some(u) => u.to_string(),
        None => return,
    };
    if uri.starts_with("juanita://") || uri.starts_with("about:") {
        return;
    }
    let domain = crate::browsing::browser::extract_domain(&uri);
    if domain.is_empty() {
        return;
    }

    let idx = crate::util::credentials::CredentialIndex::load();
    if !idx.has_credentials(&domain) {
        return;
    }

    let title = format!("🔑 Copy-Fill — {}", domain);
    let body = format!(
        "Saved credentials found for {}.\nEnter your master password to view and copy credentials.",
        domain
    );
    let pass = match ask_master_password(window, &title, &body) {
        Some(p) => p,
        None => return,
    };

    let (tx, rx) = async_channel::unbounded::<Option<(String, String, String)>>();
    let win_clone = window.clone();
    gtk::glib::spawn_future_local(async move {
        while let Ok(creds) = rx.recv().await {
            if let Some((username, password, _email)) = creds {
                show_copy_fill_dialog(&win_clone, &username, &password);
            } else {
                show_error(
                    &win_clone,
                    "Failed to decrypt credentials. Wrong master password?",
                );
            }
        }
    });

    let domain_clone = domain.to_string();
    std::thread::spawn(move || {
        let creds = decrypt_and_get(&domain_clone, &pass);
        let _ = tx.send_blocking(creds);
    });
}

fn decrypt_and_get(domain: &str, master_pass: &str) -> Option<(String, String, String)> {
    let mut mgr = crate::unsubscribe::db::SecureDbManager::new(master_pass).ok()?;
    let conn = mgr.open_connection().ok()?;
    let result = crate::unsubscribe::db::get_credentials_for_domain(&conn, domain);
    mgr.save_and_close(conn).ok()?;
    result
}

fn inject_autofill(
    webview: &WebView,
    window: &ApplicationWindow,
    username_or_email: &str,
    username_raw: &str,
    password: &str,
) {
    let u = username_or_email.replace('\\', "\\\\").replace('\'', "\\'");
    let p = password.replace('\\', "\\\\").replace('\'', "\\'");
    let js = include_str!("../../scripts/autofill_injection.js")
        .replace("USERNAME_PLACEHOLDER", &u)
        .replace("PASSWORD_PLACEHOLDER", &p);

    let window_clone = window.clone();
    let u_copy = username_raw.to_string();
    let p_copy = password.to_string();
    webview.evaluate_javascript(
        &js,
        None,
        None,
        gtk::gio::Cancellable::NONE,
        move |result| {
            let filled_str = result
                .ok()
                .map(|v| v.to_string().to_string())
                .unwrap_or_default();
            let filled = filled_str.parse::<i32>().unwrap_or(0);
            if filled == 0 {
                show_copy_fill_dialog(&window_clone, &u_copy, &p_copy);
            }
        },
    );
}

fn show_copy_fill_dialog(window: &ApplicationWindow, username: &str, password: &str) {
    let dialog = gtk::Dialog::with_buttons(
        Some("📋 Copy-fill — autofill blocked"),
        Some(window),
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        &[("Close", ResponseType::Close)],
    );
    dialog.set_modal(false);
    dialog.set_keep_above(true);

    let content = dialog.content_area();
    content.set_margin_start(18);
    content.set_margin_end(18);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_spacing(10);

    let note = Label::new(Some(
        "Autofill was blocked by the page. Copy each field manually:",
    ));
    note.set_halign(gtk::Align::Start);
    content.pack_start(&note, false, false, 0);

    // ── Username row ──
    let u_box = GtkBox::new(Orientation::Horizontal, 6);
    let u_lbl = Label::new(Some("Username:"));
    u_lbl.set_width_chars(10);
    u_lbl.set_halign(gtk::Align::End);
    let u_entry = Entry::new();
    u_entry.set_text(username);
    u_entry.set_editable(false);
    u_entry.set_hexpand(true);
    let u_btn = Button::with_label("Copy");
    let u_val = username.to_string();
    let win_u = window.clone();
    u_btn.connect_clicked(move |_| {
        copy_to_clipboard(&win_u, &u_val);
    });
    u_box.pack_start(&u_lbl, false, false, 0);
    u_box.pack_start(&u_entry, true, true, 0);
    u_box.pack_start(&u_btn, false, false, 0);
    content.pack_start(&u_box, false, false, 0);

    // ── Password row ──
    let p_box = GtkBox::new(Orientation::Horizontal, 6);
    let p_lbl = Label::new(Some("Password:"));
    p_lbl.set_width_chars(10);
    p_lbl.set_halign(gtk::Align::End);
    let p_entry = Entry::new();
    p_entry.set_text(password);
    p_entry.set_editable(false);
    p_entry.set_visibility(false);
    p_entry.set_hexpand(true);
    let p_btn = Button::with_label("Copy");
    let p_val = password.to_string();
    let win_p = window.clone();
    p_btn.connect_clicked(move |_| {
        copy_to_clipboard(&win_p, &p_val);
    });
    // Show/hide toggle
    let p_entry_toggle = p_entry.clone();
    let show_btn = Button::with_label("👁");
    show_btn.connect_clicked(move |_| {
        let is_visible = p_entry_toggle.property::<bool>("visibility");
        p_entry_toggle.set_visibility(!is_visible);
    });
    p_box.pack_start(&p_lbl, false, false, 0);
    p_box.pack_start(&p_entry, true, true, 0);
    p_box.pack_start(&show_btn, false, false, 0);
    p_box.pack_start(&p_btn, false, false, 0);
    content.pack_start(&p_box, false, false, 0);

    dialog.connect_response(|d, _| {
        d.close();
    });

    content.show_all();
    dialog.show();
}

fn copy_to_clipboard(_window: &ApplicationWindow, text: &str) {
    let clipboard = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD);
    clipboard.set_text(text);
}

// ─── Save Suggest ────────────────────────────────────────────────────────────

/// Called when a `credential_capture` message arrives from the page JS.
/// Shows a "Save credentials?" dialog and, if confirmed, persists to the vault.
pub fn handle_save_suggest(
    window: &ApplicationWindow,
    domain: &str,
    username: &str,
    password: &str,
) {
    let domain = domain.to_string();
    let username = username.to_string();
    let password = password.to_string();

    let title = format!("💾 Save credentials — {}", domain);
    let body = format!(
        "Save credentials for {}?\nUsername / email: {}",
        domain, username
    );

    let pass = match ask_master_password(window, &title, &body) {
        Some(p) => p,
        None => return,
    };

    // Derive email heuristic: if username looks like an email, store it as email too
    let email = if username.contains('@') {
        username.clone()
    } else {
        String::new()
    };

    match crate::unsubscribe::db::SecureDbManager::new_responsive(&pass) {
        Ok(mut mgr) => match mgr.open_connection() {
            Ok(conn) => {
                let saved = crate::unsubscribe::db::save_full_credentials(
                    &conn, &domain, &username, &password, &email,
                );
                let _ = mgr.save_and_close(conn);
                match saved {
                    Ok(_) => {
                        let mut idx = crate::util::credentials::CredentialIndex::load();
                        idx.register(&domain);
                        println!("[CREDS] Saved credentials for: {}", domain);
                    }
                    Err(e) => show_error(window, &format!("Failed to save: {}", e)),
                }
            }
            Err(e) => show_error(window, &format!("DB open error: {}", e)),
        },
        Err(_) => show_error(window, "Wrong master password."),
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn show_error(parent: &ApplicationWindow, msg: &str) {
    let d = MessageDialog::new(
        Some(parent),
        gtk::DialogFlags::MODAL,
        MessageType::Error,
        ButtonsType::Ok,
        msg,
    );
    d.run();
    d.close();
}

// ─── Form Monitor JS ─────────────────────────────────────────────────────────

/// JavaScript injected into every page to detect login form submissions
/// and suggest saving credentials.
pub fn form_monitor_script() -> &'static str {
    include_str!("../../scripts/form_monitor.js")
}

/// JavaScript injected to detect when the user interacts (clicks or focuses)
/// with a login form input, triggering interactive autofill.
pub fn form_interact_script() -> &'static str {
    include_str!("../../scripts/form_interact.js")
}

/// Called when the user interacts with a login form on the page.
/// Checks if the master password has already been prompted during this page load.
/// If not, prompts the user and asynchronously decrypts and injects the credentials.
pub fn handle_form_interact(webview: &WebView, window: &ApplicationWindow) {
    let wv = webview.clone();
    let win = window.clone();
    webview.evaluate_javascript(
        "window.__juanita_autofill_prompted",
        None,
        None,
        gtk::gio::Cancellable::NONE,
        move |result| {
            let already_prompted = result.ok()
                .map(|v| v.to_boolean())
                .unwrap_or(false);
            if already_prompted {
                return;
            }

            // Set the prompted flag immediately to avoid race conditions/multiple popups
            wv.evaluate_javascript(
                "window.__juanita_autofill_prompted = true;",
                None,
                None,
                gtk::gio::Cancellable::NONE,
                |_| {}
            );

            let uri = match wv.uri() {
                Some(u) => u.to_string(),
                None => return,
            };
            let domain = crate::browsing::browser::extract_domain(&uri);
            if domain.is_empty() {
                return;
            }

            let idx = crate::util::credentials::CredentialIndex::load();
            if !idx.has_credentials(&domain) {
                return;
            }

            let title = format!("🔑 Autofill — {}", domain);
            let body = format!(
                "Saved credentials found for {}.\nEnter your master password to fill the login form.",
                domain
            );
            let pass = match ask_master_password(&win, &title, &body) {
                Some(p) => p,
                None => return, // If they cancel, prompted flag is already true, so they won't be asked again
            };

            let (tx, rx) = async_channel::unbounded::<Option<(String, String, String)>>();
            let wv_clone = wv.clone();
            let win_clone = win.clone();
            gtk::glib::spawn_future_local(async move {
                while let Ok(creds) = rx.recv().await {
                    if let Some((username, password, email)) = creds {
                        let fill_value = if !email.is_empty() {
                            email
                        } else {
                            username.clone()
                        };
                        inject_autofill(&wv_clone, &win_clone, &fill_value, &username, &password);
                    } else {
                        // If wrong password, allow retry by resetting the prompted flag
                        wv_clone.evaluate_javascript(
                            "window.__juanita_autofill_prompted = false;",
                            None,
                            None,
                            gtk::gio::Cancellable::NONE,
                            |_| {}
                        );
                        show_error(&win_clone, "Autofill failed. Wrong master password?");
                    }
                }
            });

            let domain_clone = domain.to_string();
            std::thread::spawn(move || {
                let creds = decrypt_and_get(&domain_clone, &pass);
                let _ = tx.send_blocking(creds);
            });
        }
    );
}
