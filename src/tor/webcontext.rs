use crate::log;
use crate::util::config::AppConfig;
use webkit2gtk::{
    NetworkProxyMode, NetworkProxySettings, WebContext, WebContextExt, WebsiteDataManagerExt,
};

/// Applies (or removes) the Tor SOCKS5 proxy on the shared `WebContext` used by all tabs.
///
/// When Tor is enabled and arti is running:
/// - Sets the SOCKS5 proxy to `socks5://127.0.0.1:9150` on the `WebsiteDataManager`.
/// - If `tor_route_all` is false, only `.onion` addresses reach the proxy (the `OnionResolver`
///   returns sentinel `127.0.0.2`, `policy.rs` keeps the `.onion` URI and calls `decision.use_()`).
/// - If `tor_route_all` is true, all clearnet traffic also routes through Tor exit nodes.
///
/// When Tor is disabled or arti is not running the proxy is left at its default (system proxy).
pub fn apply_tor_proxy(web_context: &WebContext) {
    let cfg = AppConfig::load();
    if !cfg.tor_enabled || !crate::tor::is_tor_running() {
        return;
    }

    let socks_uri = format!("socks5://127.0.0.1:{}", crate::tor::ARTI_SOCKS_PORT);

    let mut proxy_settings = NetworkProxySettings::new(Some(socks_uri.as_str()), &[]);

    // `WebContextExtManual` is not re-exported at the webkit2gtk crate root
    // (web_context module is commented out in lib.rs). We reach the same FFI call
    // via `WebsiteDataManager`, which IS part of the public surface.
    if let Some(data_manager) = web_context.website_data_manager() {
        data_manager
            .set_network_proxy_settings(NetworkProxyMode::Custom, Some(&mut proxy_settings));
        log!(Info, TOR, "WebKit SOCKS5 proxy configured: {}", socks_uri);
    }
}
