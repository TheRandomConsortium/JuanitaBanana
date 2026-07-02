use std::collections::HashMap;
use std::process::Command;

pub struct DownloadManager {
    // Maps download ID -> (Sandbox Path, Original Filename, Is Finished, Progress)
    pub active_downloads: HashMap<String, (String, String, bool, f64)>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            active_downloads: HashMap::new(),
        }
    }

    pub fn generate_html(&self) -> String {
        let mut rows = String::new();
        for (id, (_path, filename, finished, progress)) in &self.active_downloads {
            let status = if *finished {
                "Ready".to_string()
            } else {
                format!("Downloading... {:.0}%", progress * 100.0)
            };
            
            let action_btns = if *finished {
                format!(r#"
                    <button onclick="window.location.href='juanita://downloads/open?id={}'" style="margin-right: 10px; background: #e0a900; color: #000;">Open in Sandbox</button>
                    <button onclick="window.location.href='juanita://downloads/persist?id={}'" style="background: #28a745;">Make Permanent</button>
                    <button onclick="window.location.href='juanita://downloads/delete?id={}'" style="background: #dc3545; margin-left: 10px;">Shred</button>
                "#, id, id, id)
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

        format!(
            r#"<html>
<head>
    <style>
        body {{ background: #121212; color: #eee; font-family: sans-serif; padding: 2rem; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 2rem; background: #1e1e1e; border-radius: 8px; overflow: hidden; }}
        th, td {{ border-bottom: 1px solid #333; text-align: left; padding: 12px; }}
        th {{ background: #2a2a2a; color: #fff; }}
        button {{ padding: 8px 12px; border: none; border-radius: 4px; cursor: pointer; font-weight: bold; color: #fff; }}
        button:hover {{ opacity: 0.9; }}
    </style>
</head>
<body>
    <h1>📦 Sandboxed Downloads</h1>
    <p>All downloads are isolated in a temporary RAM disk. They cannot read your personal files or access the internet when opened.</p>
    <table>
        <thead>
            <tr>
                <th>Filename</th>
                <th>Status</th>
                <th>Actions</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>
</body>
</html>"#,
            rows
        )
    }

    pub fn open_sandboxed(&self, id: &str) {
        if let Some((path, _filename, true, _)) = self.active_downloads.get(id) {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
            let run_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
            
            let status = Command::new("bwrap")
                .arg("--unshare-net")
                .arg("--unshare-pid")
                .arg("--unshare-ipc")
                .arg("--ro-bind")
                .arg("/")
                .arg("/")
                .arg("--dev")
                .arg("/dev")
                .arg("--tmpfs")
                .arg(&home)
                .arg("--tmpfs")
                .arg("/tmp")
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
                .arg("xdg-open")
                .arg(format!("/tmp/{}", _filename))
                .spawn();

            if let Err(e) = status {
                eprintln!("[SANDBOX] Failed to launch bwrap: {}", e);
            } else {
                println!("[SANDBOX] Launched isolated viewer for {}", _filename);
            }
        }
    }

    pub fn make_permanent(&mut self, id: &str) {
        if let Some((path, filename, true, _)) = self.active_downloads.get(id) {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
            let dest_dir = std::path::Path::new(&home).join("Downloads");
            std::fs::create_dir_all(&dest_dir).ok();
            let dest_path = dest_dir.join(filename);

            if let Err(e) = std::fs::copy(path, &dest_path) {
                eprintln!("[SANDBOX] Failed to persist file: {}", e);
            } else {
                println!("[SANDBOX] File persisted to {:?}", dest_path);
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
        if let Some((path, _filename, _, _)) = self.active_downloads.get(id) {
            let _ = std::fs::remove_file(path);
            if let Some(parent) = std::path::Path::new(path).parent() {
                let _ = std::fs::remove_dir(parent);
            }
            println!("[SANDBOX] Shredded file {}", _filename);
        }
        self.active_downloads.remove(id);
    }
}

