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
        let html_template = include_str!("../../../../templates/contact.html");
        let html = html_template.replace("{b64_icon}", &b64_icon);
        ctx.webview
            .load_html(&html, Some("juanita://contact-page/"));
        true
    }
}
