use crate::browsing::internal::{InternalPage, PageContext};
use lazy_static::lazy_static;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Mutex;
use webkit2gtk::WebViewExt;

pub struct PasswordsPage;

const SHARED_CSS: &str = crate::browsing::internal::SHARED_CSS;
const LOCKED_HTML_TEMPLATE: &str = include_str!("../../../../templates/passwords/locked.html");

// ── In-memory session store ─────────────────────────────────────────────────
// Maps opaque random token → master password.  The token travels in URLs;
// the master password never appears in a URL, log, or browser history.
lazy_static! {
    static ref SESSION: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

fn new_session(master_pass: &str) -> String {
    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    if let Ok(mut map) = SESSION.lock() {
        // Only one active session at a time
        map.clear();
        map.insert(token.clone(), master_pass.to_string());
    }
    token
}

fn resolve_session(token: &str) -> Option<String> {
    SESSION.lock().ok().and_then(|map| map.get(token).cloned())
}

pub fn clear_session() {
    if let Ok(mut map) = SESSION.lock() {
        map.clear();
    }
}

// ── HTML helpers ────────────────────────────────────────────────────────────

fn render_vault(
    creds: &[(String, String, String, String)],
    error: Option<&str>,
    session_token: &str,
) -> String {
    let rows: String = if creds.is_empty() {
        "<tr><td colspan='5' style='text-align:center;color:var(--jb-text-muted);padding:24px'>No saved credentials yet.</td></tr>".to_string()
    } else {
        creds
            .iter()
            .enumerate()
            .map(|(i, (domain, user, password, email))| {
                let domain_esc = html_escape(domain);
                let user_esc = html_escape(user);
                let email_esc = html_escape(email);
                let password_esc = html_escape(password);
                format!(
                    "<tr id='row-{i}'>\
                  <td>{domain_esc}</td>\
                  <td>{user_esc}</td>\
                  <td>{email_esc}</td>\
                  <td>\
                    <div style='display:flex; align-items:center; gap:8px;'>\
                      <input type='password' id='pass-field-{i}' value='{password_esc}' readonly \
                             style='background:transparent; border:none; color:var(--jb-text-primary); font-family:var(--jb-font-family-mono); width:120px; outline:none;'>\
                      <button class='eye-btn' onclick='togglePass({i})' style='background:transparent; border:none; cursor:pointer; color:var(--jb-text-secondary); font-size:1.1rem; padding:0 4px;'>👁️</button>\
                    </div>\
                  </td>\
                  <td>\
                    <div style='display:flex; gap:6px;'>\
                      <button class='edit-btn jb-btn-outline' style='padding:4px 10px; font-size:0.8rem;' data-domain='{domain_esc}' data-user='{user_esc}' data-email='{email_esc}' data-pass='{password_esc}' onclick='editRow(this)'>✏️</button>\
                      <button class='del-btn jb-btn-danger' style='padding:4px 10px; font-size:0.8rem;' onclick='deleteRow(\"{domain_esc}\")'>✕</button>\
                    </div>\
                  </td>\
                </tr>",
                    i = i,
                    domain_esc = domain_esc,
                    user_esc = user_esc,
                    email_esc = email_esc,
                    password_esc = password_esc
                )
            })
            .collect()
    };

    let error_html = match error {
        Some(e) => format!("<div class='jb-card-alert' style='max-width:100%;'>⚠️ {}</div>", html_escape(e)),
        None => String::new(),
    };

    include_str!("../../../../templates/passwords/vault.html")
        .replace("{shared_css}", SHARED_CSS)
        .replace("{rows}", &rows)
        .replace("{error_html}", &error_html)
        .replace("{unlock_pass_esc}", session_token)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn parse_query(uri: &str, key: &str) -> Option<String> {
    let query = uri.split('?').nth(1)?;
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if kv.next()? == key {
            let val = kv.next().unwrap_or("");
            return Some(urlencoding::decode(val).unwrap_or_default().into_owned());
        }
    }
    None
}

impl InternalPage for PasswordsPage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:passwords") || input.starts_with("juanita://passwords")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        uri.starts_with("juanita://passwords")
            && !uri.starts_with("juanita://passwords-page")
            && !uri.starts_with("juanita://passwords-unlocking")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool {
        let uri_clone = uri.to_string();
        let webview_clone = ctx.webview.clone();

        // ── Determine what kind of request this is ───────────────────────────
        let maybe_token = parse_query(&uri_clone, "session");
        let maybe_unlock = parse_query(&uri_clone, "unlock_pass");

        match (maybe_unlock, maybe_token) {
            // ── No credentials at all → show lock screen ─────────────────────
            (None, None) => {
                clear_session();
                let wv = webview_clone.clone();
                let locked_html = LOCKED_HTML_TEMPLATE.replace("{shared_css}", SHARED_CSS);
                gtk::glib::idle_add_local(move || {
                    wv.load_html(&locked_html, Some("juanita://passwords-page"));
                    gtk::glib::ControlFlow::Break
                });
            }

            // ── Fresh unlock attempt: validate master password ────────────────
            (Some(raw_pass), _) => {
                let unlocking_html = include_str!("../../../../templates/passwords/unlocking.html")
                    .replace("{shared_css}", SHARED_CSS);
                let wv_unlocking = webview_clone.clone();
                gtk::glib::idle_add_local(move || {
                    wv_unlocking.load_html(&unlocking_html, Some("juanita://passwords-unlocking"));
                    gtk::glib::ControlFlow::Break
                });

                enum PasswordResult {
                    Redirect(String),
                    Html(String),
                }

                let (tx, rx) = async_channel::unbounded::<PasswordResult>();
                let wv = webview_clone.clone();
                gtk::glib::spawn_future_local(async move {
                    while let Ok(res) = rx.recv().await {
                        match res {
                            PasswordResult::Redirect(uri) => {
                                wv.load_uri(&uri);
                            }
                            PasswordResult::Html(html) => {
                                wv.load_html(&html, Some("juanita://passwords-page"));
                            }
                        }
                    }
                });

                std::thread::spawn(move || {
                    // Try to open the DB – this validates the master password
                    match crate::unsubscribe::db::SecureDbManager::new(&raw_pass) {
                        Ok(mut mgr) => match mgr.open_connection() {
                            Ok(conn) => {
                                // Handle add/update form submission
                                if let (Some(domain), Some(username), Some(password)) = (
                                    parse_query(&uri_clone, "add_domain"),
                                    parse_query(&uri_clone, "add_username"),
                                    parse_query(&uri_clone, "add_password"),
                                ) {
                                    let email =
                                        parse_query(&uri_clone, "add_email").unwrap_or_default();
                                    let _ = crate::unsubscribe::db::save_full_credentials(
                                        &conn, &domain, &username, &password, &email,
                                    );
                                    let _ = mgr.save_and_close(conn);
                                    let mut idx = crate::util::credentials::CredentialIndex::load();
                                    idx.register(&domain);
                                } else {
                                    let _ = mgr.save_and_close(conn);
                                }

                                // Issue a session token – password leaves the URL here
                                let token = new_session(&raw_pass);
                                let redirect = format!(
                                    "juanita://passwords?session={}",
                                    urlencoding::encode(&token)
                                );
                                let _ = tx.send_blocking(PasswordResult::Redirect(redirect));
                            }
                            Err(e) => {
                                let html = render_vault(&[], Some(&format!("DB error: {}", e)), "");
                                let _ = tx.send_blocking(PasswordResult::Html(html));
                            }
                        },
                        Err(_) => {
                            // Wrong master password
                            let locked_base = LOCKED_HTML_TEMPLATE.replace("{shared_css}", SHARED_CSS);
                            let locked = format!(
                                "{}<script>document.querySelector('p').textContent='Wrong master password. Try again.';</script>",
                                locked_base
                            );
                            let _ = tx.send_blocking(PasswordResult::Html(locked));
                        }
                    }
                });
            }

            // ── Returning visit with a valid session token ────────────────────
            (None, Some(token)) => {
                match resolve_session(&token) {
                    None => {
                        // Session expired / invalid – back to lock screen
                        let wv = webview_clone.clone();
                        let locked = LOCKED_HTML_TEMPLATE.replace("{shared_css}", SHARED_CSS);
                        gtk::glib::idle_add_local(move || {
                            wv.load_html(&locked, Some("juanita://passwords-page"));
                            gtk::glib::ControlFlow::Break
                        });
                    }
                    Some(master_pass) => {
                        let unlocking_html =
                            include_str!("../../../../templates/passwords/unlocking.html")
                                .replace("{shared_css}", SHARED_CSS);
                        let wv_unlocking = webview_clone.clone();
                        gtk::glib::idle_add_local(move || {
                            wv_unlocking
                                .load_html(&unlocking_html, Some("juanita://passwords-unlocking"));
                            gtk::glib::ControlFlow::Break
                        });

                        enum PasswordResult {
                            Redirect(String),
                            Html(String),
                        }

                        let (tx, rx) = async_channel::unbounded::<PasswordResult>();
                        let wv = webview_clone.clone();
                        gtk::glib::spawn_future_local(async move {
                            while let Ok(res) = rx.recv().await {
                                match res {
                                    PasswordResult::Redirect(uri) => {
                                        wv.load_uri(&uri);
                                    }
                                    PasswordResult::Html(html) => {
                                        wv.load_html(&html, Some("juanita://passwords-page"));
                                    }
                                }
                            }
                        });

                        let token_clone = token.clone();
                        std::thread::spawn(move || {
                            // Handle add/update form submission via session
                            if let (Some(domain), Some(username), Some(password)) = (
                                parse_query(&uri_clone, "add_domain"),
                                parse_query(&uri_clone, "add_username"),
                                parse_query(&uri_clone, "add_password"),
                            ) {
                                let email =
                                    parse_query(&uri_clone, "add_email").unwrap_or_default();
                                if let Ok(mut mgr) =
                                    crate::unsubscribe::db::SecureDbManager::new(&master_pass)
                                {
                                    if let Ok(conn) = mgr.open_connection() {
                                        let _ = crate::unsubscribe::db::save_full_credentials(
                                            &conn, &domain, &username, &password, &email,
                                        );
                                        let _ = mgr.save_and_close(conn);
                                        let mut idx =
                                            crate::util::credentials::CredentialIndex::load();
                                        idx.register(&domain);
                                    }
                                }
                                let redirect = format!(
                                    "juanita://passwords?session={}",
                                    urlencoding::encode(&token_clone)
                                );
                                let _ = tx.send_blocking(PasswordResult::Redirect(redirect));
                                return;
                            }

                            // Load and render vault
                            let html = match crate::unsubscribe::db::SecureDbManager::new(
                                &master_pass,
                            ) {
                                Ok(mut mgr) => match mgr.open_connection() {
                                    Ok(conn) => {
                                        let creds =
                                            crate::unsubscribe::db::list_all_credentials(&conn);
                                        let _ = mgr.save_and_close(conn);
                                        render_vault(&creds, None, &token_clone)
                                    }
                                    Err(e) => render_vault(
                                        &[],
                                        Some(&format!("DB error: {}", e)),
                                        &token_clone,
                                    ),
                                },
                                Err(_) => {
                                    // Session token exists but password no longer works –
                                    // wipe session and go back to lock screen
                                    clear_session();
                                    let locked_base = LOCKED_HTML_TEMPLATE.replace("{shared_css}", SHARED_CSS);
                                    format!(
                                        "{}<script>document.querySelector('p').textContent='Session expired. Please unlock again.';</script>",
                                        locked_base
                                    )
                                }
                            };
                            let _ = tx.send_blocking(PasswordResult::Html(html));
                        });
                    }
                }
            }
        }
        true
    }
}
