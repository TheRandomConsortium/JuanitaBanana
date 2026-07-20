use crate::browsing::internal::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct ContactPage;

impl InternalPage for ContactPage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:contact") || input.starts_with("juanita://contact")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        uri.starts_with("juanita://contact") && !uri.starts_with("juanita://contact-page")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, _uri: &str, ctx: &PageContext) -> bool {
        let b64_icon = crate::util::image::get_icon_b64();
        let html = include_str!("../../../../templates/pages/contact.html")
            .replace(
                "{shared_css}",
                crate::browsing::internal::SHARED_CSS.as_str(),
            )
            .replace("{b64_icon}", &b64_icon);
        ctx.webview
            .load_html(&html, Some("juanita://contact-page/"));
        true
    }
}
