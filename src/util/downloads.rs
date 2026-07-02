use std::collections::HashMap;
use std::process::Command;

pub struct DownloadManager {
    // Maps download ID -> (Sandbox Path, Original Filename, Is Finished)
    pub active_downloads: HashMap<String, (String, String, bool)>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            active_downloads: HashMap::new(),
        }
    }

    pub fn generate_html(&self) -> String {
        let mut rows = String::new();
        for (id, (_path, filename, finished)) in &self.active_downloads {
            let status = if *finished { "Ready" } else { "Downloading..." };
            
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
        if let Some((path, _filename, true)) = self.active_downloads.get(id) {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
            
            // Launch Bubblewrap to open the file completely isolated
            // --unshare-all: No IPC, User, PID
            // --unshare-net: No Internet
            // --ro-bind /usr /usr: Need binaries
            // --tmpfs $HOME: Hide user home
            let status = Command::new("bwrap")
                .arg("--unshare-all")
                .arg("--unshare-net")
                .arg("--ro-bind")
                .arg("/usr")
                .arg("/usr")
                .arg("--ro-bind-try")
                .arg("/lib")
                .arg("/lib")
                .arg("--ro-bind-try")
                .arg("/lib64")
                .arg("/lib64")
                .arg("--ro-bind-try")
                .arg("/etc/fonts")
                .arg("/etc/fonts") // needed for fonts
                .arg("--ro-bind-try")
                .arg("/usr/share/fonts")
                .arg("/usr/share/fonts")
                .arg("--dir")
                .arg("/tmp")
                .arg("--tmpfs")
                .arg(&home)
                .arg("--bind")
                .arg(path)
                .arg(format!("/sandbox_{}", _filename))
                .arg("/usr/bin/xdg-open")
                .arg(format!("/sandbox_{}", _filename))
                .spawn();

            if let Err(e) = status {
                eprintln!("[SANDBOX] Failed to launch bwrap: {}", e);
            } else {
                println!("[SANDBOX] Launched isolated viewer for {}", _filename);
            }
        }
    }

    pub fn make_permanent(&mut self, id: &str) {
        if let Some((path, filename, true)) = self.active_downloads.get(id) {
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
        if let Some((path, _filename, _)) = self.active_downloads.get(id) {
            let _ = std::fs::remove_file(path);
            if let Some(parent) = std::path::Path::new(path).parent() {
                let _ = std::fs::remove_dir(parent);
            }
            println!("[SANDBOX] Shredded file {}", _filename);
        }
        self.active_downloads.remove(id);
    }
}

