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
mod fingerprint;
mod search;
mod util;

fn main() {
    let config = util::config::AppConfig::load();
    let state = browsing::browser::BanList::load(&config);
    config.save();
    state.borrow().save();

    // Launch GTK application
    browsing::gui::run(state);
}
