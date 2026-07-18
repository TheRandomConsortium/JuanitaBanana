//! # `webcontext.rs` — WebKit proxy configuration — PARTIALLY DEPRECATED
//!
//! > **⚠️ Phase 4: proxy logic in this file will be removed. The file may survive in reduced form.**
//!
//! Currently configures WebKit's `WebsiteDataManager` to use the local SOCKS5 helper
//! (port 9151) when Tor or Handshake is active.
//!
//! **What dies in Phase 4:**
//! - `socks5://127.0.0.1:9151` proxy URI configuration (local helper removed)
//! - `ARTI_SOCKS_PORT` / `LOCAL_PROXY_PORT` references (both ports disappear)
//!
//! **What may survive:**
//! - Any non-proxy WebContext/WebsiteDataManager configuration (e.g. TLS settings,
//!   content filters, data storage paths) that is unrelated to the SOCKS5 hop chain.
//!   Evaluate at Phase 4 implementation time.
//!
//! **Replacement for the proxy part:** `arti-client` intercepts connections in-process;
//! no external proxy URI needs to be set on WebKit at all.

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
    let tor_active = cfg.tor_enabled && crate::tor::is_tor_running();
    let handshake_active = cfg.handshake_enabled;

    if !tor_active && !handshake_active {
        if let Some(data_manager) = web_context.website_data_manager() {
            data_manager.set_network_proxy_settings(NetworkProxyMode::Default, None);
            log!(Info, TOR, "WebKit proxy settings reset to default");
        }
        return;
    }

    let socks_uri = format!("socks5://127.0.0.1:{}", crate::tor::LOCAL_PROXY_PORT);

    let mut proxy_settings = NetworkProxySettings::new(Some(socks_uri.as_str()), &[]);

    // `WebContextExtManual` is not re-exported at the webkit2gtk crate root
    // (web_context module is commented out in lib.rs). We reach the same FFI call
    // via `WebsiteDataManager`, which IS part of the public surface.
    if let Some(data_manager) = web_context.website_data_manager() {
        data_manager
            .set_network_proxy_settings(NetworkProxyMode::Custom, Some(&mut proxy_settings));
        log!(
            Info,
            TOR,
            "WebKit SOCKS5 proxy configured to local helper: {}",
            socks_uri
        );
    }
}
