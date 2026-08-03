#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::util::config::AppConfig;

    #[test]
    fn test_config_page_html() {
        let config = AppConfig {
            max_concurrent_searches: 777,
            toxic_threshold: 999,
            guilt_trip_enabled: true,
            guilt_trip_opacity: 0.088,
            guilt_trip_threshold: 42,
            guilt_trip_nsfw_rules: vec!["nsfwmeme".to_string()],
            guilt_trip_news_rules: vec!["newsmeme".to_string()],
            guilt_trip_shopping_rules: vec!["shopmeme".to_string()],
            guilt_trip_social_rules: vec!["socialmeme".to_string()],
            ..AppConfig::default()
        };

        let html = config_page_html(&config, false, None, false, None);
        assert!(html.contains("777"));
        assert!(html.contains("999"));
        assert!(html.contains("Hacker News"));
        assert!(html.contains("DuckDuckGo"));
        assert!(html.contains("0.088"));
        assert!(html.contains("42"));
        assert!(html.contains("checked"));
        assert!(html.contains("nsfwmeme"));
        assert!(html.contains("newsmeme"));
        assert!(html.contains("shopmeme"));
        assert!(html.contains("socialmeme"));
    }

    #[test]
    fn test_config_page_html_ua_spoof_mode() {
        let config_rotate = AppConfig {
            ua_spoof_mode: "rotate_daily".to_string(),
            ..AppConfig::default()
        };
        let html = config_page_html(&config_rotate, false, None, false, None);
        assert!(html.contains(r#"value="rotate_daily" selected"#));

        let config_honest = AppConfig {
            ua_spoof_mode: "honest".to_string(),
            ..AppConfig::default()
        };
        let html2 = config_page_html(&config_honest, false, None, false, None);
        assert!(html2.contains(r#"value="honest" selected"#));
    }

    #[test]
    fn test_config_page_html_permanent_download_dir() {
        let config = AppConfig {
            permanent_download_dir: "~/VaultDownloads".to_string(),
            ..AppConfig::default()
        };
        let html = config_page_html(&config, false, None, false, None);
        assert!(html.contains(r#"value="~/VaultDownloads""#));
        assert!(html.contains("Sandboxed Download Settings"));
    }

    #[test]
    fn test_config_page_html_p2p_gossip() {
        let config = AppConfig {
            allow_dht_search_sharing: true,
            rss_search_weight_percent: 70,
            contribute_own_searches: true,
            search_terms_ttl_days: 14,
            prohibited_keywords_regex: "^secret.*".to_string(),
            ..AppConfig::default()
        };
        let html = config_page_html(&config, false, None, false, None);
        assert!(html.contains("P2P Search Intoxication & Gossip Settings"));
        assert!(html.contains("id=\"allow-dht-search-sharing\" checked"));
        assert!(html.contains("70</span>% RSS / <span id=\"p2p-weight-val\">30</span>% P2P"));
        assert!(html.contains("id=\"contribute-own-searches\" checked"));
        assert!(html.contains("value=\"14\""));
        assert!(html.contains("value=\"^secret.*\""));
    }
}
