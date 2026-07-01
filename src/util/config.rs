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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub search_engines: Vec<SearchEngineRule>,
    pub rss_sources: Vec<RssSource>,
    pub max_concurrent_searches: usize,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub first_launch_epoch: u64,
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
            first_launch_epoch: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

impl AppConfig {
    pub fn expected_secret_id(&self) -> String {
        use sha2::{Sha256, Digest};
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
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            fs::write(path, data).ok();
        }
    }
}

pub fn config_page_html(config: &AppConfig) -> String {
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

    let json_data = serde_json::to_string(&config).unwrap_or_default();

    let max_concurrent = config.max_concurrent_searches;
    let min_delay = config.min_delay_ms;
    let max_delay = config.max_delay_ms;

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
            <li class="active" onclick="showTab('intoxication')">Search Intoxication</li>
            <li onclick="showTab('rss')">RSS Sources</li>
        </ul>
    </div>
    <div id="content">
        <div id="intoxication" class="tab-content active">
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
            showTab('unban');
        }}

        function showTab(tabId) {{
            if (tabId !== 'unban' && window.location.hash === '#unban') return; // Enforce lock
            document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
            document.getElementById(tabId).classList.add('active');
            
            document.querySelectorAll('#sidebar li').forEach(el => el.classList.remove('active'));
            const activeLi = Array.from(document.querySelectorAll('#sidebar li')).find(li => li.innerText.toLowerCase().includes(tabId));
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
                const name = row.cells[0].innerText;
                const url = row.cells[1].innerText;
                newRss.push({{ name, url }});
            }});
            
            // Build new Engines array from table
            const enginesRows = document.querySelectorAll('#engines-tbody tr');
            const newEngines = [];
            enginesRows.forEach(row => {{
                const name = row.cells[0].innerText;
                const domain_regex = row.cells[1].innerText;
                const query_params = row.cells[2].innerText.split(',').map(s => s.trim()).filter(s => s.length > 0);
                newEngines.push({{ name, domain_regex, query_params }});
            }});

            const configData = {json_data};
            configData.rss_sources = newRss;
            configData.search_engines = newEngines;
            configData.max_concurrent_searches = parseInt(document.getElementById('max-concurrent').value, 10);
            configData.min_delay_ms = parseInt(document.getElementById('min-delay').value, 10);
            configData.max_delay_ms = parseInt(document.getElementById('max-delay').value, 10);
            
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
    </script>
</body>
</html>
    "#
    )
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
    fn test_config_page_html() {
        let mut config = AppConfig::default();
        config.max_concurrent_searches = 777;

        let html = config_page_html(&config);

        // Ensure our unique value is in the HTML
        assert!(html.contains("777"));
        assert!(html.contains("Hacker News"));
        assert!(html.contains("DuckDuckGo"));
    }
}
