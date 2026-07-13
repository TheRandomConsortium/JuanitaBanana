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
            <div style="background: #2d2d30; padding: 20px; border-radius: 5px;">
                <h3>GDPR Complainant Profile</h3>
                <label style="display: inline-block; width: 220px;">Full Name:</label>
                <input type="text" id="secure-name" value="{name}" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">National ID / Passport:</label>
                <input type="text" id="secure-id" value="{id}" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 20px;"><br>

                <h3>SMTP Configuration (Outgoing)</h3>
                <label style="display: inline-block; width: 220px;">SMTP Server:</label>
                <input type="text" id="secure-smtp-server" value="{smtp_server}" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">SMTP Port:</label>
                <input type="number" id="secure-smtp-port" value="{smtp_port}" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">SMTP Username:</label>
                <input type="text" id="secure-smtp-user" value="{smtp_user}" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">SMTP Password:</label>
                <input type="password" id="secure-smtp-pass" value="{smtp_pass}" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 20px;"><br>

                <h3>POP3 Configuration (Incoming)</h3>
                <label style="display: inline-block; width: 220px;">POP3 Server:</label>
                <input type="text" id="secure-pop-server" value="{pop_server}" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">POP3 Port:</label>
                <input type="number" id="secure-pop-port" value="{pop_port}" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">POP3 Username:</label>
                <input type="text" id="secure-pop-user" value="{pop_user}" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">POP3 Password:</label>
                <input type="password" id="secure-pop-pass" value="{pop_pass}" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 20px;"><br>

                <input type="hidden" id="secure-pass" value="{master_pass}">
                <button onclick="saveSecureConfig(false)">Save Secure Settings</button>
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
            <p>Your Secure Database is currently encrypted. Please enter your Master Password to unlock and edit these settings.</p>
            {err_msg}
            <div style="background: #2d2d30; padding: 20px; border-radius: 5px; max-width: 450px;">
                <label style="display: block; margin-bottom: 10px;">Master Password:</label>
                <input type="password" id="secure-unlock-pass" style="width: 100%; padding: 8px; background: #1e1e1e; color: #fff; border: 1px solid #444; box-sizing: border-box; margin-bottom: 15px;">
                <button onclick="unlockSecureDb()" style="margin-top: 0;">Unlock Settings</button>
            </div>
            "#,
            err_msg = err_msg
        ));
    } else {
        secure_db_html.push_str(
            r#"
            <p style="color: #ff9800; font-weight: bold;">⚠️ Secure Database is not enabled.</p>
            <p>To store your personal details (necessary for GDPR erasure complaints) and email configurations encrypted locally, fill out the form below to initialize the Secure Database:</p>
            <div style="background: #2d2d30; padding: 20px; border-radius: 5px;">
                <h3>Initialize Secure Database</h3>
                <label style="display: inline-block; width: 220px;">Create Master Password:</label>
                <input type="password" id="secure-init-pass" placeholder="Enter password..." style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">Full Name:</label>
                <input type="text" id="secure-name" placeholder="Juan Perez" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">National ID / Passport:</label>
                <input type="text" id="secure-id" placeholder="12345678X" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 20px;"><br>

                <h3>SMTP Configuration (Outgoing)</h3>
                <label style="display: inline-block; width: 220px;">SMTP Server:</label>
                <input type="text" id="secure-smtp-server" placeholder="smtp.gmail.com" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">SMTP Port:</label>
                <input type="number" id="secure-smtp-port" value="587" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">SMTP Username:</label>
                <input type="text" id="secure-smtp-user" placeholder="user@gmail.com" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">SMTP Password:</label>
                <input type="password" id="secure-smtp-pass" placeholder="App password..." style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 20px;"><br>

                <h3>POP3 Configuration (Incoming)</h3>
                <label style="display: inline-block; width: 220px;">POP3 Server:</label>
                <input type="text" id="secure-pop-server" placeholder="pop.gmail.com" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">POP3 Port:</label>
                <input type="number" id="secure-pop-port" value="995" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">POP3 Username:</label>
                <input type="text" id="secure-pop-user" placeholder="user@gmail.com" style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 10px;"><br>

                <label style="display: inline-block; width: 220px;">POP3 Password:</label>
                <input type="password" id="secure-pop-pass" placeholder="App password..." style="width: 250px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444; margin-bottom: 20px;"><br>

                <button onclick="saveSecureConfig(true)">Initialize & Save Settings</button>
            </div>
            "#
        );
    }

    let html_template = include_str!("../../templates/config.html");
    let js_content = include_str!("../../scripts/config.js");

    html_template
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
            ..AppConfig::default()
        };

        let html = config_page_html(&config, false, None, false);
        assert!(html.contains("777"));
        assert!(html.contains("999"));
        assert!(html.contains("Hacker News"));
        assert!(html.contains("DuckDuckGo"));
    }
}
