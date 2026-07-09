use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotifiedDomain {
    pub domain: String,
    pub notified_date: String,
    pub emails_used: Vec<String>,
    pub status: String, // "NOTIFIED" or "REINCIDENT"
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct UnsubscribeRegistry {
    pub notified_domains: HashMap<String, NotifiedDomain>,
}

impl UnsubscribeRegistry {
    pub fn file_path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
            });
        let mut path = base.join("juanita-banana");
        fs::create_dir_all(&path).ok();
        path.push("unsubscribe_registry.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::file_path();
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::file_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            fs::write(path, data).ok();
        }
    }

    pub fn is_notified(&self, domain: &str) -> bool {
        self.notified_domains.contains_key(domain)
    }

    pub fn get_domain(&self, domain: &str) -> Option<&NotifiedDomain> {
        self.notified_domains.get(domain)
    }

    pub fn add_notified(&mut self, domain: String, emails: Vec<String>) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.notified_domains.insert(
            domain.clone(),
            NotifiedDomain {
                domain,
                notified_date: now,
                emails_used: emails,
                status: "NOTIFIED".to_string(),
            },
        );
        self.save();
    }

    pub fn mark_reincident(&mut self, domain: &str) {
        if let Some(entry) = self.notified_domains.get_mut(domain) {
            entry.status = "REINCIDENT".to_string();
            self.save();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsubscribe_registry_ops() {
        let mut reg = UnsubscribeRegistry::default();
        assert!(!reg.is_notified("example.com"));

        reg.add_notified(
            "example.com".to_string(),
            vec!["test@example.com".to_string()],
        );
        assert!(reg.is_notified("example.com"));

        let entry = reg.get_domain("example.com").unwrap();
        assert_eq!(entry.status, "NOTIFIED");
        assert_eq!(entry.emails_used, vec!["test@example.com".to_string()]);

        reg.mark_reincident("example.com");
        let entry = reg.get_domain("example.com").unwrap();
        assert_eq!(entry.status, "REINCIDENT");
    }
}
