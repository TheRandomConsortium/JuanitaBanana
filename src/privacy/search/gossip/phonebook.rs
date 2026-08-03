use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerIdentity {
    pub node_id: String,
    pub public_key: [u8; 32],
    pub endpoint: String,
    pub last_seen_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwarmPhonebook {
    pub peers: Vec<PeerIdentity>,
}

impl SwarmPhonebook {
    fn phonebook_path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
            });
        base.join("juanita-banana").join("phonebook.bin")
    }

    pub fn empty() -> Self {
        Self { peers: Vec::new() }
    }

    pub fn load() -> Self {
        let path = Self::phonebook_path();
        if path.exists() {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(pb) = bincode::deserialize::<SwarmPhonebook>(&bytes) {
                    return pb;
                }
            }
        }
        SwarmPhonebook::default()
    }

    pub fn save(&self) {
        let path = Self::phonebook_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bin) = bincode::serialize(self) {
            let _ = fs::write(path, bin);
        }
    }

    pub fn register_peer(&mut self, node_id: String, public_key: [u8; 32], endpoint: String) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.peers.retain(|p| p.node_id != node_id);
        self.peers.push(PeerIdentity {
            node_id,
            public_key,
            endpoint,
            last_seen_epoch: now,
        });
        self.save();
    }

    pub fn remove_peer(&mut self, node_id: &str) {
        self.peers.retain(|p| p.node_id != node_id);
        self.save();
    }

    pub fn get_active_peers(&self) -> Vec<PeerIdentity> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.peers
            .iter()
            .filter(|p| now.saturating_sub(p.last_seen_epoch) < 86400)
            .cloned()
            .collect()
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_PHONEBOOK: std::sync::Arc<std::sync::Mutex<SwarmPhonebook>> =
        std::sync::Arc::new(std::sync::Mutex::new(SwarmPhonebook::load()));
}
