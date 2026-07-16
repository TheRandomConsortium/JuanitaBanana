use crate::browsing::internal::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct AboutPage;

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
        let html_template = include_str!("../../../../templates/about.html");
        let html = html_template
            .replace("{b64_icon}", &b64_icon)
            .replace("{b64_noise}", &b64_noise);
        ctx.webview.load_html(&html, Some("juanita://about-page/"));
        true
    }
}
