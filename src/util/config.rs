use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchEngineRule {
    pub name: String,
    pub domain_regex: String,
    pub query_params: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RssSource {
    pub name: String,
    pub url: String,
}

pub const HONEST_USER_AGENT: &str = "JuanitaBanana/0.1 (FOSS; Not-Google; Linux)";

pub const GENUINE_UA_POOL: &[&str] = &[
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64; rv:127.0) Gecko/20100101 Firefox/127.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:127.0) Gecko/20100101 Firefox/127.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:127.0) Gecko/20100101 Firefox/127.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 Edg/126.0.0.0",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub search_engines: Vec<SearchEngineRule>,
    pub rss_sources: Vec<RssSource>,
    pub max_concurrent_searches: usize,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub noise_queries_amount: usize,
    pub first_launch_epoch: u64,
    pub ad_click_probability: f64,
    pub ad_jitter_min_secs: u64,
    pub ad_jitter_max_secs: u64,
    pub ad_domains: Vec<String>,
    pub ad_intox_regex: String,
    pub ad_intox_max_depth: usize,
    pub toxic_threshold: usize,
    pub deep_crawl_max_pages: usize,
    /// How to open local HTML files: "edit" (text viewer, default) or "render".
    pub local_html_default: String,
    pub resolver_order: Vec<String>,
    pub handshake_enabled: bool,
    pub tor_enabled: bool,
    pub tor_route_all: bool,
    pub i2p_enabled: bool,
    pub i2p_route_all: bool,
    pub protocol_stacking: bool,
    pub overlay_default_transport: String,
    pub guilt_trip_enabled: bool,
    pub guilt_trip_opacity: f64,
    pub guilt_trip_threshold: usize,
    pub guilt_trip_nsfw_rules: Vec<String>,
    pub guilt_trip_news_rules: Vec<String>,
    pub guilt_trip_shopping_rules: Vec<String>,
    pub guilt_trip_social_rules: Vec<String>,
    pub tab_inactivity_ttl: usize,
    pub last_tab_nuke_action: String,
    pub ai_slop_detection_enabled: bool,
    pub ai_slop_phrases: Vec<String>,
    /// User-Agent spoof mode: "honest" (default, Honest Banana UA) or "rotate_daily".
    pub ua_spoof_mode: String,
    /// Destination folder for permanent downloads (defaults to ~/Downloads if empty).
    pub permanent_download_dir: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            search_engines: vec![
                SearchEngineRule {
                    name: "Google".to_string(),
                    domain_regex: r"google\.[a-z]{2,3}/search".to_string(),
                    query_params: vec!["q".to_string(), "oq".to_string()],
                },
                SearchEngineRule {
                    name: "DuckDuckGo".to_string(),
                    domain_regex: r"duckduckgo\.com".to_string(),
                    query_params: vec!["q".to_string()],
                },
                SearchEngineRule {
                    name: "Bing".to_string(),
                    domain_regex: r"bing\.com/search".to_string(),
                    query_params: vec!["q".to_string()],
                },
            ],
            rss_sources: vec![
                RssSource {
                    name: "Hacker News".to_string(),
                    url: "https://news.ycombinator.com/rss".to_string(),
                },
                RssSource {
                    name: "BBC News".to_string(),
                    url: "http://feeds.bbci.co.uk/news/rss.xml".to_string(),
                },
            ],
            max_concurrent_searches: 2,
            min_delay_ms: 500,
            max_delay_ms: 3000,
            noise_queries_amount: 20,
            first_launch_epoch: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ad_click_probability: 0.15,
            ad_jitter_min_secs: 5,
            ad_jitter_max_secs: 15,
            ad_intox_regex: r"(\b|[-_])ad(s)?(\b|[-_])|adpos|adblk|robapaginas|banner|publicidad|advertisement|anuncio|sponsored|patrocinado|google_ads".to_string(),
            ad_intox_max_depth: 5,
            ad_domains: vec![
                "doubleclick.net".to_string(),
                "googleads.g.doubleclick.net".to_string(),
                "googlesyndication.com".to_string(),
                "adnxs.com".to_string(),
                "adservice.google".to_string(),
                "amazon-adsystem.com".to_string(),
                "criteo.com".to_string(),
                "criteo.net".to_string(),
                "rubiconproject.com".to_string(),
                "pubmatic.com".to_string(),
                "openx.net".to_string(),
                "outbrain.com".to_string(),
                "taboola.com".to_string(),
                "casalemedia.com".to_string(),
                "popads.net".to_string(),
                "propellerads.com".to_string(),
                "bidswitch.net".to_string(),
                "smartadserver.com".to_string(),
                "triplelift.com".to_string(),
                "adroll.com".to_string(),
                "adcolony.com".to_string(),
                "flurry.com".to_string(),
                "petametric.com".to_string(),
            ],
            toxic_threshold: 5,
            deep_crawl_max_pages: 25,
            local_html_default: "edit".to_string(),
            resolver_order: vec![
                "Handshake".to_string(),
                "Onion".to_string(),
                "I2P".to_string(),
                "System".to_string(),
            ],
            handshake_enabled: true,
            tor_enabled: false,
            tor_route_all: false,
            i2p_enabled: false,
            i2p_route_all: false,
            protocol_stacking: false,
            overlay_default_transport: "tor".to_string(),
            guilt_trip_enabled: false,
            guilt_trip_opacity: 0.015,
            guilt_trip_threshold: 10,
            guilt_trip_nsfw_rules: vec![
                "porn".to_string(), "xxx".to_string(), "redtube".to_string(),
                "xvideos".to_string(), "pornhub".to_string(), "hentai".to_string(),
                "nsfw".to_string(), "erotic".to_string(), "sex".to_string(),
                "chaturbate".to_string(), "onlyfans".to_string()
            ],
            guilt_trip_news_rules: vec![
                "news".to_string(), "nytimes".to_string(), "cnn".to_string(),
                "elpais".to_string(), "reuters".to_string(), "bbc".to_string(),
                "guardian".to_string(), "huffpost".to_string(), "dailymail".to_string(),
                "foxnews".to_string(), "msnbc".to_string(), "larazon".to_string(),
                "lavozdegalicia".to_string(), "elmundo".to_string(), "marca".to_string()
            ],
            guilt_trip_shopping_rules: vec![
                "shop".to_string(), "amazon".to_string(), "ebay".to_string(),
                "aliexpress".to_string(), "gamble".to_string(), "casino".to_string(),
                "bet".to_string(), "target".to_string(), "walmart".to_string(),
                "bestbuy".to_string(), "poker".to_string(), "lottery".to_string()
            ],
            guilt_trip_social_rules: vec![
                "twitter.com".to_string(), "x.com".to_string(), "reddit.com".to_string(),
                "facebook.com".to_string(), "instagram.com".to_string(), "tiktok.com".to_string(),
                "linkedin.com".to_string(), "pinterest.com".to_string()
            ],
            tab_inactivity_ttl: 15,
            last_tab_nuke_action: "survive".to_string(),
            ai_slop_detection_enabled: true,
            ai_slop_phrases: vec![
                "written with ai".to_string(),
                "generated with ai".to_string(),
                "generated by ai".to_string(),
                "created with ai".to_string(),
                "assisted by ai".to_string(),
                "ai-generated".to_string(),
                "ai generated".to_string(),
                "this article was written using ai".to_string(),
                "language model".to_string(),
                "as an ai language model".to_string(),
            ],
            ua_spoof_mode: "honest".to_string(),
            permanent_download_dir: String::new(),
        }
    }
}

impl AppConfig {
    pub fn resolved_permanent_download_dir(&self) -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        let dir_str = self.permanent_download_dir.trim();
        if dir_str.is_empty() {
            std::path::Path::new(&home).join("Downloads")
        } else if let Some(stripped) = dir_str.strip_prefix("~/") {
            std::path::Path::new(&home).join(stripped)
        } else if dir_str == "~" {
            std::path::PathBuf::from(&home)
        } else {
            std::path::PathBuf::from(dir_str)
        }
    }

    pub fn active_user_agent(&self) -> String {
        if self.ua_spoof_mode == "rotate_daily" {
            let days_since_epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                / 86400;
            let index = (days_since_epoch as usize) % GENUINE_UA_POOL.len();
            GENUINE_UA_POOL[index].to_string()
        } else {
            HONEST_USER_AGENT.to_string()
        }
    }

    pub fn expected_secret_id(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let hostname = gethostname::gethostname().to_string_lossy().to_string();
        let payload = format!("{}-{}", hostname, self.first_launch_epoch);
        hasher.update(payload);
        let hash = hasher.finalize();
        hash.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
            });
        let mut path = base;
        path.push("juanita-banana");
        fs::create_dir_all(&path).ok();
        path.push("config.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        let mut cfg = if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        };
        // If "Onion" is missing from the resolver order (e.g. legacy config),
        // automatically inject it after Handshake or before System, and persist.
        if !cfg.resolver_order.iter().any(|r| r == "Onion") {
            if let Some(pos) = cfg.resolver_order.iter().position(|r| r == "Handshake") {
                cfg.resolver_order.insert(pos + 1, "Onion".to_string());
            } else if let Some(pos) = cfg.resolver_order.iter().position(|r| r == "System") {
                cfg.resolver_order.insert(pos, "Onion".to_string());
            } else {
                cfg.resolver_order.push("Onion".to_string());
            }
            cfg.save();
        }
        if !cfg.resolver_order.iter().any(|r| r == "I2P") {
            if let Some(pos) = cfg.resolver_order.iter().position(|r| r == "Onion") {
                cfg.resolver_order.insert(pos + 1, "I2P".to_string());
            } else if let Some(pos) = cfg.resolver_order.iter().position(|r| r == "Handshake") {
                cfg.resolver_order.insert(pos + 1, "I2P".to_string());
            } else if let Some(pos) = cfg.resolver_order.iter().position(|r| r == "System") {
                cfg.resolver_order.insert(pos, "I2P".to_string());
            } else {
                cfg.resolver_order.push("I2P".to_string());
            }
            cfg.save();
        }
        cfg
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            fs::write(path, data).ok();
        }
    }
}

pub fn is_default_browser() -> bool {
    let exe_path =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("juanita-banana"));
    let is_system_install = exe_path.starts_with("/usr/");
    let desktop_filename = if is_system_install {
        "juanita-banana.desktop"
    } else {
        "juanita-banana-local.desktop"
    };

    if let Ok(output) = std::process::Command::new("xdg-settings")
        .arg("check")
        .arg("default-web-browser")
        .arg(desktop_filename)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.trim() == "yes"
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.search_engines.len(), 3);
        assert_eq!(config.search_engines[0].name, "Google");
        assert_eq!(config.rss_sources.len(), 2);
        assert_eq!(config.max_concurrent_searches, 2);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(
            config.max_concurrent_searches,
            deserialized.max_concurrent_searches
        );
        assert_eq!(
            config.search_engines.len(),
            deserialized.search_engines.len()
        );
        assert_eq!(config.rss_sources[0].url, deserialized.rss_sources[0].url);
    }

    #[test]
    fn test_tab_inactivity_settings() {
        let config = AppConfig::default();
        assert_eq!(config.tab_inactivity_ttl, 15);
        assert_eq!(config.last_tab_nuke_action, "survive");

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tab_inactivity_ttl, 15);
        assert_eq!(deserialized.last_tab_nuke_action, "survive");
    }

    #[test]
    fn test_ua_spoof_mode() {
        let mut config = AppConfig::default();
        assert_eq!(config.ua_spoof_mode, "honest");
        assert_eq!(config.active_user_agent(), HONEST_USER_AGENT);

        config.ua_spoof_mode = "rotate_daily".to_string();
        let rotated_ua = config.active_user_agent();
        assert!(GENUINE_UA_POOL.contains(&rotated_ua.as_str()));
    }

    #[test]
    fn test_resolved_permanent_download_dir() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());

        let default_config = AppConfig::default();
        assert_eq!(
            default_config.resolved_permanent_download_dir(),
            std::path::Path::new(&home).join("Downloads")
        );

        let tilde_config = AppConfig {
            permanent_download_dir: "~/SecureVault".to_string(),
            ..AppConfig::default()
        };
        assert_eq!(
            tilde_config.resolved_permanent_download_dir(),
            std::path::Path::new(&home).join("SecureVault")
        );

        let custom_config = AppConfig {
            permanent_download_dir: "/var/tmp/downloads".to_string(),
            ..AppConfig::default()
        };
        assert_eq!(
            custom_config.resolved_permanent_download_dir(),
            std::path::PathBuf::from("/var/tmp/downloads")
        );
    }
}
