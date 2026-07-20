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
            <div class="jb-competitor-card" onclick="window.location.href='juanita://set-competitor?desktop={}'">
                <img class="jb-competitor-icon" src="{}" alt="{}" />
                <div class="jb-competitor-name">{}</div>
            </div>
        "#, c.desktop_file, img_src, c.name, c.name));
    }

    let shared_css = crate::browsing::internal::SHARED_CSS;
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Choose Competitor — Juanita Banana</title>
    <style>
        {shared_css}
    </style>
</head>
<body class="jb-page">
    <div class="jb-container jb-container-wide">
        <div class="jb-title-group">
            <h1 class="jb-title">Betrayal makes bananas go brown 🍌🍂</h1>
            <div class="jb-subtitle">Choose your new master:</div>
        </div>

        <div class="jb-competitor-grid">
            {cards}
        </div>

        <div style="margin-top: var(--jb-space-4xl);">
            <button class="jb-btn jb-btn-primary" onclick="window.location.href='juanita://config'">I changed my mind, take me back</button>
        </div>
    </div>
</body>
</html>"#,
        shared_css = shared_css,
        cards = grid_html
    )
}
