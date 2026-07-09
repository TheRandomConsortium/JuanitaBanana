use super::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct CompetitorsPage;

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
