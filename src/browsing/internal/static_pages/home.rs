use crate::browsing::internal::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct HomePage;

impl InternalPage for HomePage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:home") || input.starts_with("juanita://home")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        uri.starts_with("juanita://home") && !uri.starts_with("juanita://home-page")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, _uri: &str, ctx: &PageContext) -> bool {
        let b64_image = crate::util::image::get_juanita_throwing_papers_b64();
        let html_template = include_str!("../../../../templates/home.html");
        let html = html_template.replace("{b64_image}", &b64_image);
        let webview_clone = ctx.webview.clone();
        gtk::glib::idle_add_local(move || {
            webview_clone.load_html(&html, Some("juanita://home-page/"));
            gtk::glib::ControlFlow::Break
        });
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_page_matching() {
        let page = HomePage;
        assert!(page.matches_input("juanita:home"));
        assert!(page.matches_input("juanita://home"));
        assert!(page.matches_policy("juanita://home"));
        assert!(!page.matches_policy("juanita://home-page/"));
    }
}
