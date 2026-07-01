use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct BanList {
    pub secret_id: String,
    pub banned_domains: HashSet<String>,
    pub toxic_domains: HashSet<String>,
    #[serde(skip)]
    pub vengeful_mode: bool,
}

pub type SharedBanList = Rc<RefCell<BanList>>;

impl BanList {
    fn state_path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
            });
        base.join("juanita-banana").join("banlist.bin")
    }

    pub fn load(config: &crate::util::config::AppConfig) -> SharedBanList {
        let path = Self::state_path();
        let expected_secret = config.expected_secret_id();

        let state = if path.exists() {
            if let Ok(content) = fs::read(&path) {
                if let Ok(mut loaded_state) = bincode::deserialize::<BanList>(&content) {
                    if loaded_state.secret_id != expected_secret {
                        println!("[BAN] CRITICAL: Secret ID mismatch! File was copied from another machine.");
                        loaded_state.vengeful_mode = true;
                    }
                    loaded_state
                } else {
                    println!("[BAN] CRITICAL: banlist.bin is corrupted! Tampering detected.");
                    BanList { vengeful_mode: true, ..Default::default() }
                }
            } else {
                BanList { vengeful_mode: true, ..Default::default() }
            }
        } else {
            // File does not exist. If config has search engines, it's not a fresh install!
            let mut s = BanList { secret_id: expected_secret.clone(), ..Default::default() };
            // We assume that if config exists, the directory existed before.
            // But actually, we know it's not a fresh install if they have modified config or we can just rely on first_launch_epoch.
            // If they deleted banlist.bin, it's missing but expected.
            println!("[BAN] Missing banlist.bin. Treating as fresh install or tampering.");
            // To be truly vengeful: if the path parent exists and has config.json, but no banlist, we brick.
            let config_path = path.parent().unwrap().join("config.json");
            if config_path.exists() {
                println!("[BAN] CRITICAL: config.json exists but banlist.bin is missing! Tampering detected.");
                s.vengeful_mode = true;
            }
            s
        };
        
        Rc::new(RefCell::new(state))
    }

    pub fn save(&self) {
        let path = Self::state_path();
        if let Some(p) = path.parent() {
            let _ = fs::create_dir_all(p);
        }
        if let Ok(bin) = bincode::serialize(self) {
            let _ = fs::write(path, bin);
        }
    }

    pub fn unban(&mut self, domain: &str) {
        self.banned_domains.remove(domain);
    }

    pub fn ban(&mut self, domain: &str) {
        self.banned_domains.insert(domain.to_string());
    }

    pub fn is_banned(&self, uri: &str) -> bool {
        if self.vengeful_mode {
            return true; // BRICKED! Everything is banned.
        }
        self.banned_domains.iter().any(|d| uri.contains(d.as_str()))
    }
}

pub fn normalize_url(raw: &str) -> String {
    let t = raw.trim();
    if t.starts_with("http://") || t.starts_with("https://") {
        t.to_string()
    } else if t.contains('.') && !t.contains(' ') {
        format!("https://{t}")
    } else {
        format!("https://duckduckgo.com/?q={}", t.replace(' ', "+"))
    }
}

pub fn extract_domain(uri: &str) -> String {
    uri.split("://")
        .nth(1)
        .unwrap_or(uri)
        .split('/')
        .next()
        .unwrap_or(uri)
        .to_string()
}
