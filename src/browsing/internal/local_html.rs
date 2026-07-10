use crate::browsing::internal::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct LocalHtmlPage;

/// Returns true if the URI points to a local HTML file.
fn is_local_html(uri: &str) -> bool {
    let lower = uri.to_lowercase();
    uri.starts_with("file://") && (lower.ends_with(".html") || lower.ends_with(".htm"))
}

fn read_file_from_uri(uri: &str) -> Option<String> {
    let path = uri.strip_prefix("file://")?;
    let decoded = urlencoding::decode(path).ok()?;
    std::fs::read_to_string(decoded.as_ref()).ok()
}

fn viewer_page(uri: &str, source: &str) -> String {
    let escaped = source
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>HTML Viewer — {uri}</title>
<style>
  * {{ box-sizing: border-box; margin: 0; padding: 0; }}
  body {{
    background: #0d0d0d;
    color: #e0e0e0;
    font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
    display: flex;
    flex-direction: column;
    height: 100vh;
  }}
  #toolbar {{
    background: #1a1a1a;
    border-bottom: 1px solid #2a2a2a;
    padding: 10px 16px;
    display: flex;
    align-items: center;
    gap: 14px;
    flex-shrink: 0;
  }}
  #toolbar .warning {{
    flex: 1;
    font-size: 13px;
    color: #facc15;
    display: flex;
    align-items: center;
    gap: 8px;
  }}
  #toolbar .warning .icon {{ font-size: 18px; }}
  #toolbar .path {{ font-size: 11px; color: #666; margin-left: auto; word-break: break-all; max-width: 40%; text-align: right; }}
  #render-btn {{
    background: #16a34a;
    color: #fff;
    border: none;
    border-radius: 6px;
    padding: 7px 18px;
    font-size: 13px;
    font-weight: 700;
    cursor: pointer;
    transition: background 0.15s;
    flex-shrink: 0;
  }}
  #render-btn:hover {{ background: #15803d; }}
  #save-btn {{
    background: #1d4ed8;
    color: #fff;
    border: none;
    border-radius: 6px;
    padding: 7px 18px;
    font-size: 13px;
    font-weight: 700;
    cursor: pointer;
    transition: background 0.15s;
    flex-shrink: 0;
  }}
  #save-btn:hover {{ background: #1e40af; }}
  #save-btn.saved {{ background: #15803d; }}
  #editor {{
    flex: 1;
    width: 100%;
    background: #111;
    color: #d4d4d4;
    font-family: inherit;
    font-size: 13px;
    line-height: 1.6;
    border: none;
    outline: none;
    padding: 20px;
    resize: none;
    tab-size: 2;
    overflow: auto;
  }}
  #editor:focus {{ background: #111; }}
</style>
</head>
<body>
<div id="toolbar">
  <div class="warning">
    <span class="icon">⚠️</span>
    <span>Inspect this HTML file before rendering — it may contain scripts or unsafe content.</span>
  </div>
  <button id="save-btn" onclick="saveFile()">💾 Save</button>
  <button id="render-btn" onclick="renderFile()">▶ Render</button>
  <div class="path">{uri}</div>
</div>
<textarea id="editor" spellcheck="false">{escaped}</textarea>
<script>
  const FILE_URI = {uri_json};

  function renderFile() {{
    const src = document.getElementById('editor').value;
    const blob = new Blob([src], {{type: 'text/html'}});
    const url  = URL.createObjectURL(blob);
    window.location.href = url;
  }}

  function saveFile() {{
    const src = document.getElementById('editor').value;
    if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.juanita) {{
      window.webkit.messageHandlers.juanita.postMessage(JSON.stringify({{
        type: 'save_local_html',
        uri: FILE_URI,
        content: src
      }}));
      const btn = document.getElementById('save-btn');
      btn.textContent = '✓ Saved';
      btn.classList.add('saved');
      setTimeout(() => {{ btn.textContent = '💾 Save'; btn.classList.remove('saved'); }}, 2000);
    }}
  }}

  // Tab key inserts spaces instead of losing focus
  document.getElementById('editor').addEventListener('keydown', e => {{
    if (e.key === 'Tab') {{
      e.preventDefault();
      const s = e.target, start = s.selectionStart, end = s.selectionEnd;
      s.value = s.value.slice(0, start) + '  ' + s.value.slice(end);
      s.selectionStart = s.selectionEnd = start + 2;
    }}
  }});
</script>
</body>
</html>"#,
        uri = uri,
        escaped = escaped,
        uri_json = serde_json::to_string(uri).unwrap_or_else(|_| "\"\"".to_string()),
    )
}

impl InternalPage for LocalHtmlPage {
    fn matches_input(&self, input: &str) -> bool {
        is_local_html(input)
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        let default = ctx.config.local_html_default.as_str();
        if default == "render" {
            ctx.webview.load_uri(input);
        } else {
            let uri_clone = input.to_string();
            let webview_clone = ctx.webview.clone();
            match read_file_from_uri(input) {
                Some(src) => {
                    gtk::glib::idle_add_local(move || {
                        let html = viewer_page(&uri_clone, &src);
                        webview_clone.load_html(&html, Some("juanita://local-html-viewer/"));
                        gtk::glib::ControlFlow::Break
                    });
                }
                None => {
                    gtk::glib::idle_add_local(move || {
                        let err = format!(
                            "<html><body style='font-family:monospace;background:#0d0d0d;color:#f87171;padding:2em'>\
                            <h2>Cannot read file</h2><p>{}</p></body></html>",
                            uri_clone
                        );
                        webview_clone.load_html(&err, Some("juanita://error"));
                        gtk::glib::ControlFlow::Break
                    });
                }
            }
        }
    }

    fn matches_policy(&self, uri: &str) -> bool {
        is_local_html(uri)
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        let config = crate::util::config::AppConfig::load();
        config.local_html_default.as_str() != "render"
    }

    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool {
        println!("[DEBUG LOCAL_HTML] handle_policy: uri = {}", uri);
        let default = ctx.config.local_html_default.as_str();
        println!("[DEBUG LOCAL_HTML] local_html_default mode = {}", default);
        if default == "render" {
            return false; // Let WebKit render it normally
        }
        let uri_clone = uri.to_string();
        let webview_clone = ctx.webview.clone();
        match read_file_from_uri(uri) {
            Some(src) => {
                println!(
                    "[DEBUG LOCAL_HTML] read file successfully, len = {}",
                    src.len()
                );
                gtk::glib::idle_add_local(move || {
                    let html = viewer_page(&uri_clone, &src);
                    webview_clone.load_html(&html, Some("juanita://local-html-viewer/"));
                    gtk::glib::ControlFlow::Break
                });
            }
            None => {
                println!("[DEBUG LOCAL_HTML] failed to read file from URI: {}", uri);
                gtk::glib::idle_add_local(move || {
                    let err = format!(
                        "<html><body style='font-family:monospace;background:#0d0d0d;color:#f87171;padding:2em'>\
                        <h2>Cannot read file</h2><p>{}</p></body></html>",
                        uri_clone
                    );
                    webview_clone.load_html(&err, Some("juanita://error"));
                    gtk::glib::ControlFlow::Break
                });
            }
        }
        true
    }
}
