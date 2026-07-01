use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct BanList {
    pub banned_domains: HashSet<String>,
    pub toxic_domains: HashSet<String>,
}

pub type SharedBanList = Rc<RefCell<BanList>>;

impl BanList {
    fn state_path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
            });
        base.join("juanita-banana").join("banlist.json")
    }

    pub fn load() -> SharedBanList {
        let path = Self::state_path();
        let state = if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            BanList::default()
        };
        Rc::new(RefCell::new(state))
    }

    pub fn save(&self) {
        let path = Self::state_path();
        if let Some(p) = path.parent() {
            let _ = fs::create_dir_all(p);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn ban(&mut self, domain: &str) {
        self.banned_domains.insert(domain.to_string());
    }

    pub fn is_banned(&self, uri: &str) -> bool {
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
