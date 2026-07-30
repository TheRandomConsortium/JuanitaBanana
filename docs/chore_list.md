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

### 14. Configure WebKit Proxy Timeout Patience
- **Files:** `src/browsing/tabs/tab.rs`, `src/tor/webcontext.rs`
- **Chore:** WebKit's internal network stack has highly aggressive connection and proxy handshake timeout thresholds. When Tor circuit building is slow, WebKit aborts prematurely and issues a load-failed event before the local SOCKS5 proxy gets a chance to establish the circuit.
- **Action Plan:**
  - Investigate WebKitGTK setting interfaces, environment variables (e.g., Soup settings or system variables), and system-level configuration parameters that govern request connection timeouts.
  - Find a way to make WebKit more patient and wait longer for proxy handshakes to resolve before aborting.

### 15. Handle Last Tab Manual Closure Behavior
- **Files:** `src/browsing/tabs/cleanup.rs`
- **Chore:** When the final tab is manually closed by the user, immediately open a new tab pointing to the home page or close/exit the browser, depending on the `last_tab_nuke_action` configuration setting (e.g., if set to `home`, open home; if set to `survive` (or when closing the final tab manually), exit the browser).
- **Action Plan:**
  - In `manual_close_tab`, check if the removed tab was the last active tab (i.e., `tabs` becomes empty).
  - Load the application configuration via `crate::util::config::AppConfig::load()`.
  - Evaluate `config.last_tab_nuke_action`:
    - If it is `"home"`, create/open a new tab pointing to `juanita://home`.
    - If it is `"survive"`, close the browser window (or trigger application quit).

### 17. Suppress Connection Error Screen on Downloads Navigating in New Tabs
- **Files:** `src/browsing/policy.rs`, `src/browsing/tabs/tab.rs`, `src/util/downloads.rs`
- **Chore:** When a user clicks a download link or navigation that opens a new tab (`target="_blank"`), WebKit initiates page navigation before classifying the response as a binary file download. When WebKit transfers the policy decision to the `DownloadManager` and cancels the tab's page navigation, WebKit emits a navigation error / load failure signal, causing the newly opened tab to display a confusing "Connection Error / Server Not Found" error screen even though the download itself successfully started in the background.
- **Action Plan:**
  - Intercept download policy decisions in `policy.rs` and `tab.rs`.
  - When a navigation is cancelled because it was converted into an active download, suppress loading the error template (`proxy.html` / `tls.html`).
  - Either close the newly spawned tab automatically if it was opened solely for the download, or load `juanita://downloads` to present the active download progress cleanly instead of a confusing error screen.

### 18. Custom Error Screen for Overlay Network 502 / 503 / 504 Gateway & Eepsite Errors
- **Files:** `src/tor/i2p_helper.rs`, `src/browsing/tabs/tab.rs`, `templates/errors/eepsite_503.html`
- **Chore:** When attempting to access an I2P eepsite (`.i2p`), Tor hidden service (`.onion`), or Handshake site that is offline, unreachable, or building tunnels, the upstream I2P router or proxy returns HTTP status 502 (Domain Not Found), 503 (Service Unavailable), or 504 (Gateway Timeout). Currently, raw proxy error strings or generic connection error pages are shown to the user.
- **Action Plan:**
  - Create a dedicated custom error template (e.g. `templates/errors/eepsite_503.html` or `gateway_error.html`) explaining that the target overlay site is currently unreachable, offline, or building tunnels in the network.
  - Intercept HTTP 502 / 503 / 504 responses in `src/tor/i2p_helper.rs` and `tab.rs`.
  - Render the custom overlay error page with helpful guidance and a "Retry Connection" action.

### 19. Implement JS Linting & Testing in CI / Actions
- **Files:** `.github/workflows/ci.yml`, `scripts/js/`, `scripts/sh/check_cleanliness.sh`
- **Chore:** JavaScript files (`scripts/js/`) represent approximately ~9% of the Juanita Banana project codebase. Implement automated JS linting (e.g., ESLint / `node --check`) and JS unit testing into our verification scripts and CI workflow actions.
- **Action Plan:**
  - Add `node --check` and ESLint checks to `./scripts/sh/check_cleanliness.sh` and CI pipeline.
  - Create a lightweight test harness to execute narrow unit tests for anti-fingerprinting JS helpers (`_makeNative`, `_defineGetter`, `_overrideMethod`, `_protectedToString`).

### 20. Transition Testing Strategy to TDD-Adjacent Narrow Functional & Broad Happy Path Tests
- **Files:** `src/`, `tests/`
- **Chore:** Shift Rust test suite methodology away from implementation-dependent, brittle unit tests (which break during frequent refactoring and contribute little value) towards a TDD-adjacent testing paradigm centered around narrow functional tests and broad end-to-end happy path tests.
- **Action Plan:**
  - Audit existing test suite to deprecate/refactor brittle internal state assertions.
  - Implement narrow functional tests for domain logic (e.g., User-Agent coincidence, config serialization, resolver routing, certificate parsing) that verify external contract behavior rather than internal private helpers.
  - Expand broad happy-path integration tests verifying full workflows (e.g., URI resolution chain, config generation, and ban page rendering).

### 21. Remote Fedora RPM Repository Distribution Pipeline (Staged DDNS -> Handshake Rollout)
- **Files:** `build_rpm.sh`, `juanita.repo`, `docs/spec/DISTRIBUTION.md`
- **Chore:** Establish remote automated repository hosting infrastructure for Juanita Banana Fedora RPM builds (`fedora.repo.randºm`).
- **Action Plan:**
  - **Stage 1 (DDNS + HTTP / GPG)**: Configure remote Ubuntu server with `createrepo_c` and Caddy. Sync built RPMs via `rsync` over SSH. Enable OpenPGP package signing (`gpgcheck=1`, `gpgkey=...`) and serve initial release repository via DDNS.
  - **Stage 2 (Handshake DNF Plugin)**: Develop `dnf-plugin-hns` (DNF4/DNF5 Python plugin) that invokes lightweight Rust Handshake resolution (`hnsd` / custom HNS resolver) in-memory to resolve `.hns` domains (`repo.juanita.hns` / `fedora.repo.randºm`) without requiring system-wide DNS reconfigurations or full nodes.
  - **Stage 3 (Decentralized TLS with `randbotd`)**: Integrate `randbotd` for automatic decentralized TLS certificate verification and trust establishment on Handshake names.

### 22. Replace Text Input with Native GTK FileChooser Dialog for Download Location
- **Files:** `templates/pages/config.html`, `scripts/js/config.js`, `src/browsing/internal/config_pages/config.rs`
- **Chore:** Replace the plain text `<input type="text" id="permanent-download-dir">` field in `juanita://config` with a native GTK FolderChooser / FileChooser dialog button to allow users to graphically pick their target download directory without typing manual paths.
- **Action Plan:**
  - Update `templates/pages/config.html` to add a "Browse..." button alongside the location path display.
  - Implement GTK IPC/webcontent handler for launching `GtkFileChooserNative` / `GtkFileChooserDialog` in folder selection mode.