# Juanita Banana — Technical Debt & Chore Roadmap

This document maintains the tracking of known technical chores, API deprecations, and platform upgrades to preserve the security and robustness of the Juanita Banana browser.

> [!NOTE]
> Chores marked as Done will be removed from this list upon major or minor version bumps (e.g. vX.Y.0) to prevent the list from growing infinitely.

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

### 11. Verify Hardware Security Key (FIDO2/WebAuthn) Support
- **Chore:** Verify that physical hardware keys (USB/NFC) function correctly for WebAuthn/U2F flows within the WebKitGTK environment, given the architectural rejection of cloud-based CaBLE passkeys.
- **Action Plan:**
  - Test WebAuthn registration and authentication flows on standard services using a physical hardware key (e.g., YubiKey, Nitrokey).
  - Ensure the browser successfully communicates with local daemon services (`pcscd`, `libfido2`) and that strict sandboxing/hardening layers do not inadvertently block hardware USB polling.

### 12. Create a Unified Internal Stylesheet
- **Files:** `templates/`, `src/browsing/internal/`, `src/browsing/tabs/tab.rs`, all inline HTML in Rust source
- **Chore:** All internal pages (`juanita://home`, `juanita://config`, `juanita://downloads`, `juanita://unban`, `juanita://mail`, error pages, TLS warning page, ban page, unsubscribe wizard, etc.) currently have ad-hoc inline CSS with inconsistent colors, fonts, spacing, and border-radius values.
- **Action Plan:**
  - Define a single CSS design token file (`templates/juanita.css` or `assets/internal.css`) containing all shared variables: color palette, font families, font sizes, spacing scale, border-radius, button styles, card styles, section-title styles.
  - Embed this stylesheet via `include_str!` so it is compiled into the binary and injected into every internal page's `<head>`.
  - Migrate all internal HTML templates and inline Rust format strings to reference the shared CSS classes instead of their own ad-hoc rules.
  - Retire redundant per-page style blocks once migrated.

### 13. Deploy and Integrate Tox Contact Channel
- **Files:** `templates/contact.html`
- **Chore:** Establish a metadata-free, serverless Tox account for Consortium support queries.
- **Action Plan:**
  - Spin up a dedicated Tox node using a secure client (e.g. standard aTox or qTox instance).
  - Extract the public Tox ID.
  - Replace the "Inactive / Currently Disabled" placeholder in templates/contact.html with the active Tox ID.
  - Document operational guidelines for incoming queries.

### 14. Configure WebKit Proxy Timeout Patience
- **Files:** `src/browsing/tabs/tab.rs`, `src/tor/webcontext.rs`
- **Chore:** WebKit's internal network stack has highly aggressive connection and proxy handshake timeout thresholds. When Tor circuit building is slow, WebKit aborts prematurely and issues a load-failed event before the local SOCKS5 proxy gets a chance to establish the circuit.
- **Action Plan:**
  - Investigate WebKitGTK setting interfaces, environment variables (e.g., Soup settings or system variables), and system-level configuration parameters that govern request connection timeouts.
  - Find a way to make WebKit more patient and wait longer for proxy handshakes to resolve before aborting.