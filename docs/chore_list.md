# Juanita Banana — Technical Debt & Chore Roadmap

This document maintains the tracking of known technical chores, API deprecations, and platform upgrades to preserve the security and robustness of the Juanita Banana browser.

## 📋 Scheduled Chores

### 1. Refactor Legacy Channels
- **File:** `src/browsing/gui.rs`
- **Chore:** Remove the deprecated `gtk::glib::MainContext::channel` setup.
- **Action Plan:**
  - Route URL loading inside `connect_open` directly to the main thread's webview instance (or enqueue it in `pending_urls` if the GUI is not yet fully initialized).
  - Transition the notify-send download completion callback to use a standard `std::sync::mpsc::channel` polled via a low-frequency `glib::timeout_add_local` check (e.g., every 250ms), eliminating cross-thread GSource deprecations.

### 2. Standardize WebKitGTK API Calls
- **Files:** `src/browsing/gui.rs`, `src/browsing/gui_plugin.rs`, `src/ad_intoxication/engine.rs`
- **Chore:** Resolve WebKitGTK-specific deprecated property accessors.
- **Action Plan:**
  - Replace `download.request()` with `download.uri_request()`.
  - Replace `nav_decision.request()` with `nav_decision.navigation_action().request()`.
  - Once replaced, remove all local `#[allow(deprecated)]` directives in these files to ensure any future deprecated APIs fail compilation under `-D warnings`.

### 3. Upgrade WebKitGTK Bindings
- **Chore:** Migrate dependencies away from deprecated and increasingly unsupported GTK 3/libsoup 2 bindings.
- **Action Plan:**
  - **Short Term:** Upgrade `webkit2gtk` dependencies in `Cargo.toml` to target the `4.1` API level (GTK 3 with libsoup 3) to remain compatible with modern TLS and HTTP/2 requirements.
  - **Long Term:** Re-evaluate and plan a migration path to `webkit6` (GTK 4 with libsoup 3) to align with modern upstream GNOME development.
