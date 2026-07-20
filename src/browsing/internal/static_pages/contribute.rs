use crate::browsing::internal::{InternalPage, PageContext};
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
        let html = include_str!("../../../../templates/pages/contribute.html")
            .replace(
                "{shared_css}",
                crate::browsing::internal::SHARED_CSS.as_str(),
            )
            .replace("{b64_image}", &b64_image);
        ctx.webview
            .load_html(&html, Some("juanita://contribute-page/"));
        true
    }
}
