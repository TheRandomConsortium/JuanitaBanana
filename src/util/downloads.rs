use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::process::Command;
use std::rc::Rc;
use webkit2gtk::{DownloadExt, URIRequestExt, WebContextExt};

pub struct DownloadManager {
    // Maps download ID -> (Sandbox Path, Original Filename, Is Finished, Progress, Origin Domain)
    pub active_downloads: HashMap<String, (String, String, bool, f64, String)>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            active_downloads: HashMap::new(),
        }
    }

    pub fn generate_html(&self) -> String {
        let mut rows = String::new();
        for (id, (_path, filename, finished, progress, _origin_domain)) in &self.active_downloads {
            let status = if *finished {
                "Ready".to_string()
            } else {
                format!("Downloading... {:.0}%", progress * 100.0)
            };

            let action_btns = if *finished {
                format!(
                    r#"
                    <button onclick="window.location.href='juanita://downloads/open?id={}'" style="margin-right: 10px; background: #e0a900; color: #000;">Open in Sandbox</button>
                    <button onclick="window.location.href='juanita://downloads/persist?id={}'" style="background: #28a745;">Make Permanent</button>
                    <button onclick="window.location.href='juanita://downloads/delete?id={}'" style="background: #dc3545; margin-left: 10px;">Shred</button>
                "#,
                    id, id, id
                )
            } else {
                String::from("<span>Wait...</span>")
            };

            rows.push_str(&format!(
                r#"<tr>
                    <td style="padding: 10px;">{}</td>
                    <td style="padding: 10px;">{}</td>
                    <td style="padding: 10px;">{}</td>
                </tr>"#,
                filename, status, action_btns
            ));
        }

        if self.active_downloads.is_empty() {
            rows = String::from("<tr><td colspan='3' style='text-align: center; padding: 20px;'>No isolated downloads yet.</td></tr>");
        }

        let shared_css = crate::browsing::internal::SHARED_CSS.as_str();
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Downloads — Juanita Banana</title>
    <style>
        {shared_css}
    </style>
</head>
<body class="jb-page" style="justify-content: flex-start; padding-top: var(--jb-space-4xl);">
    <div class="jb-container jb-container-wide">
        <div class="jb-title-group" style="width: 100%; text-align: left;">
            <h1 class="jb-title">📦 Sandboxed Downloads</h1>
            <div class="jb-subtitle">All downloads are isolated in a temporary RAM disk. They cannot read your personal files or access the internet when opened.</div>
        </div>
        <div class="jb-card">
            <table class="jb-table">
                <thead>
                    <tr>
                        <th>Filename</th>
                        <th>Status</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {rows}
                </tbody>
            </table>
        </div>
        <nav class="jb-nav">
            <a class="jb-nav-link" href="juanita://home">Home</a>
            <a class="jb-nav-link" href="juanita://config">Settings</a>
            <a class="jb-nav-link" href="juanita://about">About</a>
        </nav>
    </div>
</body>
</html>"#,
            shared_css = shared_css,
            rows = rows
        )
    }

    pub fn open_sandboxed(&self, id: &str) {
        if let Some((path, _filename, true, _, origin_domain)) = self.active_downloads.get(id) {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
            let run_dir =
                std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());

            let parent_dir = std::path::Path::new(path).parent().unwrap();
            let fake_xdg_open_path = parent_dir.join("fake-xdg-open");
            let script = format!(
                r#"#!/bin/bash
TARGET="$1"

if [[ "$TARGET" == "/tmp/{}" ]]; then
    /usr/bin/gio open "$TARGET"
    exit 0
fi

RESULT=$(zenity --question --title="Juanita Banana Sandbox" --text="A sandboxed document is trying to escape and access your host system:\n\n<b>$TARGET</b>\n\nAllow this action?" --width=450 --ok-label="Allow" --cancel-label="Reject" --extra-button="Reject & Ban Origin")
EXIT_CODE=$?
if [ $EXIT_CODE -eq 0 ]; then
    busctl --user call org.freedesktop.portal.Desktop /org/freedesktop/portal/desktop org.freedesktop.portal.OpenURI OpenURI "ssa{{sv}}" "" "$TARGET" 0
elif [ "$RESULT" = "Reject & Ban Origin" ]; then
    busctl --user call org.freedesktop.portal.Desktop /org/freedesktop/portal/desktop org.freedesktop.portal.OpenURI OpenURI "ssa{{sv}}" "" "juanita://external/ban?url=$TARGET&domain={}" 0
fi
"#,
                _filename, origin_domain
            );
            std::fs::write(&fake_xdg_open_path, script).ok();
            std::process::Command::new("chmod")
                .arg("+x")
                .arg(&fake_xdg_open_path)
                .status()
                .ok();

            let fake_mimeapps_path = parent_dir.join("mimeapps.list");
            std::fs::write(&fake_mimeapps_path, "[Default Applications]\nx-scheme-handler/http=fake-browser.desktop\nx-scheme-handler/https=fake-browser.desktop\n").ok();

            let fake_desktop_path = parent_dir.join("fake-browser.desktop");
            std::fs::write(&fake_desktop_path, "[Desktop Entry]\nName=Fake Browser\nExec=/usr/bin/xdg-open %U\nType=Application\nMimeType=x-scheme-handler/http;x-scheme-handler/https;\n").ok();

            let fake_bin_dir = parent_dir.join("fake-bin");
            std::fs::create_dir_all(&fake_bin_dir).ok();
            for bin in &[
                "firefox",
                "chrome",
                "chromium",
                "brave-browser",
                "vlc",
                "mpv",
                "xdg-open",
                "gio",
            ] {
                std::os::unix::fs::symlink("/usr/bin/xdg-open", fake_bin_dir.join(bin)).ok();
            }

            let status = Command::new("bwrap")
                .arg("--unshare-net")
                .arg("--unshare-pid")
                .arg("--unshare-ipc")
                .arg("--setenv")
                .arg("PATH")
                .arg("/tmp/fake-bin:/usr/bin:/bin")
                .arg("--ro-bind")
                .arg("/")
                .arg("/")
                .arg("--dev")
                .arg("/dev")
                .arg("--tmpfs")
                .arg(&home)
                .arg("--dir")
                .arg(format!("{}/.config", home))
                .arg("--ro-bind-try")
                .arg(&fake_mimeapps_path)
                .arg(format!("{}/.config/mimeapps.list", home))
                .arg("--dir")
                .arg(format!("{}/.local/share/applications", home))
                .arg("--ro-bind-try")
                .arg(&fake_desktop_path)
                .arg(format!(
                    "{}/.local/share/applications/fake-browser.desktop",
                    home
                ))
                .arg("--tmpfs")
                .arg("/tmp")
                .arg("--ro-bind-try")
                .arg(&fake_bin_dir)
                .arg("/tmp/fake-bin")
                .arg("--ro-bind-try")
                .arg("/tmp/.X11-unix")
                .arg("/tmp/.X11-unix")
                .arg("--ro-bind-try")
                .arg(format!("{}/wayland-0", run_dir))
                .arg(format!("{}/wayland-0", run_dir))
                .arg("--ro-bind-try")
                .arg(format!("{}/bus", run_dir))
                .arg(format!("{}/bus", run_dir))
                .arg("--bind")
                .arg(path)
                .arg(format!("/tmp/{}", _filename))
                .arg("--ro-bind-try")
                .arg(&fake_xdg_open_path)
                .arg("/usr/bin/xdg-open")
                .arg("--ro-bind-try")
                .arg(&fake_xdg_open_path)
                .arg("/bin/xdg-open")
                .arg("/usr/bin/xdg-open")
                .arg(format!("/tmp/{}", _filename))
                .spawn();

            if let Err(e) = status {
                crate::log!(Error, SANDBOX, "Failed to launch bwrap: {}", e);
            } else {
                crate::log!(Info, SANDBOX, "Launched isolated viewer for {}", _filename);
            }
        }
    }

    pub fn make_permanent(&mut self, id: &str) {
        if let Some((path, filename, true, _, _)) = self.active_downloads.get(id) {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
            let dest_dir = std::path::Path::new(&home).join("Downloads");
            std::fs::create_dir_all(&dest_dir).ok();
            let dest_path = dest_dir.join(filename);

            if let Err(e) = std::fs::copy(path, &dest_path) {
                crate::log!(Error, SANDBOX, "Failed to persist file: {}", e);
            } else {
                crate::log!(raw: Info, SANDBOX, "File persisted to {:?}", dest_path);
                // Also shred it from sandbox since we moved it
                let _ = std::fs::remove_file(path);
                if let Some(parent) = std::path::Path::new(path).parent() {
                    let _ = std::fs::remove_dir(parent); // Try to clean up UUID dir
                }
            }
        }
        self.active_downloads.remove(id);
    }

    pub fn shred(&mut self, id: &str) {
        if let Some((path, _filename, _, _, _)) = self.active_downloads.get(id) {
            let _ = std::fs::remove_file(path);
            if let Some(parent) = std::path::Path::new(path).parent() {
                let _ = std::fs::remove_dir(parent);
            }
            crate::log!(Info, SANDBOX, "Shredded file {}", _filename);
        }
        self.active_downloads.remove(id);
    }
}

pub fn setup_downloads(
    web_context: &webkit2gtk::WebContext,
    downloads: &Rc<RefCell<DownloadManager>>,
    tx_activate: &async_channel::Sender<(String, bool)>,
) {
    let downloads_ctx = downloads.clone();
    let tx_download = tx_activate.clone();
    web_context.connect_download_started(move |_context, download| {
        let id = format!("{}", rand::thread_rng().gen::<u64>());

        let mut origin_domain = String::from("unknown");
        #[allow(deprecated)]
        if let Some(req) = download.request() {
            if let Some(uri) = req.uri() {
                origin_domain = crate::browsing::browser::extract_domain(uri.as_str());
            }
        }

        let downloads_ctx_dest = downloads_ctx.clone();
        let id_dest = id.clone();
        download.connect_decide_destination(move |dl, suggested_filename| {
            let filename = suggested_filename.to_string();
            let dest_dir = format!("/tmp/juanita-sandbox-{}", id_dest);
            std::fs::create_dir_all(&dest_dir).ok();

            let dest_path = format!("{}/{}", dest_dir, filename);
            dl.set_destination(&format!("file://{}", dest_path));

            downloads_ctx_dest.borrow_mut().active_downloads.insert(
                id_dest.clone(),
                (
                    dest_path,
                    filename.clone(),
                    false,
                    0.0,
                    origin_domain.clone(),
                ),
            );

            std::process::Command::new("notify-send")
                .arg("Juanita Banana 🍌")
                .arg(format!("Downloading: {}", filename))
                .spawn()
                .ok();

            true
        });

        let downloads_ctx_prog = downloads_ctx.clone();
        let id_prog = id.clone();
        download.connect_received_data(move |dl, _data_length| {
            if let Some(entry) = downloads_ctx_prog
                .borrow_mut()
                .active_downloads
                .get_mut(&id_prog)
            {
                entry.3 = dl.estimated_progress();
            }
        });

        let downloads_fin = downloads_ctx.clone();
        let id_fin = id.clone();
        let tx_clone = tx_download.clone();
        download.connect_finished(move |_| {
            if let Some(entry) = downloads_fin.borrow_mut().active_downloads.get_mut(&id_fin) {
                entry.2 = true;
                let filename = entry.1.clone();
                crate::log!(Info, SANDBOX, "Download finished: {}", filename);

                let tx_thread = tx_clone.clone();
                std::thread::spawn(move || {
                    if let Ok(out) = std::process::Command::new("notify-send")
                        .arg("--action=open=View Downloads")
                        .arg("Juanita Banana 🍌")
                        .arg(format!("Ready in Sandbox: {}", filename))
                        .output()
                    {
                        if String::from_utf8_lossy(&out.stdout).trim() == "open" {
                            let _ =
                                tx_thread.send_blocking(("juanita://downloads".to_string(), false));
                        }
                    }
                });
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_manager_html_empty() {
        let dm = DownloadManager::new();
        let html = dm.generate_html();
        assert!(html.contains("No isolated downloads yet."));
        assert!(!html.contains("Downloading..."));
        assert!(!html.contains("Ready"));
    }

    #[test]
    fn test_download_manager_html_active() {
        let mut dm = DownloadManager::new();
        dm.active_downloads.insert(
            "1234".to_string(),
            (
                "/tmp/juanita-sandbox-1234/test.pdf".to_string(),
                "test.pdf".to_string(),
                false,
                0.45,
                "example.com".to_string(),
            ),
        );

        let html = dm.generate_html();
        assert!(html.contains("test.pdf"));
        assert!(html.contains("Downloading... 45%"));
        assert!(html.contains("Wait..."));
        assert!(!html.contains("Open in Sandbox"));
    }

    #[test]
    fn test_download_manager_html_finished() {
        let mut dm = DownloadManager::new();
        dm.active_downloads.insert(
            "1234".to_string(),
            (
                "/tmp/juanita-sandbox-1234/test.pdf".to_string(),
                "test.pdf".to_string(),
                true,
                1.0,
                "example.com".to_string(),
            ),
        );

        let html = dm.generate_html();
        assert!(html.contains("test.pdf"));
        assert!(html.contains("Ready"));
        assert!(html.contains("Open in Sandbox"));
        assert!(html.contains("juanita://downloads/open?id=1234"));
    }
}
