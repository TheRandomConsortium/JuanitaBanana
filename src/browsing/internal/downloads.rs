use super::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct DownloadsPage;

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
