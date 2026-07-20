use crate::browsing::browser::BanList;
use crate::util::config::AppConfig;
use crate::util::downloads::DownloadManager;
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::WebView;

pub const SHARED_CSS: &str = concat!(
    include_str!("../../../templates/styles/fonts.css"),
    "\n",
    include_str!("../../../templates/styles/tokens.css"),
    "\n",
    include_str!("../../../templates/styles/layout.css"),
    "\n",
    include_str!("../../../templates/styles/components.css"),
    "\n",
    include_str!("../../../templates/styles/competitors.css"),
    "\n",
    include_str!("../../../templates/styles/vault.css"),
    "\n",
    include_str!("../../../templates/styles/local_html.css")
);

pub struct PageContext {
    pub webview: WebView,
    pub downloads: Rc<RefCell<DownloadManager>>,
    pub banlist: Rc<RefCell<BanList>>,
    pub expected_unban: Rc<RefCell<Option<(String, i32)>>>,
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
pub use static_pages::contact::ContactPage;
pub use static_pages::contribute::ContributePage;
pub use static_pages::history::HistoryPage;
pub use static_pages::home::HomePage;
pub use utils::downloads::DownloadsPage;
pub use utils::local_html::LocalHtmlPage;
pub use utils::passwords::PasswordsPage;

pub fn get_internal_pages() -> Vec<Box<dyn InternalPage>> {
    vec![
        Box::new(LocalHtmlPage),
        Box::new(PasswordsPage),
        Box::new(HomePage),
        Box::new(HistoryPage),
        Box::new(ConfigPage),
        Box::new(ContributePage),
        Box::new(AboutPage),
        Box::new(ContactPage),
        Box::new(CompetitorsPage),
        Box::new(DownloadsPage),
        Box::new(UnbanPage),
    ]
}
