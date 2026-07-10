use crate::ad_intoxication::AdIntoxicationEngine;
use crate::browsing::browser::SharedBanList;
use crate::browsing::internal::InternalPage;
use crate::debug_log;
use crate::search::intoxication::IntoxicationEngine;
use crate::search::noise::RssNoiseProvider;
use crate::util::config::AppConfig;
use crate::util::debug::redact_uri;
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{PolicyDecision, PolicyDecisionExt, WebView, WebViewExt};

#[allow(clippy::too_many_arguments)]
pub fn handle_decide_policy(
    decision: &PolicyDecision,
    uri_str: &str,
    webview_nav: &WebView,
    downloads_policy: &Rc<RefCell<crate::util::downloads::DownloadManager>>,
    banlist_policy: &SharedBanList,
    expected_unban_policy: &Rc<RefCell<Option<(String, i32)>>>,
    internal_pages_policy: &[Box<dyn InternalPage>],
    ad_intox_engine_policy: &AdIntoxicationEngine,
    intox_engine: &IntoxicationEngine,
    noise_provider: &RssNoiseProvider,
) -> bool {
    // Intercept navigation to ad domains in main webview
    if ad_intox_engine_policy.is_ad_domain(uri_str) {
        println!(
            "[AD_INTOX] Intercepted ad navigation in main window to: {}",
            redact_uri(uri_str)
        );
        ad_intox_engine_policy.queue_ad(crate::ad_intoxication::AdTask {
            page_url: uri_str.to_string(),
            selector: "body".to_string(),
            ad_url: uri_str.to_string(),
        });
        decision.ignore();
        return true;
    }

    intox_engine.cancel_pending();

    let ctx = crate::browsing::internal::PageContext {
        webview: webview_nav.clone(),
        downloads: downloads_policy.clone(),
        banlist: banlist_policy.clone(),
        expected_unban: expected_unban_policy.clone(),
        config: AppConfig::load(),
    };

    let mut page_handled = false;
    for page in internal_pages_policy.iter() {
        if page.matches_policy(uri_str) {
            debug_log!(GUI, "decide_policy matched internal page for: {}", uri_str);
            if page.ignore_policy(uri_str) {
                debug_log!(
                    GUI,
                    "ignore_policy returned true, ignoring navigation: {}",
                    uri_str
                );
                decision.ignore();
            } else {
                debug_log!(GUI, "ignore_policy returned false: {}", uri_str);
            }
            if page.handle_policy(uri_str, &ctx) {
                debug_log!(GUI, "handle_policy returned true for: {}", uri_str);
                page_handled = true;
                break;
            } else {
                debug_log!(GUI, "handle_policy returned false for: {}", uri_str);
            }
        }
    }
    if page_handled {
        return true;
    }

    // Check for Search Intoxication
    if intox_engine.check_and_poison_search(uri_str, &ctx.config, noise_provider) {
        decision.ignore();
        return true;
    }
    if banlist_policy.borrow().is_banned(uri_str) {
        decision.ignore();
        let banned_html = crate::util::ban::banned_page(uri_str);
        webview_nav.load_html(&banned_html, Some("juanita://banned"));
        return true;
    }
    false
}
