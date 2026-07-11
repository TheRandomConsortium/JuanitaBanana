use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// A plain (unencrypted) index of domains for which the encrypted DB holds credentials.
/// This lets the browser hint credential availability without ever decrypting the vault.
/// The index intentionally stores only domain names — no usernames, no passwords.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CredentialIndex {
    pub domains: HashSet<String>,
}

impl CredentialIndex {
    pub fn index_path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
            });
        base.join("juanita-banana").join("credindex.bin")
    }

    pub fn load() -> Self {
        let path = Self::index_path();
        if let Ok(data) = fs::read(&path) {
            bincode::deserialize(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::index_path();
        if let Some(p) = path.parent() {
            let _ = fs::create_dir_all(p);
        }
        if let Ok(bin) = bincode::serialize(self) {
            let _ = fs::write(path, bin);
        }
    }

    pub fn register(&mut self, domain: &str) {
        self.domains.insert(domain.to_string());
        self.save();
    }

    pub fn remove(&mut self, domain: &str) {
        self.domains.remove(domain);
        self.save();
    }

    pub fn has_credentials(&self, domain: &str) -> bool {
        self.domains.iter().any(|d| {
            domain == d.as_str()
                || domain.ends_with(&format!(".{}", d))
                || d.ends_with(&format!(".{}", domain))
        })
    }
}
