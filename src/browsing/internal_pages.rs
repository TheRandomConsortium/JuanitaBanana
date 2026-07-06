use crate::browsing::browser::BanList;
use crate::util::config::AppConfig;
use crate::util::downloads::DownloadManager;
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{WebView, WebViewExt};

pub struct PageContext {
    pub webview: WebView,
    pub downloads: Rc<RefCell<DownloadManager>>,
    pub banlist: Rc<RefCell<BanList>>,
    pub expected_unban: Rc<RefCell<Option<(String, i32)>>>,
    pub config: AppConfig,
}

pub trait InternalPage {
    fn matches_input(&self, input: &str) -> bool;
    fn handle_input(&self, input: &str, ctx: &PageContext);
    fn matches_policy(&self, uri: &str) -> bool;
    fn ignore_policy(&self, uri: &str) -> bool;
    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool;
}

pub fn get_internal_pages() -> Vec<Box<dyn InternalPage>> {
    vec![
        Box::new(HistoryPage),
        Box::new(ConfigPage),
        Box::new(ContributePage),
        Box::new(AboutPage),
        Box::new(CompetitorsPage),
        Box::new(DownloadsPage),
        Box::new(UnbanPage),
    ]
}

// 1. History Page
struct HistoryPage;
impl InternalPage for HistoryPage {
    fn matches_input(&self, input: &str) -> bool {
        input == "juanita:history" || input == "juanita://history"
    }

    fn handle_input(&self, _input: &str, ctx: &PageContext) {
        let html = include_str!("../../templates/history.html");
        ctx.webview.load_html(html, Some("juanita://history-page/"));
    }

    fn matches_policy(&self, uri: &str) -> bool {
        uri == "juanita:history" || uri == "juanita://history" || uri == "juanita://history/"
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, _uri: &str, ctx: &PageContext) -> bool {
        let html = include_str!("../../templates/history.html");
        ctx.webview.load_html(html, Some("juanita://history-page/"));
        true
    }
}

// 2. Config Page
struct ConfigPage;
impl InternalPage for ConfigPage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:config") || input.starts_with("juanita://config")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        // Match all except the config-page target to prevent loop
        (uri.starts_with("juanita://config") && !uri.starts_with("juanita://config-page"))
            || uri.starts_with("juanita://save-config")
            || uri.starts_with("juanita://make-default")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool {
        if uri.starts_with("juanita://config") {
            let is_default = crate::util::config::is_default_browser();
            let config_html = crate::util::config_html::config_page_html(&ctx.config, is_default);
            let base_uri = uri.replace("juanita://config", "juanita://config-page");
            ctx.webview.load_html(&config_html, Some(&base_uri));
            return true;
        }

        if let Some(data_str) = uri.strip_prefix("juanita://save-config?data=") {
            if let Ok(decoded) = urlencoding::decode(data_str) {
                if let Ok(new_config) = serde_json::from_str::<AppConfig>(&decoded) {
                    new_config.save();
                    println!("[CONFIG] Configuration saved successfully.");
                }
            }
            ctx.webview.load_uri("juanita://config?saved=true");
            return true;
        }

        if uri.starts_with("juanita://make-default") {
            let exe_path = std::env::current_exe()
                .unwrap_or_else(|_| std::path::PathBuf::from("juanita-banana"));
            let is_system_install = exe_path.starts_with("/usr/");

            let desktop_filename = if is_system_install {
                "juanita-banana.desktop".to_string()
            } else {
                let base = std::env::var("XDG_DATA_HOME")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| {
                        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
                            .join(".local/share")
                    });
                let apps_dir = base.join("applications");
                std::fs::create_dir_all(&apps_dir).ok();

                let desktop_path = apps_dir.join("juanita-banana-local.desktop");
                let desktop_content = format!(
                    "[Desktop Entry]\nVersion=1.0\nName=Juanita Banana (Local)\nGenericName=Web Browser\nComment=Weaponized Privacy Browser\nExec={} %U\nTerminal=false\nX-MultipleArgs=false\nType=Application\nIcon=web-browser\nCategories=Network;WebBrowser;\nMimeType=text/html;text/xml;application/xhtml+xml;x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/juanita;\nStartupNotify=true",
                    exe_path.display()
                );
                std::fs::write(&desktop_path, desktop_content).ok();
                "juanita-banana-local.desktop".to_string()
            };

            std::process::Command::new("xdg-settings")
                .arg("set")
                .arg("default-web-browser")
                .arg(&desktop_filename)
                .spawn()
                .ok();

            println!("[CONFIG] Set as default browser!");
            ctx.webview.load_uri("juanita://config");
            return true;
        }

        false
    }
}

// 3. Contribute Page
struct ContributePage;
impl InternalPage for ContributePage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:contribute") || input.starts_with("juanita://contribute")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        uri.starts_with("juanita://contribute") && !uri.starts_with("juanita://contribute-page")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, _uri: &str, ctx: &PageContext) -> bool {
        let b64_image = crate::util::image::get_monero_qr_b64();
        let html_template = include_str!("../../templates/contribute.html");
        let html = html_template.replace("{b64_image}", &b64_image);
        ctx.webview
            .load_html(&html, Some("juanita://contribute-page/"));
        true
    }
}

// 4. About Page
struct AboutPage;
impl InternalPage for AboutPage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:about") || input.starts_with("juanita://about")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        uri.starts_with("juanita://about") && !uri.starts_with("juanita://about-page")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, _uri: &str, ctx: &PageContext) -> bool {
        let b64_icon = crate::util::image::get_icon_b64();
        let b64_noise = crate::util::image::generate_random_noise_bmp_b64();
        let html_template = include_str!("../../templates/about.html");
        let html = html_template
            .replace("{b64_icon}", &b64_icon)
            .replace("{b64_noise}", &b64_noise);
        ctx.webview.load_html(&html, Some("juanita://about-page/"));
        true
    }
}

// 5. Competitors Page
struct CompetitorsPage;
impl InternalPage for CompetitorsPage {
    fn matches_input(&self, _input: &str) -> bool {
        false
    }

    fn handle_input(&self, _input: &str, _ctx: &PageContext) {}

    fn matches_policy(&self, uri: &str) -> bool {
        (uri.starts_with("juanita://choose-competitor")
            && !uri.starts_with("juanita://competitors-page"))
            || uri.starts_with("juanita://set-competitor")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool {
        if uri.starts_with("juanita://choose-competitor") {
            let html = crate::util::competitors::competitors_page_html();
            ctx.webview
                .load_html(&html, Some("juanita://competitors-page/"));
            return true;
        }

        if let Some(desktop_str) = uri.strip_prefix("juanita://set-competitor?desktop=") {
            let _ = std::process::Command::new("xdg-settings")
                .arg("set")
                .arg("default-web-browser")
                .arg(desktop_str)
                .output();
            ctx.webview.load_uri("juanita://config");
            return true;
        }

        false
    }
}

// 6. Downloads Page
struct DownloadsPage;
impl InternalPage for DownloadsPage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:downloads") || input.starts_with("juanita://downloads")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        uri.starts_with("juanita://downloads") && !uri.starts_with("juanita://downloads-page")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool {
        if let Some(id) = uri.strip_prefix("juanita://downloads/open?id=") {
            ctx.downloads.borrow().open_sandboxed(id);
            ctx.webview.load_uri("juanita://downloads");
            return true;
        }

        if let Some(id) = uri.strip_prefix("juanita://downloads/persist?id=") {
            ctx.downloads.borrow_mut().make_permanent(id);
            ctx.webview.load_uri("juanita://downloads");
            return true;
        }

        if let Some(id) = uri.strip_prefix("juanita://downloads/delete?id=") {
            ctx.downloads.borrow_mut().shred(id);
            ctx.webview.load_uri("juanita://downloads");
            return true;
        }

        // Just juanita://downloads
        let html = ctx.downloads.borrow().generate_html();
        ctx.webview
            .load_html(&html, Some("juanita://downloads-page/"));
        true
    }
}

// 7. Unban / Banned Page
struct UnbanPage;
impl InternalPage for UnbanPage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:unban")
            || input.starts_with("juanita://unban")
            || input.starts_with("juanita://submit-unban")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        (uri.starts_with("juanita://unban") && !uri.starts_with("juanita://unban-page"))
            || uri.starts_with("juanita://submit-unban")
            || (uri.starts_with("juanita://banned") && !uri.starts_with("juanita://banned-page"))
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool {
        if let Some(domain_query) = uri.strip_prefix("juanita://unban?domain=") {
            use crate::util::ban::{BasicIntegralEquationProvider, EquationProvider};
            let provider = BasicIntegralEquationProvider;
            let (equation, answer) = provider.generate_challenge();

            let domain = domain_query.to_string();
            *ctx.expected_unban.borrow_mut() = Some((domain.clone(), answer));

            let unban_html = crate::util::ban::unban_page(&domain, &equation);
            let base_uri = uri.replace("juanita://unban", "juanita://unban-page");
            ctx.webview.load_html(&unban_html, Some(&base_uri));
            return true;
        }

        if uri.starts_with("juanita://unban") {
            let domains = ctx.banlist.borrow().banned_domains.clone();
            let list_html = crate::util::ban::unban_list_page(&domains);
            ctx.webview
                .load_html(&list_html, Some("juanita://unban-page/"));
            return true;
        }

        if let Some(query) = uri.strip_prefix("juanita://submit-unban?") {
            let parts: Vec<&str> = query.split('&').collect();
            let mut domain = String::new();
            let mut answer = String::new();
            for p in parts {
                if let Some(d) = p.strip_prefix("domain=") {
                    domain = d.to_string();
                }
                if let Some(a) = p.strip_prefix("answer=") {
                    answer = a.to_string();
                }
            }

            if let Some((expected_domain, expected_ans)) = ctx.expected_unban.borrow().as_ref() {
                if *expected_domain == domain && answer == expected_ans.to_string() {
                    println!("[UNBAN] User solved the math! Unbanning {}", domain);
                    let mut bl = ctx.banlist.borrow_mut();
                    bl.unban(&domain);
                    bl.save();
                    ctx.webview.load_uri(&format!("https://{}", domain));
                    return true;
                }
            }

            println!("[UNBAN] Incorrect math or tampered domain. Access denied.");
            let banned_html = crate::util::ban::banned_page(&domain);
            ctx.webview
                .load_html(&banned_html, Some("juanita://banned"));
            return true;
        }

        if uri.starts_with("juanita://banned") {
            // Usually this loads directly when we call it. We shouldn't recurse here.
            // But if triggered, we can just load the banned page for the current URL.
            let domain = if let Some(current_uri) = ctx.webview.uri() {
                current_uri.to_string()
            } else {
                "unknown".to_string()
            };
            let banned_html = crate::util::ban::banned_page(&domain);
            ctx.webview
                .load_html(&banned_html, Some("juanita://banned"));
            return true;
        }

        false
    }
}
