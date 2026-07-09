use super::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct HistoryPage;

impl InternalPage for HistoryPage {
    fn matches_input(&self, input: &str) -> bool {
        input == "juanita:history" || input == "juanita://history"
    }

    fn handle_input(&self, _input: &str, ctx: &PageContext) {
        let html = include_str!("../../../templates/history.html");
        ctx.webview.load_html(html, Some("juanita://history-page/"));
    }

    fn matches_policy(&self, uri: &str) -> bool {
        uri == "juanita:history" || uri == "juanita://history" || uri == "juanita://history/"
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, _uri: &str, ctx: &PageContext) -> bool {
        let html = include_str!("../../../templates/history.html");
        ctx.webview.load_html(html, Some("juanita://history-page/"));
        true
    }
}
