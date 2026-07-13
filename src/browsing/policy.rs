use crate::ad_intoxication::AdIntoxicationEngine;
use crate::browsing::browser::SharedBanList;
use crate::browsing::internal::InternalPage;
use crate::log;
use crate::search::intoxication::IntoxicationEngine;
use crate::search::noise::RssNoiseProvider;
use crate::util::config::AppConfig;
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
        log!(
            Info,
            AD_INTOX,
            "Intercepted ad navigation in main window to: {}",
            uri_str
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
            log!(
                Debug,
                GUI,
                "decide_policy matched internal page for: {}",
                uri_str
            );
            if page.ignore_policy(uri_str) {
                log!(
                    Debug,
                    GUI,
                    "ignore_policy returned true, ignoring navigation: {}",
                    uri_str
                );
                decision.ignore();
            } else {
                log!(Debug, GUI, "ignore_policy returned false: {}", uri_str);
            }
            if page.handle_policy(uri_str, &ctx) {
                log!(Debug, GUI, "handle_policy returned true for: {}", uri_str);
                page_handled = true;
                break;
            } else {
                log!(Debug, GUI, "handle_policy returned false for: {}", uri_str);
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

    // Run priority resolver chain for domain resolution if external HTTP/HTTPS page
    if uri_str.starts_with("http://") || uri_str.starts_with("https://") {
        let domain = crate::browsing::browser::extract_domain(uri_str);
        let host = crate::resolver::clean_host(&domain);
        if !host.is_empty() && host.parse::<std::net::IpAddr>().is_err() {
            match crate::resolver::resolve_domain_with_chain(&host) {
                Ok((ip, _resolver_name)) => {
                    // If the domain is system-resolvable, let WebKit load it natively to avoid redirect loops on clearweb sites.
                    if crate::resolver::is_system_resolvable(&host) {
                        return false;
                    }

                    // Otherwise, it is a Handshake-only domain. Rewrite it to the IP address.
                    let ip_str = ip.to_string();
                    crate::resolver::register_resolved_ip(ip_str.clone(), host.clone());
                    // Force http:// to avoid TLS SNI unrecognized name alert issues
                    let new_uri = crate::resolver::rewrite_uri_host(uri_str, &host, &ip_str)
                        .replace("https://", "http://");
                    log!(
                        Info,
                        RESOLVER,
                        "Rewriting navigation from '{}' to IP '{}' (new URI: '{}')",
                        host,
                        ip_str,
                        new_uri
                    );
                    webview_nav.load_uri(&new_uri);
                    decision.ignore();
                    return true;
                }
                Err(e) => {
                    log!(
                        Error,
                        RESOLVER,
                        "Failed to resolve domain '{}' via priority chain: {}",
                        host,
                        e
                    );
                    decision.ignore();
                    let error_html = format!(
                        "<html><head><style>
                        body {{ background: #121214; color: #e1e1e6; display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; margin: 0; font-family: monospace; text-align: center; }}
                        .card {{ background: #1a1a1e; border: 1px solid #29292e; padding: 40px; border-radius: 12px; max-width: 600px; box-shadow: 0 8px 24px rgba(0,0,0,0.5); }}
                        h1 {{ color: #ff5555; font-size: 2.5rem; margin: 0 0 20px 0; }}
                        p {{ color: #a9a9b3; font-size: 1.1rem; line-height: 1.6; margin: 0 0 20px 0; }}
                        .error-box {{ background: #282a36; border-left: 4px solid #ff79c6; padding: 15px; text-align: left; font-family: monospace; font-size: 0.95rem; color: #f8f8f2; white-space: pre-wrap; }}
                        </style></head><body>
                        <div class=\"card\">
                            <h1>Server Not Found</h1>
                            <p>Juanita Banana's priority resolver chain failed to resolve the host <strong>{}</strong>.</p>
                            <div class=\"error-box\">{}</div>
                        </div>
                        </body></html>",
                        host, e
                    );
                    webview_nav.load_html(&error_html, Some("juanita://dns-error"));
                    return true;
                }
            }
        }
    }

    false
}
