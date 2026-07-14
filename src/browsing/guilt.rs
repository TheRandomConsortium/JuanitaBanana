use crate::util::config::AppConfig;
use crate::util::image;

pub fn guilt_trip_script(config: &AppConfig) -> String {
    let opacity = config.guilt_trip_opacity.to_string();
    let threshold = config.guilt_trip_threshold.to_string();

    let ceiling_cat_b64 = image::get_ceiling_cat_b64();
    let trump_b64 = image::get_trump_b64();
    let fry_b64 = image::get_fry_b64();
    let wojak_b64 = image::get_wojak_b64();
    let banana_b64 = image::get_banana_b64();

    let nsfw_rules =
        serde_json::to_string(&config.guilt_trip_nsfw_rules).unwrap_or_else(|_| "[]".to_string());
    let news_rules =
        serde_json::to_string(&config.guilt_trip_news_rules).unwrap_or_else(|_| "[]".to_string());
    let shopping_rules = serde_json::to_string(&config.guilt_trip_shopping_rules)
        .unwrap_or_else(|_| "[]".to_string());
    let social_rules =
        serde_json::to_string(&config.guilt_trip_social_rules).unwrap_or_else(|_| "[]".to_string());

    let js_template = r#"
(function() {
    'use strict';
    
    const threshold = {threshold};
    const opacity = {opacity};
    
    let adCount = 0;
    let trackingCount = 0;
    let overlayEl = null;

    const memes = {
        nsfw: "data:image/jpeg;base64,{ceiling_cat}",
        news: "data:image/png;base64,{trump}",
        shopping: "data:image/jpeg;base64,{fry}",
        social: "data:image/jpeg;base64,{wojak}",
        fallback: "data:image/png;base64,{banana}"
    };

    function getCategory() {
        const url = window.location.href.toLowerCase();
        
        const nsfwKeywords = {nsfw_rules};
        const newsKeywords = {news_rules};
        const shoppingKeywords = {shopping_rules};
        const socialKeywords = {social_rules};

        if (nsfwKeywords.some(kw => url.includes(kw))) {
            return "nsfw";
        } else if (newsKeywords.some(kw => url.includes(kw))) {
            return "news";
        } else if (shoppingKeywords.some(kw => url.includes(kw))) {
            return "shopping";
        } else if (socialKeywords.some(kw => url.includes(kw))) {
            return "social";
        }
        return "fallback";
    }

    function createOverlay() {
        if (overlayEl) return;
        if (!document.body && !document.documentElement) return;
        
        const cat = getCategory();
        const memeUrl = memes[cat] || memes.fallback;
        console.log("[JuanitaGuilt] Creating overlay for category:", cat);
        
        overlayEl = document.createElement('div');
        overlayEl.id = 'juanita-guilt-trip-overlay';
        
        overlayEl.style.position = 'fixed';
        overlayEl.style.top = '0';
        overlayEl.style.left = '0';
        overlayEl.style.width = '100vw';
        overlayEl.style.height = '100vh';
        overlayEl.style.zIndex = '2147483647';
        overlayEl.style.pointerEvents = 'none';
        overlayEl.style.opacity = opacity;
        overlayEl.style.backgroundImage = `url("${memeUrl}")`;
        overlayEl.style.backgroundRepeat = 'no-repeat';
        
        if (cat === 'nsfw') {
            overlayEl.style.backgroundPosition = 'top center';
            overlayEl.style.backgroundSize = '300px 300px';
        } else if (cat === 'news' || cat === 'shopping') {
            overlayEl.style.backgroundPosition = 'center';
            overlayEl.style.backgroundSize = 'contain';
        } else {
            overlayEl.style.backgroundPosition = 'center';
            overlayEl.style.backgroundSize = '50% 50%';
        }
        
        (document.body || document.documentElement).appendChild(overlayEl);
    }

    function checkToxicity() {
        console.log("[JuanitaGuilt] Toxicity check. Total ads+trackers:", adCount + trackingCount, "vs Threshold:", threshold);
        if (adCount + trackingCount >= threshold) {
            createOverlay();
        }
    }

    window.addEventListener('message', (event) => {
        if (event.data && event.data.type === 'juanita_track_or_ad') {
            if (event.data.isAd) {
                adCount += event.data.count || 0;
            } else {
                trackingCount += event.data.count || 0;
            }
            checkToxicity();
        }
    });

    console.log("[JuanitaGuilt] Guilt trip script initialized. Category:", getCategory(), "Threshold:", threshold, "Opacity:", opacity);

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', checkToxicity);
    } else {
        checkToxicity();
    }
})();
"#;

    js_template
        .replace("{threshold}", &threshold)
        .replace("{opacity}", &opacity)
        .replace("{ceiling_cat}", &ceiling_cat_b64)
        .replace("{trump}", &trump_b64)
        .replace("{fry}", &fry_b64)
        .replace("{wojak}", &wojak_b64)
        .replace("{banana}", &banana_b64)
        .replace("{nsfw_rules}", &nsfw_rules)
        .replace("{news_rules}", &news_rules)
        .replace("{shopping_rules}", &shopping_rules)
        .replace("{social_rules}", &social_rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guilt_trip_script_generation() {
        let config = AppConfig {
            guilt_trip_opacity: 0.05,
            guilt_trip_threshold: 15,
            ..Default::default()
        };

        let script = guilt_trip_script(&config);
        assert!(script.contains("const threshold = 15;"));
        assert!(script.contains("const opacity = 0.05;"));
        assert!(script.contains("data:image/png;base64,"));
        assert!(script.contains("juanita-guilt-trip-overlay"));
    }
}
