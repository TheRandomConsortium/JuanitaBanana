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

### 4. Expand DPA Complaint Scope (Multi-Authority Support)
- **Chore:** Research and catalogue the online complaint submission portals for each major EU DPA and map their form fields to the data already present in the generated report.
- **Affected Authorities:**
  - AEPD (Spain) — `https://www.aepd.es/es/derechos-y-deberes/conoce-tus-derechos/derecho-de-reclamacion`
  - Garante (Italy) — `https://www.garanteprivacy.it/ricorsi`
  - CNIL (France) — `https://www.cnil.fr/fr/plaintes`
  - BfDI (Germany) — `https://www.bfdi.bund.de/DE/Service/Beschwerde/beschwerde_node.html`
  - ICO (UK) — `https://ico.org.uk/make-a-complaint/`
  - DPC (Ireland) — `https://www.dataprotection.ie/en/individuals/raising-concern-data-protection-commission`
  - EDPB One-Stop-Shop — `https://edpb.europa.eu`
- **Action Plan:**
  - For each authority: scrape the submission form HTML, identify required fields, and create a field-mapping from the GDPR report struct.
  - Build a generic `DpaSubmissionAdapter` trait with one implementation per authority.
  - Add a new wizard step (Step 6) offering direct online submission using the already-generated `.p7m` as the attachment.

### 5. XAdES Support (XML Digital Signatures)
- **Chore:** Some DPAs (notably the AEPD's Cl@ve gateway) prefer XAdES (XML Advanced Electronic Signatures) over CAdES. Research which authorities require XAdES and implement an `xmlsec1`-based or `openssl`-based XAdES-B-BES signing path as an alternative output format.
- **Action Plan:**
  - Audit each DPA submission portal for accepted signature formats.
  - Implement `sign_document_xades()` analogous to `sign_pdf_cades_in_memory()`, keeping all key material in RAM.
  - Gate behind a per-authority capability flag in the `DpaSubmissionAdapter`.

### 6. Certificate Rotation / Expiry Warning
- **Chore:** PKCS#12 certificates from issuers like FNMT expire (typically 2–4 years). Add expiry detection when loading a stored certificate and warn the user via an info bar in the wizard.
- **Action Plan:**
  - Parse the `not_after` field from the X.509 cert in `db_certs.rs` at load time using the `openssl` crate's `X509::not_after()`.
  - If within 90 days of expiry, show a non-blocking warning banner. If expired, treat as no-certificate (fall back to unsigned PDF) and show an error.

### 7. Migrate all prints from `println` to `log!(...)`
- **Chore:** Use `log!(...)` macros for all debug output instead of `println!`.
- **Action Plan:**
  - Find all `println!` macros in the codebase.
  - Replace them with `log!(...)`.
  - Keep the `JUANITA_LOG` environment variable to control the log level.
- **Considerations:**
  - error level => `eprintln!(...)`
  - warn, info, debug level => `println!(...)`

### 8. Fix Depth Slider for Adblocking
- **Chore:** Resolve issues with the adblocking depth slider being unresponsive or not saving values properly in the UI.
- **Action Plan:**
  - Audit templates/config.html and script/config.js for event handling and mapping of the depth slider input.
  - Ensure config state updates properly on change.

### 9. Investigate DoubleClick Ad Blocker Evasion
- **Chore:** Determine why DoubleClick ads are evading blocking/poisoning in specific scenarios (e.g., repeating Toyota Yaris ads on La Voz de Galicia).
- **Action Plan:**
  - Analyze network request patterns and script contexts on affected pages.
  - Check if specific subdomains or redirect paths bypass standard host/regex matching.

