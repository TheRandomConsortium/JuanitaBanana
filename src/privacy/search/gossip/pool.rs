use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchPoolEntry {
    pub term: String,
    pub origin: String,
    pub ingested_epoch: u64,
    pub expires_epoch: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SearchNoisePool {
    pub entries: Vec<SearchPoolEntry>,
}

impl SearchNoisePool {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn clean_expired(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.entries
            .retain(|e| e.expires_epoch == 0 || e.expires_epoch > now);
    }

    pub fn add_term(&mut self, term: String, origin: String, ttl_days: u32) {
        self.clean_expired();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let expires = now + (ttl_days as u64 * 86400);
        self.entries.push(SearchPoolEntry {
            term,
            origin,
            ingested_epoch: now,
            expires_epoch: expires,
        });
    }

    pub fn get_all_terms(&mut self) -> Vec<SearchPoolEntry> {
        self.clean_expired();
        self.entries.clone()
    }

    pub fn remove_term(&mut self, term: &str) {
        self.entries.retain(|e| e.term != term);
    }

    pub fn remove_by_node(&mut self, node_id: &str) {
        let node_id_clean = node_id.trim().to_lowercase();
        self.entries.retain(|e| {
            let origin_clean = e.origin.trim().to_lowercase();
            !origin_clean.contains(&node_id_clean)
        });
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_NOISE_POOL: std::sync::Arc<std::sync::Mutex<SearchNoisePool>> =
        std::sync::Arc::new(std::sync::Mutex::new(SearchNoisePool::new()));
}
