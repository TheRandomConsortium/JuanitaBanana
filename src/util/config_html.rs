use crate::util::config::AppConfig;

pub fn config_page_html(config: &AppConfig, is_default: bool) -> String {
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

    let max_concurrent = config.max_concurrent_searches;
    let min_delay = config.min_delay_ms;
    let max_delay = config.max_delay_ms;
    let noise_amount = config.noise_queries_amount;
    let click_prob = config.ad_click_probability;
    let jitter_min = config.ad_jitter_min_secs;
    let jitter_max = config.ad_jitter_max_secs;
    let intox_max_depth = config.ad_intox_max_depth;
    let intox_regex = config.ad_intox_regex.replace('"', "&quot;");
    let toxic_threshold = config.toxic_threshold;

    let default_btn = if is_default {
        r#"<button disabled style="background: #444; color: #888; cursor: not-allowed;">Already Default Browser</button>
           <button style="margin-left: 10px;" onclick="window.location.href='juanita://choose-competitor'">Choose Competitor</button>"#
    } else {
        r#"<button onclick="if(confirm('Really this piece of shit?')) { window.location.href='juanita://make-default'; }">Make Default Browser</button>"#
    };

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Juanita Banana - Config</title>
    <style>
        body {{
            background: #1e1e1e;
            color: #d4d4d4;
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            display: flex;
            height: 100vh;
        }}
        #sidebar {{
            width: 250px;
            background: #252526;
            padding: 20px 0;
            border-right: 1px solid #333;
        }}
        #sidebar ul {{
            list-style: none;
            padding: 0;
            margin: 0;
        }}
        #sidebar li {{
            padding: 15px 20px;
            cursor: pointer;
            border-bottom: 1px solid #333;
        }}
        #sidebar li:hover, #sidebar li.active {{
            background: #37373d;
        }}
        #content {{
            flex: 1;
            padding: 40px;
            overflow-y: auto;
        }}
        .tab-content {{
            display: none;
        }}
        .tab-content.active {{
            display: block;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
        }}
        th, td {{
            text-align: left;
            padding: 12px;
            border-bottom: 1px solid #444;
        }}
        th {{
            background-color: #2d2d30;
        }}
        h2 {{
            border-bottom: 2px solid #007acc;
            padding-bottom: 10px;
            margin-top: 0;
        }}
        button {{
            background: #007acc;
            color: white;
            border: none;
            padding: 10px 20px;
            cursor: pointer;
            border-radius: 3px;
            margin-top: 20px;
        }}
        button:hover {{
            background: #0098ff;
        }}
        /* The unban section is completely hidden from sidebar */
        #unban-section {{ display: none; }}
    </style>
</head>
<body>
    <div id="sidebar">
        <ul>
            <li id="li-general" class="active" onclick="showTab('general')">General</li>
            <li id="li-intoxication" onclick="showTab('intoxication')">Search Intoxication</li>
            <li id="li-ad-intox" onclick="showTab('ad-intox')">Ad Intoxication</li>
            <li id="li-rss" onclick="showTab('rss')">RSS Sources</li>
        </ul>
    </div>
    <div id="content">
        <div id="general" class="tab-content active">
            <h2>General Settings</h2>
            <div style="margin-top: 20px;">
                <p>Make Juanita Banana your default browser to open links from other apps automatically.</p>
                {default_btn}
            </div>
        </div>
        <div id="intoxication" class="tab-content">
            <div style="margin-bottom: 30px; padding: 15px; background: #2d2d30; border-radius: 5px;">
                <h3 style="margin-top: 0; display: inline-block;">Intoxication Rules</h3>
                <span title="⚠️ WARNING: Modifying these values can affect fingerprinting evasion, memory consumption, and make you wait unnecessarily." style="cursor: help; margin-left: 10px; font-size: 1.2em;">⚠️</span>
                <div style="margin-top: 10px;">
                    <label style="display: inline-block; width: 220px;">Max Concurrent Searches:</label>
                    <input type="number" id="max-concurrent" value="{max_concurrent}" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444;"><br>
                    
                    <label style="display: inline-block; width: 220px; margin-top: 10px;">Min Camouflage Delay (ms):</label>
                    <input type="number" id="min-delay" value="{min_delay}" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444;"><br>
                    
                    <label style="display: inline-block; width: 220px; margin-top: 10px;">Max Camouflage Delay (ms):</label>
                    <input type="number" id="max-delay" value="{max_delay}" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444;">

                    <label style="display: inline-block; width: 220px; margin-top: 10px;">Hidden Queries Amount:</label>
                    <input type="number" id="noise-amount" value="{noise_amount}" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444;">
                </div>
            </div>

            <h2>Search Engine Rules</h2>
            <p>Define which search queries to intercept and poison.</p>
            <table>
                <thead>
                    <tr><th>Name</th><th>Domain Regex</th><th>Query Params (comma-sep)</th><th>Action</th></tr>
                </thead>
                <tbody id="engines-tbody">
                    {engines_html}
                </tbody>
            </table>
            <div style="margin-top: 15px;">
                <input type="text" id="new-engine-name" placeholder="Name" style="padding: 5px; width: 100px;">
                <input type="text" id="new-engine-regex" placeholder="Regex" style="padding: 5px; width: 150px;">
                <input type="text" id="new-engine-params" placeholder="Params (e.g. q,oq)" style="padding: 5px; width: 150px;">
                <button onclick="addEngine()" style="margin-top: 0; padding: 6px 15px;">Add</button>
            </div>
        </div>

        <div id="ad-intox" class="tab-content">
            <h2>Ad Intoxication Settings</h2>
             <div style="margin-bottom: 30px; padding: 15px; background: #2d2d30; border-radius: 5px;">
                <label style="display: inline-block; width: 250px;">Ad Click Probability (0.0 to 1.0):</label>
                <input type="number" step="0.01" min="0" max="1" id="ad-click-prob" value="{click_prob}" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444;"><br>
                
                <label style="display: inline-block; width: 250px; margin-top: 10px;">Min Jitter Delay (seconds):</label>
                <input type="number" id="ad-jitter-min" value="{jitter_min}" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444;"><br>
                
                <label style="display: inline-block; width: 250px; margin-top: 10px;">Max Jitter Delay (seconds):</label>
                <input type="number" id="ad-jitter-max" value="{jitter_max}" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444;"><br>

                <label style="display: inline-block; width: 250px; margin-top: 10px;">Max Surgery Depth:</label>
                <input type="number" id="ad-intox-max-depth" value="{intox_max_depth}" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444;"><br>
                
                <label style="display: inline-block; width: 250px; margin-top: 10px;">Ad Surgery Regex Pattern:</label>
                <input type="text" id="ad-intox-regex" value="{intox_regex}" style="width: 350px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444;"><br>

                <label style="display: inline-block; width: 250px; margin-top: 10px;">Toxic Warning Threshold:</label>
                <input type="number" id="toxic-threshold" value="{toxic_threshold}" style="width: 80px; padding: 5px; background: #1e1e1e; color: #fff; border: 1px solid #444;">
            </div>

            <h2>Learned Ad Domains</h2>
            <p>Domains checked against resources and link hrefs to detect ad intents.</p>
            <table>
                <thead>
                    <tr><th>Domain</th><th>Action</th></tr>
                </thead>
                <tbody id="ad-domains-tbody">
                    {ad_domains_html}
                </tbody>
            </table>
            <div style="margin-top: 15px;">
                <input type="text" id="new-ad-domain" placeholder="Domain (e.g. tracking.com)" style="padding: 5px; width: 300px;">
                <button onclick="addAdDomain()" style="margin-top: 0; padding: 6px 15px;">Add</button>
            </div>
        </div>

        <div id="rss" class="tab-content">
            <h2>Noise Sources (RSS)</h2>
            <p>RSS feeds used to pull real, trending search queries.</p>
            <table>
                <thead>
                    <tr><th>Name</th><th>URL</th><th>Action</th></tr>
                </thead>
                <tbody id="rss-tbody">
                    {rss_html}
                </tbody>
            </table>
            <div style="margin-top: 15px;">
                <input type="text" id="new-rss-name" placeholder="Feed Name (e.g. El Pais)" style="padding: 5px;">
                <input type="text" id="new-rss-url" placeholder="RSS URL" style="padding: 5px; width: 300px;">
                <button onclick="addRss()" style="margin-top: 0; padding: 6px 15px;">Add</button>
            </div>
        </div>

        <div id="unban" class="tab-content">
            <h2>Improbable Redemption</h2>
            <p>Solve the following equation to unban a site: ∫ e^x dx from 0 to ln(3).</p>
            <input type="text" id="math-answer" placeholder="Answer..." style="padding: 5px;">
            <button onclick="checkUnban()">Submit</button>
            <div id="unban-result"></div>
        </div>

        <button onclick="saveConfig()">Save Configuration</button>
    </div>

    <script>
        // Only show unban if the path is specifically requested via URL hash or secret
        if (window.location.hash === '#unban') {{
            showTab('unban', null);
        }}

        if (window.location.href.includes('saved=true')) {{
            const btn = document.querySelector('button[onclick="saveConfig()"]');
            if(btn) {{
                btn.innerText = "Saved!";
                setTimeout(() => {{ btn.innerText = "Save Configuration"; }}, 3000);
            }}
        }}

        function showTab(tabId) {{
            if (tabId !== 'unban' && window.location.hash === '#unban') return; // Enforce lock
            document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
            document.getElementById(tabId).classList.add('active');
            
            document.querySelectorAll('#sidebar li').forEach(el => el.classList.remove('active'));
            const activeLi = document.getElementById('li-' + tabId);
            if (activeLi) activeLi.classList.add('active');
        }}

        function checkUnban() {{
            const ans = document.getElementById('math-answer').value;
            // integral of e^x from 0 to ln(3) is e^ln(3) - e^0 = 3 - 1 = 2
            if (ans.trim() === "2") {{
                document.getElementById('unban-result').innerHTML = "<br>Correct. <a href='juanita://unban-page'>Manage Bans</a>";
            }} else {{
                document.getElementById('unban-result').innerHTML = "<br><span style='color:red;'>Incorrect. Back to the abyss.</span>";
            }}
        }}

        function saveConfig() {{
            // Build new RSS array from table
            const rssRows = document.querySelectorAll('#rss-tbody tr');
            const newRss = [];
            rssRows.forEach(row => {{
                const name = row.cells[0].textContent;
                const url = row.cells[1].textContent;
                newRss.push({{ name, url }});
            }});
            
            // Build new Engines array from table
            const enginesRows = document.querySelectorAll('#engines-tbody tr');
            const newEngines = [];
            enginesRows.forEach(row => {{
                const name = row.cells[0].textContent;
                const domain_regex = row.cells[1].textContent;
                const query_params = row.cells[2].textContent.split(',').map(s => s.trim()).filter(s => s.length > 0);
                newEngines.push({{ name, domain_regex, query_params }});
            }});

            const configData = {json_data};
            configData.rss_sources = newRss;
            configData.search_engines = newEngines;
            configData.max_concurrent_searches = parseInt(document.getElementById('max-concurrent').value, 10);
            configData.min_delay_ms = parseInt(document.getElementById('min-delay').value, 10);
            configData.max_delay_ms = parseInt(document.getElementById('max-delay').value, 10);
            configData.noise_queries_amount = parseInt(document.getElementById('noise-amount').value, 10);

            // Ad Intoxication save fields
            configData.ad_click_probability = parseFloat(document.getElementById('ad-click-prob').value);
            configData.ad_jitter_min_secs = parseInt(document.getElementById('ad-jitter-min').value, 10);
            configData.ad_jitter_max_secs = parseInt(document.getElementById('ad-jitter-max').value, 10);
            configData.ad_intox_max_depth = parseInt(document.getElementById('ad-intox-max-depth').value, 10);
            configData.ad_intox_regex = document.getElementById('ad-intox-regex').value;
            configData.toxic_threshold = parseInt(document.getElementById('toxic-threshold').value, 10);

            const adRows = document.querySelectorAll('#ad-domains-tbody tr');
            const newAdDomains = [];
            adRows.forEach(row => {{
                const domain = row.cells[0].textContent.trim();
                if (domain) newAdDomains.push(domain);
            }});
            configData.ad_domains = newAdDomains;
            
            // Send config back to rust layer via URI interception
            window.location.href = "juanita://save-config?data=" + encodeURIComponent(JSON.stringify(configData));
        }}

        function addRss() {{
            const name = document.getElementById('new-rss-name').value;
            const url = document.getElementById('new-rss-url').value;
            if (!name || !url) return;
            
            const tbody = document.getElementById('rss-tbody');
            const row = document.createElement('tr');
            row.innerHTML = `<td>${{name}}</td><td>${{url}}</td><td><button onclick="this.parentElement.parentElement.remove()" style="margin:0; padding: 5px;">X</button></td>`;
            tbody.appendChild(row);
            
            document.getElementById('new-rss-name').value = '';
            document.getElementById('new-rss-url').value = '';
        }}

        function addEngine() {{
            const name = document.getElementById('new-engine-name').value;
            const regex = document.getElementById('new-engine-regex').value;
            const params = document.getElementById('new-engine-params').value;
            if (!name || !regex || !params) return;
            
            const tbody = document.getElementById('engines-tbody');
            const row = document.createElement('tr');
            row.innerHTML = `<td>${{name}}</td><td>${{regex}}</td><td>${{params}}</td><td><button onclick="this.parentElement.parentElement.remove()" style="margin:0; padding: 5px;">X</button></td>`;
            tbody.appendChild(row);
            
            document.getElementById('new-engine-name').value = '';
            document.getElementById('new-engine-regex').value = '';
            document.getElementById('new-engine-params').value = '';
        }}

        function addAdDomain() {{
            const domain = document.getElementById('new-ad-domain').value.trim();
            if (!domain) return;
            const tbody = document.getElementById('ad-domains-tbody');
            const row = document.createElement('tr');
            row.innerHTML = `<td>${{domain}}</td><td><button onclick="this.parentElement.parentElement.remove()" style="margin:0; padding: 5px;">X</button></td>`;
            tbody.appendChild(row);
            document.getElementById('new-ad-domain').value = '';
        }}
    </script>
</body>
</html>
    "#,
        default_btn = default_btn,
        max_concurrent = max_concurrent,
        min_delay = min_delay,
        max_delay = max_delay,
        noise_amount = noise_amount,
        click_prob = click_prob,
        jitter_min = jitter_min,
        jitter_max = jitter_max,
        engines_html = engines_html,
        rss_html = rss_html,
        ad_domains_html = ad_domains_html,
        json_data = json_data,
        intox_max_depth = intox_max_depth,
        intox_regex = intox_regex,
        toxic_threshold = toxic_threshold
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::config::AppConfig;

    #[test]
    fn test_config_page_html() {
        let mut config = AppConfig::default();
        config.max_concurrent_searches = 777;
        config.toxic_threshold = 999;

        let html = config_page_html(&config, false);
        assert!(html.contains("777"));
        assert!(html.contains("999"));
        assert!(html.contains("Hacker News"));
        assert!(html.contains("DuckDuckGo"));
    }
}
