use crate::browsing::browser::BanList;
use crate::util::config::AppConfig;
use crate::util::downloads::DownloadManager;
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::WebView;

lazy_static::lazy_static! {
    pub static ref SHARED_CSS: String = {
        let font_b64 = crate::util::font::get_outfit_font_b64();
        let fonts_css = include_root_str!(@templates, "fonts.css")
            .replace("{outfit_font_b64}", &font_b64);
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}",
            fonts_css,
            include_root_str!(@templates, "tokens.css"),
            include_root_str!(@templates, "layout.css"),
            include_root_str!(@templates, "components.css"),
            include_root_str!(@templates, "competitors.css"),
            include_root_str!(@templates, "vault.css"),
            include_root_str!(@templates, "local_html.css")
        )
    };
}

pub struct PageContext {
    pub webview: WebView,
    pub downloads: Rc<RefCell<DownloadManager>>,
    pub banlist: Rc<RefCell<BanList>>,
    pub expected_unban: Rc<RefCell<Option<(String, i32)>>>,
    pub noise_pool: Rc<RefCell<crate::privacy::search::gossip::SearchNoisePool>>,
    pub config: AppConfig,
}

pub trait InternalPage {
    fn matches_input(&self, input: &str) -> bool;
    fn handle_input(&self, input: &str, ctx: &PageContext);
    fn matches_policy(&self, uri: &str) -> bool;
    fn ignore_policy(&self, uri: &str) -> bool;
    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool;
}

pub mod config_pages;
pub mod static_pages;
pub mod utils;

pub use config_pages::competitors::CompetitorsPage;
pub use config_pages::config::ConfigPage;
pub use config_pages::unban::UnbanPage;
pub use static_pages::about::AboutPage;
pub use static_pages::ai_alternatives::AiAlternativesPage;
pub use static_pages::contact::ContactPage;
pub use static_pages::contribute::ContributePage;
pub use static_pages::history::HistoryPage;
pub use static_pages::home::HomePage;
pub use utils::downloads::DownloadsPage;
pub use utils::local_html::LocalHtmlPage;
pub use utils::passwords::PasswordsPage;
pub use utils::search_explorer::SearchExplorerPage;

pub fn get_internal_pages() -> Vec<Box<dyn InternalPage>> {
    vec![
        Box::new(LocalHtmlPage),
        Box::new(PasswordsPage),
        Box::new(HomePage),
        Box::new(HistoryPage),
        Box::new(ConfigPage),
        Box::new(ContributePage),
        Box::new(AboutPage),
        Box::new(AiAlternativesPage),
        Box::new(ContactPage),
        Box::new(CompetitorsPage),
        Box::new(DownloadsPage),
        Box::new(UnbanPage),
        Box::new(SearchExplorerPage),
    ]
}
