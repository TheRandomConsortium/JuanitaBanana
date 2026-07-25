use crate::browsing::internal::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct AiAlternativesPage;

impl InternalPage for AiAlternativesPage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:ai-alternatives")
            || input.starts_with("juanita://ai-alternatives")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        uri.starts_with("juanita://ai-alternatives")
            && !uri.starts_with("juanita://ai-alternatives-page")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, _uri: &str, ctx: &PageContext) -> bool {
        let html = include_str!("../../../../templates/pages/ai_alternatives.html").replace(
            "{shared_css}",
            crate::browsing::internal::SHARED_CSS.as_str(),
        );
        ctx.webview
            .load_html(&html, Some("juanita://ai-alternatives-page/"));
        true
    }
}
