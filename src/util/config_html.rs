use crate::util::config::AppConfig;

pub struct DecryptedSecureData {
    pub master_pass: String,
    pub name: String,
    pub id: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub pop_server: String,
    pub pop_port: u16,
    pub pop_user: String,
    pub pop_pass: String,
}

pub fn config_page_html(
    config: &AppConfig,
    is_default: bool,
    decrypted_data: Option<&DecryptedSecureData>,
    unlock_error: bool,
) -> String {
    let mut engines_html = String::new();
    for engine in &config.search_engines {
        engines_html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td><button onclick=\"this.parentElement.parentElement.remove()\" style=\"margin:0; padding: 5px;\">X</button></td></tr>",
            engine.name, engine.domain_regex, engine.query_params.join(", ")
        ));
    }

    let mut rss_html = String::new();
    for rss in &config.rss_sources {
        rss_html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td><button onclick=\"this.parentElement.parentElement.remove()\" style=\"margin:0; padding: 5px;\">X</button></td></tr>",
            rss.name, rss.url
        ));
    }

    let mut ad_domains_html = String::new();
    for dom in &config.ad_domains {
        ad_domains_html.push_str(&format!(
            "<tr><td>{}</td><td><button onclick=\"this.parentElement.parentElement.remove()\" style=\"margin:0; padding: 5px;\">X</button></td></tr>",
            dom
        ));
    }

    let json_data = serde_json::to_string(&config).unwrap_or_default();

    let mut resolver_list_html = String::new();
    for res in &config.resolver_order {
        resolver_list_html.push_str(&format!(
            r#"<li class="resolver-item" data-name="{name}" style="padding: 12px; margin: 8px 0; background: #37373d; border-radius: 4px; display: flex; align-items: center; justify-content: space-between; border: 1px solid #444;">
                <span style="font-weight: bold; color: #fff;">{name}</span>
                <div>
                    <button type="button" onclick="moveResolverUp(this)" style="margin: 0; padding: 4px 10px; font-size: 0.85em; background: #444; border: 1px solid #555; border-radius: 3px; color: #fff; cursor: pointer;">Up</button>
                    <button type="button" onclick="moveResolverDown(this)" style="margin: 0; padding: 4px 10px; font-size: 0.85em; background: #444; border: 1px solid #555; border-radius: 3px; color: #fff; cursor: pointer; margin-left: 5px;">Down</button>
                </div>
            </li>"#,
            name = res
        ));
    }

    let max_concurrent = config.max_concurrent_searches.to_string();
    let min_delay = config.min_delay_ms.to_string();
    let max_delay = config.max_delay_ms.to_string();
    let noise_amount = config.noise_queries_amount.to_string();
    let click_prob = config.ad_click_probability.to_string();
    let jitter_min = config.ad_jitter_min_secs.to_string();
    let jitter_max = config.ad_jitter_max_secs.to_string();
    let intox_max_depth = config.ad_intox_max_depth.to_string();
    let intox_regex = config.ad_intox_regex.replace('"', "&quot;");
    let toxic_threshold = config.toxic_threshold.to_string();
    let deep_crawl_max_pages = config.deep_crawl_max_pages.to_string();

    let default_btn = if is_default {
        r#"<button disabled style="background: #444; color: #888; cursor: not-allowed;">Already Default Browser</button>
           <button style="margin-left: 10px;" onclick="window.location.href='juanita://choose-competitor'">Choose Competitor</button>"#
    } else {
        r#"<button onclick="if(confirm('Really this piece of shit?')) { window.location.href='juanita://make-default'; }">Make Default Browser</button>"#
    };

    let db_exists = crate::unsubscribe::db::SecureDbManager::exists();

    let mut secure_db_html = String::new();
    if let Some(sec) = decrypted_data {
        secure_db_html.push_str(&format!(
            r#"
            <p style="color: #4caf50; font-weight: bold;">🔓 Secure Database Unlocked successfully.</p>
            <div class="jb-card">
                <h3 class="jb-card-item-title">GDPR Complainant Profile</h3>
                <label class="jb-label" style="display: inline-block; width: 220px;">Full Name:</label>
                <input type="text" id="secure-name" value="{name}" class="jb-input" style="width: 250px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">National ID / Passport:</label>
                <input type="text" id="secure-id" value="{id}" class="jb-input" style="width: 250px; margin-bottom: 20px;"><br>

                <h3 class="jb-card-item-title">SMTP Configuration (Outgoing)</h3>
                <label class="jb-label" style="display: inline-block; width: 220px;">SMTP Server:</label>
                <input type="text" id="secure-smtp-server" value="{smtp_server}" class="jb-input" style="width: 250px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">SMTP Port:</label>
                <input type="number" id="secure-smtp-port" value="{smtp_port}" class="jb-input" style="width: 80px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">SMTP Username:</label>
                <input type="text" id="secure-smtp-user" value="{smtp_user}" class="jb-input" style="width: 250px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">SMTP Password:</label>
                <input type="password" id="secure-smtp-pass" value="{smtp_pass}" class="jb-input" style="width: 250px; margin-bottom: 20px;"><br>

                <h3 class="jb-card-item-title">POP3 Configuration (Incoming)</h3>
                <label class="jb-label" style="display: inline-block; width: 220px;">POP3 Server:</label>
                <input type="text" id="secure-pop-server" value="{pop_server}" class="jb-input" style="width: 250px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">POP3 Port:</label>
                <input type="number" id="secure-pop-port" value="{pop_port}" class="jb-input" style="width: 80px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">POP3 Username:</label>
                <input type="text" id="secure-pop-user" value="{pop_user}" class="jb-input" style="width: 250px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">POP3 Password:</label>
                <input type="password" id="secure-pop-pass" value="{pop_pass}" class="jb-input" style="width: 250px; margin-bottom: 20px;"><br>

                <input type="hidden" id="secure-pass" value="{master_pass}">
                <button class="jb-btn jb-btn-primary" onclick="saveSecureConfig(false)">Save Secure Settings</button>
            </div>
            "#,
            name = sec.name,
            id = sec.id,
            smtp_server = sec.smtp_server,
            smtp_port = sec.smtp_port,
            smtp_user = sec.smtp_user,
            smtp_pass = sec.smtp_pass,
            pop_server = sec.pop_server,
            pop_port = sec.pop_port,
            pop_user = sec.pop_user,
            pop_pass = sec.pop_pass,
            master_pass = sec.master_pass
        ));
    } else if db_exists {
        let err_msg = if unlock_error {
            r#"<p style="color: #f44336; margin-bottom: 15px; font-weight: bold;">❌ Invalid Master Password. Please try again.</p>"#
        } else {
            ""
        };
        secure_db_html.push_str(&format!(
            r#"
            <p class="jb-text-secondary">Your Secure Database is currently encrypted. Please enter your Master Password to unlock and edit these settings.</p>
            {err_msg}
            <div class="jb-card" style="max-width: 450px;">
                <label class="jb-label" style="display: block; margin-bottom: 10px;">Master Password:</label>
                <input type="password" id="secure-unlock-pass" class="jb-input" style="width: 100%; box-sizing: border-box; margin-bottom: 15px;">
                <button class="jb-btn jb-btn-primary" onclick="unlockSecureDb()">Unlock Settings</button>
            </div>
            "#,
            err_msg = err_msg
        ));
    } else {
        secure_db_html.push_str(
            r#"
            <p style="color: #ff9800; font-weight: bold;">⚠️ Secure Database is not enabled.</p>
            <p class="jb-text-secondary">To store your personal details (necessary for GDPR erasure complaints) and email configurations encrypted locally, fill out the form below to initialize the Secure Database:</p>
            <div class="jb-card">
                <h3 class="jb-card-item-title">Initialize Secure Database</h3>
                <label class="jb-label" style="display: inline-block; width: 220px;">Create Master Password:</label>
                <input type="password" id="secure-init-pass" placeholder="Enter password..." class="jb-input" style="width: 250px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">Full Name:</label>
                <input type="text" id="secure-name" placeholder="Juan Perez" class="jb-input" style="width: 250px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">National ID / Passport:</label>
                <input type="text" id="secure-id" placeholder="12345678X" class="jb-input" style="width: 250px; margin-bottom: 20px;"><br>

                <h3 class="jb-card-item-title">SMTP Configuration (Outgoing)</h3>
                <label class="jb-label" style="display: inline-block; width: 220px;">SMTP Server:</label>
                <input type="text" id="secure-smtp-server" placeholder="smtp.gmail.com" class="jb-input" style="width: 250px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">SMTP Port:</label>
                <input type="number" id="secure-smtp-port" value="587" class="jb-input" style="width: 80px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">SMTP Username:</label>
                <input type="text" id="secure-smtp-user" placeholder="user@gmail.com" class="jb-input" style="width: 250px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">SMTP Password:</label>
                <input type="password" id="secure-smtp-pass" placeholder="Password..." class="jb-input" style="width: 250px; margin-bottom: 20px;"><br>

                <h3 class="jb-card-item-title">POP3 Configuration (Incoming)</h3>
                <label class="jb-label" style="display: inline-block; width: 220px;">POP3 Server:</label>
                <input type="text" id="secure-pop-server" placeholder="pop.gmail.com" class="jb-input" style="width: 250px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">POP3 Port:</label>
                <input type="number" id="secure-pop-port" value="995" class="jb-input" style="width: 80px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">POP3 Username:</label>
                <input type="text" id="secure-pop-user" placeholder="user@gmail.com" class="jb-input" style="width: 250px; margin-bottom: 10px;"><br>

                <label class="jb-label" style="display: inline-block; width: 220px;">POP3 Password:</label>
                <input type="password" id="secure-pop-pass" placeholder="Password..." class="jb-input" style="width: 250px; margin-bottom: 20px;"><br>

                <button class="jb-btn jb-btn-primary" onclick="saveSecureConfig(true)">Initialize Secure Database</button>
            </div>
            "#
        );
    }

    let shared_css = crate::browsing::internal::SHARED_CSS.as_str();
    let html_template = include_str!("../../templates/pages/config.html");
    let js_content = include_str!("../../scripts/js/config.js");

    let handshake_enabled_checked = if config.handshake_enabled {
        "checked"
    } else {
        ""
    };

    let tor_enabled_checked = if config.tor_enabled { "checked" } else { "" };
    let tor_route_all_checked = if config.tor_route_all { "checked" } else { "" };
    // The route-all checkbox should be disabled when Tor itself is not enabled
    let tor_enabled_checked_disabled = if config.tor_enabled { "" } else { "disabled" };

    let guilt_trip_enabled_checked = if config.guilt_trip_enabled {
        "checked"
    } else {
        ""
    };

    let last_tab_action_survive_selected = if config.last_tab_nuke_action == "survive" {
        "selected"
    } else {
        ""
    };
    let last_tab_action_home_selected = if config.last_tab_nuke_action == "home" {
        "selected"
    } else {
        ""
    };

    html_template
        .replace("{shared_css}", shared_css)
        .replace("{config_js}", js_content)
        .replace("{default_btn}", default_btn)
        .replace("{max_concurrent}", &max_concurrent)
        .replace("{min_delay}", &min_delay)
        .replace("{max_delay}", &max_delay)
        .replace("{noise_amount}", &noise_amount)
        .replace("{click_prob}", &click_prob)
        .replace("{jitter_min}", &jitter_min)
        .replace("{jitter_max}", &jitter_max)
        .replace("{engines_html}", &engines_html)
        .replace("{rss_html}", &rss_html)
        .replace("{ad_domains_html}", &ad_domains_html)
        .replace("{json_data}", &json_data)
        .replace("{intox_max_depth}", &intox_max_depth)
        .replace("{intox_regex}", &intox_regex)
        .replace("{toxic_threshold}", &toxic_threshold)
        .replace("{deep_crawl_max_pages}", &deep_crawl_max_pages)
        .replace("{secure_db_html}", &secure_db_html)
        .replace("{resolver_list_html}", &resolver_list_html)
        .replace("{handshake_enabled_checked}", handshake_enabled_checked)
        .replace("{tor_enabled_checked}", tor_enabled_checked)
        .replace("{tor_route_all_checked}", tor_route_all_checked)
        .replace(
            "{tor_enabled_checked_disabled}",
            tor_enabled_checked_disabled,
        )
        .replace("{guilt_trip_enabled_checked}", guilt_trip_enabled_checked)
        .replace(
            "{guilt_trip_opacity}",
            &config.guilt_trip_opacity.to_string(),
        )
        .replace(
            "{guilt_trip_threshold}",
            &config.guilt_trip_threshold.to_string(),
        )
        .replace(
            "{guilt_trip_nsfw_rules}",
            &config.guilt_trip_nsfw_rules.join(", "),
        )
        .replace(
            "{guilt_trip_news_rules}",
            &config.guilt_trip_news_rules.join(", "),
        )
        .replace(
            "{guilt_trip_shopping_rules}",
            &config.guilt_trip_shopping_rules.join(", "),
        )
        .replace(
            "{guilt_trip_social_rules}",
            &config.guilt_trip_social_rules.join(", "),
        )
        .replace(
            "{tab_inactivity_ttl}",
            &config.tab_inactivity_ttl.to_string(),
        )
        .replace(
            "{last_tab_action_survive_selected}",
            last_tab_action_survive_selected,
        )
        .replace(
            "{last_tab_action_home_selected}",
            last_tab_action_home_selected,
        )
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let html = config_page_html(&config, false, None, false);
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
}
