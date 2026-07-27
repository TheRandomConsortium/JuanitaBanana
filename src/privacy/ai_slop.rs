use crate::util::config::AppConfig;

pub fn ai_slop_detector_script(config: &AppConfig) -> String {
    if !config.ai_slop_detection_enabled {
        return String::new();
    }

    let phrases_json =
        serde_json::to_string(&config.ai_slop_phrases).unwrap_or_else(|_| "[]".to_string());
    let tokens_css = include_root_str!(@templates, "tokens.css");

    include_root_str!(@scripts, "ai_slop.js")
        .replace("AI_PHRASES_PLACEHOLDER", &phrases_json)
        .replace("TOKENS_CSS_PLACEHOLDER", tokens_css)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_slop_detector_script_generation() {
        let config = AppConfig {
            ai_slop_detection_enabled: true,
            ai_slop_phrases: vec!["written with ai".to_string()],
            ..Default::default()
        };

        let script = ai_slop_detector_script(&config);
        assert!(script.contains("written with ai"));
        assert!(script.contains("juanita://ai-alternatives"));
        assert!(script.contains("--jb-accent-yellow"));

        let mut config_disabled = config;
        config_disabled.ai_slop_detection_enabled = false;
        let script_disabled = ai_slop_detector_script(&config_disabled);
        assert!(script_disabled.is_empty());
    }
}
