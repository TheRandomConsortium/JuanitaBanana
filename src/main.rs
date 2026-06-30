// ============================================================
// JUANITA BANANA 🍌
//
// Stack: 100% Rust
//   Engine:  Servo (Linux Foundation, not Google)
//   Window:  winit
//   GL:      surfman
//   Chrome:  egui (URL bar + BAN button, minimal)
//
// Anti-fingerprinting:
//   Canvas/viewport spoofing is achieved by implementing
//   Servo's EmbedderMethods — we control what JS sees.
//   SpiderMonkey executes the page's JS, but we control
//   the APIs that the JS can read.
//   No engine rewrite needed for this.
//
// NOTE: First build = 30-60 min (SpiderMonkey in C++).
//       Subsequent builds are fast.
// ============================================================

mod ban;
mod browser;
mod gui;
mod spoof;

fn main() {
    // Logger initialized by Servo
    // Load persisted ban list
    let state = browser::BanList::load();

    // Launch GTK application
    gui::run(state);
}
