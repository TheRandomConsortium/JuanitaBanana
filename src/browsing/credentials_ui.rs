/// Credential UI helpers: autofill, save-suggest, and master password prompt.
///
/// All three user-facing flows live here to keep gui.rs under the line limit.
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box as GtkBox, Button, ButtonsType, Entry, Label, MessageDialog,
    MessageType, Orientation, ResponseType,
};
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

/// Called on `LoadEvent::Finished`. If the credential index has an entry for
/// the current domain, asks for the master password and injects credentials.
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

    let title = format!("🔑 Autofill — {}", domain);
    let body = format!(
        "Saved credentials found for {}.\nEnter your master password to fill the login form.",
        domain
    );
    let pass = match ask_master_password(window, &title, &body) {
        Some(p) => p,
        None => return,
    };

    let creds = decrypt_and_get(&domain, &pass);
    if let Some((username, password, email)) = creds {
        let fill_value = if !email.is_empty() {
            email
        } else {
            username.clone()
        };
        inject_autofill(webview, window, &fill_value, &username, &password);
    } else {
        show_error(window, "Autofill failed. Wrong master password?");
    }
}

fn decrypt_and_get(domain: &str, master_pass: &str) -> Option<(String, String, String)> {
    let mut mgr = crate::unsubscribe::db::SecureDbManager::new_responsive(master_pass).ok()?;
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
    let js = format!(
        r#"(function(){{
  var filled=0;
  var uCandidates=document.querySelectorAll(
    'input[type=email],input[autocomplete=email],input[autocomplete=username],'
    +'input[type=text][name*=email i],input[type=text][name*=user i],'
    +'input[type=text][name*=login i],input[type=text][id*=email i],'
    +'input[type=text][id*=user i],input[type=text][id*=login i]'
  );
  if(uCandidates.length>0){{
    uCandidates[0].value='{u}';
    uCandidates[0].dispatchEvent(new Event('input',{{bubbles:true}}));
    uCandidates[0].dispatchEvent(new Event('change',{{bubbles:true}}));
    filled++;
  }}
  var pCandidates=document.querySelectorAll('input[type=password]');
  if(pCandidates.length>0){{
    pCandidates[0].value='{p}';
    pCandidates[0].dispatchEvent(new Event('input',{{bubbles:true}}));
    pCandidates[0].dispatchEvent(new Event('change',{{bubbles:true}}));
    filled++;
  }}
  return filled;
}})();"#,
        u = u,
        p = p
    );

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
    r#"(function(){
  'use strict';
  if(window.__juanita_form_monitor)return;
  window.__juanita_form_monitor=true;

  function capture(form){
    var passEl=form.querySelector('input[type=password]');
    if(!passEl||!passEl.value)return;
    var uEl=form.querySelector(
      'input[type=email],input[autocomplete=email],input[autocomplete=username],'
      +'input[type=text][name*=email i],input[type=text][name*=user i],'
      +'input[type=text][name*=login i],input[type=text][id*=email i],'
      +'input[type=text][id*=user i],input[type=text][id*=login i]'
    );
    var username=uEl?uEl.value:'';
    if(!username&&!passEl.value)return;
    if(window.webkit&&window.webkit.messageHandlers&&window.webkit.messageHandlers.juanita){
      window.webkit.messageHandlers.juanita.postMessage(JSON.stringify({
        type:'credential_capture',
        domain:window.location.hostname,
        username:username,
        password:passEl.value
      }));
    }
  }

  document.addEventListener('submit',function(e){capture(e.target);},true);
})();"#
}
