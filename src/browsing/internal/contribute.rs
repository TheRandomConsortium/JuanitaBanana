use super::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct ContributePage;

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
        let html_template = include_str!("../../../templates/contribute.html");
        let html = html_template.replace("{b64_image}", &b64_image);
        ctx.webview
            .load_html(&html, Some("juanita://contribute-page/"));
        true
    }
}
