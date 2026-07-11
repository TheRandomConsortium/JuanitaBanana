use crate::browsing::browser::BanList;
use crate::util::config::AppConfig;
use crate::util::downloads::DownloadManager;
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::WebView;

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

mod about;
mod competitors;
mod config;
mod contribute;
mod downloads;
mod history;
mod home;
mod local_html;
mod passwords;
mod unban;

pub use about::AboutPage;
pub use competitors::CompetitorsPage;
pub use config::ConfigPage;
pub use contribute::ContributePage;
pub use downloads::DownloadsPage;
pub use history::HistoryPage;
pub use home::HomePage;
pub use local_html::LocalHtmlPage;
pub use passwords::PasswordsPage;
pub use unban::UnbanPage;

pub fn get_internal_pages() -> Vec<Box<dyn InternalPage>> {
    vec![
        Box::new(LocalHtmlPage),
        Box::new(PasswordsPage),
        Box::new(HomePage),
        Box::new(HistoryPage),
        Box::new(ConfigPage),
        Box::new(ContributePage),
        Box::new(AboutPage),
        Box::new(CompetitorsPage),
        Box::new(DownloadsPage),
        Box::new(UnbanPage),
    ]
}
