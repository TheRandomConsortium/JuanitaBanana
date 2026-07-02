use std::process::Command;

pub struct Competitor {
    pub name: String,
    pub icon: String,
    pub desktop_file: String,
}

pub fn get_competitors() -> Vec<Competitor> {
    let mut competitors = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Ok(output) = Command::new("gio")
        .arg("mime")
        .arg("x-scheme-handler/http")
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.ends_with(".desktop") && !line.contains("Aplicación predeterminada") {
                let desktop = if let Some(idx) = line.rfind(' ') {
                    &line[idx + 1..]
                } else {
                    line
                };
                let desktop = desktop.trim();

                if desktop == "juanita-banana.desktop" || desktop == "juanita-banana-local.desktop"
                {
                    continue;
                }

                if !seen.insert(desktop.to_string()) {
                    continue;
                }

                let paths = [
                    format!("/usr/share/applications/{}", desktop),
                    format!(
                        "{}/.local/share/applications/{}",
                        std::env::var("HOME").unwrap_or_default(),
                        desktop
                    ),
                ];

                for path in &paths {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        let mut name = String::new();
                        let mut icon = String::new();
                        for l in content.lines() {
                            if l.starts_with("Name=") && name.is_empty() {
                                name = l.trim_start_matches("Name=").to_string();
                            }
                            if l.starts_with("Icon=") && icon.is_empty() {
                                icon = l.trim_start_matches("Icon=").to_string();
                            }
                        }
                        if !name.is_empty() {
                            competitors.push(Competitor {
                                name,
                                icon,
                                desktop_file: desktop.to_string(),
                            });
                        }
                        break;
                    }
                }
            }
        }
    }
    competitors
}

pub fn competitors_page_html() -> String {
    let competitors = get_competitors();
    let mut grid_html = String::new();

    for c in competitors {
        let mut img_src = String::from("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><text y='.9em' font-size='90'>🤮</text></svg>");

        let icon_paths = [
            format!("/usr/share/pixmaps/{}.png", c.icon),
            format!("/usr/share/icons/hicolor/48x48/apps/{}.png", c.icon),
            format!("/usr/share/icons/hicolor/scalable/apps/{}.svg", c.icon),
        ];

        for p in &icon_paths {
            if let Ok(bytes) = std::fs::read(p) {
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let mime = if p.ends_with(".svg") {
                    "image/svg+xml"
                } else {
                    "image/png"
                };
                img_src = format!("data:{};base64,{}", mime, b64);
                break;
            }
        }

        grid_html.push_str(&format!(r#"
            <div class="competitor-card" onclick="window.location.href='juanita://set-competitor?desktop={}'">
                <img class="icon" src="{}" alt="{}" />
                <div class="name">{}</div>
            </div>
        "#, c.desktop_file, img_src, c.name, c.name));
    }

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Choose Competitor</title>
    <style>
        body {{
            background: #1e1e1e;
            color: #d4d4d4;
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
        }}
        h1 {{
            color: #ffcc00;
            margin-bottom: 10px;
        }}
        p {{
            font-size: 1.2em;
            color: #aaa;
            margin-bottom: 40px;
        }}
        .grid {{
            display: flex;
            gap: 20px;
            flex-wrap: wrap;
            justify-content: center;
            max-width: 800px;
        }}
        .competitor-card {{
            background: #252526;
            border: 1px solid #333;
            border-radius: 10px;
            padding: 30px;
            cursor: pointer;
            text-align: center;
            transition: transform 0.2s, background 0.2s;
            width: 150px;
        }}
        .competitor-card:hover {{
            background: #2d2d30;
            transform: scale(1.05);
            border-color: #666;
        }}
        .icon {{
            width: 64px;
            height: 64px;
            margin-bottom: 15px;
            object-fit: contain;
        }}
        .name {{
            font-size: 1.1em;
            font-weight: bold;
        }}
        .back-btn {{
            margin-top: 50px;
            padding: 10px 20px;
            background: #007acc;
            color: white;
            border: none;
            border-radius: 5px;
            cursor: pointer;
            font-size: 1em;
        }}
        .back-btn:hover {{
            background: #0098ff;
        }}
    </style>
</head>
<body>
    <h1>Betrayal makes bananas go brown 🍌🍂</h1>
    <p>Choose your new master:</p>
    
    <div class="grid">
        {}
    </div>

    <button class="back-btn" onclick="window.location.href='juanita://config'">I changed my mind, take me back</button>
</body>
</html>
    "#,
        grid_html
    )
}
