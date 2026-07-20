// ============================================================
// JUANITA BANANA 🍌
//
// Stack:
//   Engine:  WebKitGTK (pragmatic fallback)
//            Goal: Return to Servo once its HTTP/2 stack matures.
//   Window:  GTK3 native
//   Chrome:  GTK HeaderBar + Entry + Button
//
// Anti-fingerprinting:
//   A JS payload is injected into EVERY page (including all
//   sub-frames) via WebKit's UserContentManager BEFORE any
//   page script executes. We overwrite tracking APIs at the
//   JS prototype level. No engine rewrite needed.
//
// NOTE: Build is fast — dynamically links to system GTK/WebKit.
// ============================================================

mod browsing;
mod plugins;
mod privacy;
mod resolver;
mod tor;
mod unsubscribe;
mod util;

pub use privacy::{ad_intoxication, fingerprint, search};

struct CleanupGuard;

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        crate::tor::shutdown_tor();
        crate::resolver::shutdown_resolver();
    }
}

fn main() {
    let _guard = CleanupGuard;
    let config = util::config::AppConfig::load();
    let state = browsing::browser::BanList::load(&config);
    config.save();
    state.borrow().save();

    // Start local SOCKS5 proxy server for Handshake & Tor resolution
    tor::start_local_proxy();

    // Start Tor transport if enabled in config
    tor::init_tor();

    // Start local resolvers / daemon
    resolver::init_resolver();

    // Launch GTK application
    browsing::gui::run(state);
}
