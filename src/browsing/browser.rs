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

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PeerBanList {
    pub banned_peers: HashSet<String>,
}

impl PeerBanList {
    fn state_path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
            });
        base.join("juanita-banana").join("peer_banlist.bin")
    }

    pub fn load() -> Self {
        let path = Self::state_path();
        if path.exists() {
            if let Ok(content) = fs::read(&path) {
                if let Ok(loaded) = bincode::deserialize::<PeerBanList>(&content) {
                    return loaded;
                }
            }
        }
        PeerBanList::default()
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

    pub fn ban_peer(&mut self, node_id: &str) {
        self.banned_peers.insert(node_id.trim().to_lowercase());
        self.save();
    }

    pub fn is_peer_banned(&self, node_id: &str) -> bool {
        self.banned_peers.contains(&node_id.trim().to_lowercase())
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_PEER_BANLIST: std::sync::Arc<std::sync::Mutex<PeerBanList>> =
        std::sync::Arc::new(std::sync::Mutex::new(PeerBanList::load()));
}

pub fn ban_peer(node_id: &str) {
    if let Ok(mut guard) = GLOBAL_PEER_BANLIST.lock() {
        guard.ban_peer(node_id);
    }
}

pub fn is_peer_banned(node_id: &str) -> bool {
    if let Ok(guard) = GLOBAL_PEER_BANLIST.lock() {
        guard.is_peer_banned(node_id)
    } else {
        false
    }
}

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
                        crate::log!(
                            Error,
                            BAN,
                            "CRITICAL: Secret ID mismatch! File was copied from another machine."
                        );
                        loaded_state.vengeful_mode = true;
                    }
                    loaded_state
                } else {
                    crate::log!(
                        Error,
                        BAN,
                        "CRITICAL: banlist.bin is corrupted! Tampering detected."
                    );
                    BanList {
                        vengeful_mode: true,
                        ..Default::default()
                    }
                }
            } else {
                BanList {
                    vengeful_mode: true,
                    ..Default::default()
                }
            }
        } else {
            let mut s = BanList {
                secret_id: expected_secret.clone(),
                ..Default::default()
            };
            crate::log!(
                Warn,
                BAN,
                "Missing banlist.bin. Treating as fresh install or tampering."
            );
            let config_path = path.parent().unwrap().join("config.json");
            if config_path.exists() {
                crate::log!(
                    Error,
                    BAN,
                    "CRITICAL: config.json exists but banlist.bin is missing! Tampering detected."
                );
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

pub fn punycode_label(label: &str) -> String {
    if !label.is_ascii() {
        if let Some(encoded) = idna::punycode::encode_str(label) {
            return format!("xn--{}", encoded);
        }
    }
    label.to_string()
}

pub fn punycode_host(host: &str) -> String {
    host.split('.')
        .map(punycode_label)
        .collect::<Vec<_>>()
        .join(".")
}

pub fn normalize_url(raw: &str) -> String {
    let t = raw.trim();
    let (scheme_prefix, rest) = if let Some(stripped) = t.strip_prefix("http://") {
        ("http://", stripped)
    } else if let Some(stripped) = t.strip_prefix("https://") {
        ("https://", stripped)
    } else if t.contains('.') && !t.contains(' ') {
        ("https://", t)
    } else {
        return format!("https://duckduckgo.com/?q={}", t.replace(' ', "+"));
    };

    let parts: Vec<&str> = rest.splitn(2, '/').collect();
    let host_part = parts[0];
    let path_part = if parts.len() > 1 {
        format!("/{}", parts[1])
    } else {
        "/".to_string()
    };

    let safe_host = punycode_host(host_part);
    format!("{}{}{}", scheme_prefix, safe_host, path_part)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ban_and_unban_peer() {
        let mut peer_banlist = PeerBanList::default();
        let peer_id = "node-juanita-test1234";

        assert!(!peer_banlist.is_peer_banned(peer_id));
        peer_banlist.ban_peer(peer_id);
        assert!(peer_banlist.is_peer_banned(peer_id));
    }

    #[test]
    fn test_normalize_url_punycode() {
        let raw = "lo.randºm";
        let normalized = normalize_url(raw);
        assert_eq!(normalized, "https://lo.xn--randm-cka/");
    }
}
