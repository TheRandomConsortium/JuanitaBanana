use crate::browsing::internal::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct PasswordsPage;

const LOCKED_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Passwords — Juanita Banana</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{background:#0d0d0d;color:#e0e0e0;font-family:system-ui,-apple-system,sans-serif;
  display:flex;align-items:center;justify-content:center;height:100vh}
.card{background:#1a1a1a;border:1px solid #2a2a2a;border-radius:12px;
  padding:40px 48px;max-width:380px;width:100%;text-align:center}
.icon{font-size:3rem;margin-bottom:16px}
h1{font-size:1.3rem;font-weight:700;margin-bottom:8px}
p{font-size:.875rem;color:#888;margin-bottom:24px}
input{width:100%;background:#111;border:1px solid #333;border-radius:8px;
  padding:10px 14px;color:#e0e0e0;font-size:.9rem;outline:none;margin-bottom:12px}
input:focus{border-color:#3b82f6}
button{width:100%;background:#3b82f6;color:#fff;border:none;border-radius:8px;
  padding:10px;font-size:.9rem;font-weight:700;cursor:pointer}
button:hover{background:#2563eb}
</style>
</head>
<body>
<div class="card">
  <div class="icon">🔐</div>
  <h1>Password Vault</h1>
  <p>Enter your master password to view saved credentials.</p>
  <form method="get" action="juanita://passwords">
    <input type="password" name="unlock_pass" placeholder="Master password" autofocus>
    <button type="submit">Unlock</button>
  </form>
</div>
</body>
</html>"#;

fn vault_html(
    creds: &[(String, String, String)],
    unlock_pass: &str,
    error: Option<&str>,
) -> String {
    let rows: String = if creds.is_empty() {
        "<tr><td colspan='4' style='text-align:center;color:#555;padding:24px'>No saved credentials yet.</td></tr>".to_string()
    } else {
        creds
            .iter()
            .enumerate()
            .map(|(i, (domain, user, email))| {
                let domain_esc = html_escape(domain);
                let user_esc = html_escape(user);
                let email_esc = html_escape(email);
                format!(
                    "<tr id='row-{i}'>\
                  <td>{domain_esc}</td>\
                  <td>{user_esc}</td>\
                  <td>{email_esc}</td>\
                  <td><button class='del-btn' onclick='deleteRow(\"{domain_esc}\")'>✕</button></td>\
                </tr>",
                    i = i,
                    domain_esc = domain_esc,
                    user_esc = user_esc,
                    email_esc = email_esc
                )
            })
            .collect()
    };

    let error_html = match error {
        Some(e) => format!("<div class='error'>⚠️ {}</div>", html_escape(e)),
        None => String::new(),
    };

    let unlock_pass_esc = html_escape(unlock_pass);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Passwords — Juanita Banana</title>
<style>
*{{box-sizing:border-box;margin:0;padding:0}}
body{{background:#0d0d0d;color:#e0e0e0;font-family:system-ui,-apple-system,sans-serif;padding:32px}}
h1{{font-size:1.4rem;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:10px}}
.error{{background:#450a0a;border:1px solid #7f1d1d;border-radius:8px;
  padding:10px 16px;margin-bottom:16px;font-size:.85rem;color:#fca5a5}}
.add-form {{
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  border-radius: 10px;
  padding: 20px;
  margin-bottom: 24px;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
}}
.add-form input {{
  background: #111;
  border: 1px solid #333;
  border-radius: 6px;
  padding: 8px 12px;
  color: #e0e0e0;
  font-size: .85rem;
  outline: none;
  flex: 1;
  min-width: 150px;
}}
.add-form input:focus {{
  border-color: #3b82f6;
}}
.add-form button {{
  background: #3b82f6;
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 8px 16px;
  font-size: .85rem;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
}}
.add-form button:hover {{
  background: #2563eb;
}}
table{{width:100%;border-collapse:collapse;background:#1a1a1a;border-radius:10px;overflow:hidden}}
th{{background:#222;padding:10px 16px;text-align:left;font-size:.8rem;
  color:#888;text-transform:uppercase;letter-spacing:.05em}}
td{{padding:10px 16px;border-top:1px solid #222;font-size:.875rem}}
tr:hover td{{background:#1f1f1f}}
.del-btn{{background:#7f1d1d;color:#fca5a5;border:none;border-radius:6px;
  padding:4px 10px;cursor:pointer;font-size:.8rem}}
.del-btn:hover{{background:#991b1b}}
.disclaimer-trigger {{
  text-align: center;
  margin-top: 40px;
  font-size: .8rem;
  color: #555;
  cursor: pointer;
  text-decoration: underline;
  transition: color 0.2s;
}}
.disclaimer-trigger:hover {{
  color: #888;
}}
.modal {{
  display: none;
  position: fixed;
  z-index: 1000;
  left: 0;
  top: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0,0,0,0.85);
  align-items: center;
  justify-content: center;
}}
.modal.show {{
  display: flex;
}}
.modal-content {{
  background: #161616;
  border: 1px solid #333;
  border-radius: 12px;
  padding: 32px;
  max-width: 520px;
  width: 90%;
  position: relative;
  box-shadow: 0 10px 30px rgba(0,0,0,0.5);
  font-size: .875rem;
  line-height: 1.5;
}}
.modal-content h2 {{
  font-size: 1.1rem;
  margin-bottom: 16px;
  color: #fff;
}}
.modal-content p {{
  margin-bottom: 14px;
  color: #b0b0b0;
}}
.modal-content p strong {{
  color: #fff;
}}
.close-modal {{
  position: absolute;
  top: 16px;
  right: 20px;
  font-size: 1.5rem;
  color: #777;
  cursor: pointer;
}}
.close-modal:hover {{
  color: #fff;
}}
</style>
</head>
<body>
<h1>🔐 Password Vault</h1>
{error_html}

<form method="get" action="juanita://passwords" class="add-form">
  <input type="hidden" name="unlock_pass" value="{unlock_pass_esc}">
  <input type="text" name="add_domain" placeholder="Domain (e.g. github.com)" required>
  <input type="text" name="add_username" placeholder="Username" required>
  <input type="text" name="add_email" placeholder="Email (optional)">
  <input type="password" name="add_password" placeholder="Password" required>
  <button type="submit">Add Credential</button>
</form>

<table>
  <thead>
    <tr><th>Domain</th><th>Username</th><th>Email</th><th></th></tr>
  </thead>
  <tbody id="cred-body">{rows}</tbody>
</table>

<div class="disclaimer-trigger" onclick="toggleDisclaimer()">ℹ️ Security & Privacy Philosophy</div>

<div id="disclaimer-modal" class="modal">
  <div class="modal-content">
    <span class="close-modal" onclick="toggleDisclaimer()">&times;</span>
    <h2>Security Philosophy & Privacy Disclaimer</h2>
    <p><strong>No password generator or rotation prompts:</strong> We will never nag you to rotate passwords or suggest "secure" passwords. Both are annoying security theater. Rotation targets human memory bottlenecks, while auto-generated passwords assume your compromised account on a compromised server hasn't already been drained of value. Passwords are a legacy design flaw. Physical, cryptographically signed hardware keys (not Google's QR surveillance-bait), throwaway identities, refusing to save confidential data on other people's hardware, and reducing your attack surface are the only real defenses.</p>
    <p><strong>No third-party leak checks:</strong> We do not query "Have I Been Pwned" or any other leak checkers. They do not know everything, and querying them leaks metadata. Most importantly: <em>we do not see your data, we do not check your data, we do not share your data, and we do not care about your data.</em></p>
  </div>
</div>

<script>
function toggleDisclaimer() {{
  var m = document.getElementById('disclaimer-modal');
  m.classList.toggle('show');
}}

function deleteRow(domain){{
  if(!confirm('Delete credentials for '+domain+'?'))return;
  if(window.webkit&&window.webkit.messageHandlers&&window.webkit.messageHandlers.juanita){{
    window.webkit.messageHandlers.juanita.postMessage(JSON.stringify({{
      type:'delete_credential',domain:domain
    }}));
  }}
  // Remove row optimistically
  var rows=document.querySelectorAll('#cred-body tr');
  for(var i=0;i<rows.length;i++){{
    if(rows[i].cells[0]&&rows[i].cells[0].textContent===domain){{
      rows[i].remove();break;
    }}
  }}
}}
</script>
</body>
</html>"#,
        rows = rows,
        error_html = error_html,
        unlock_pass_esc = unlock_pass_esc
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
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
        input == "juanita://passwords" || input.starts_with("juanita://passwords?")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        self.handle_policy(input, ctx);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        uri == "juanita://passwords" || uri.starts_with("juanita://passwords?")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool {
        match parse_query(uri, "unlock_pass") {
            None => {
                ctx.webview
                    .load_html(LOCKED_HTML, Some("juanita://passwords"));
            }
            Some(pass) => {
                // If add request is present, handle it first!
                if let (Some(domain), Some(username), Some(password)) = (
                    parse_query(uri, "add_domain"),
                    parse_query(uri, "add_username"),
                    parse_query(uri, "add_password"),
                ) {
                    let email = parse_query(uri, "add_email").unwrap_or_default();
                    if let Ok(mut mgr) =
                        crate::unsubscribe::db::SecureDbManager::new_responsive(&pass)
                    {
                        if let Ok(conn) = mgr.open_connection() {
                            let _ = crate::unsubscribe::db::save_full_credentials(
                                &conn, &domain, &username, &password, &email,
                            );
                            let _ = mgr.save_and_close(conn);
                            let mut idx = crate::util::credentials::CredentialIndex::load();
                            idx.register(&domain);
                        }
                    }
                    // Redirect back to clean unlocked view (no add params) to prevent refresh-re-submit
                    let clean_uri = format!(
                        "juanita://passwords?unlock_pass={}",
                        urlencoding::encode(&pass)
                    );
                    ctx.webview.load_uri(&clean_uri);
                    return true;
                }

                match crate::unsubscribe::db::SecureDbManager::new_responsive(&pass) {
                    Ok(mut mgr) => match mgr.open_connection() {
                        Ok(conn) => {
                            let creds = crate::unsubscribe::db::list_all_credentials(&conn);
                            let _ = mgr.save_and_close(conn);
                            let html = vault_html(&creds, &pass, None);
                            ctx.webview.load_html(&html, Some("juanita://passwords"));
                        }
                        Err(e) => {
                            let html = vault_html(&[], &pass, Some(&format!("DB error: {}", e)));
                            ctx.webview.load_html(&html, Some("juanita://passwords"));
                        }
                    },
                    Err(_) => {
                        ctx.webview.load_html(
                            &format!(
                                "{}<script>document.querySelector('p').textContent='Wrong master password. Try again.';</script>",
                                LOCKED_HTML
                            ),
                            Some("juanita://passwords"),
                        );
                    }
                }
            }
        }
        true
    }
}
