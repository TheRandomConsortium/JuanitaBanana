# 🍌 Juanita Banana — Features Tracker

| Status | Meaning |
|:---:|---|
| ✅ | **Done**: Fully implemented and tested. |
| 🔨 | **WIP**: Currently being worked on. |
| 📋 | **Planned**: Accepted into the roadmap, pending implementation. |
| 🔭 | **Future**: Conceptual idea, needs research or architecture design. |
| ❓ | **Doubtful**: May be implemented however might not be within the project philosophy |

---

## 🛡️ Active Anti-Fingerprinting (DOM & JS APIs)

| Feature | Status | Notes |
|---|---|---|
| **Canvas Spoofing** | ✅ Done | Intercepts `toDataURL`, `toBlob`, AND `getImageData`. Returns a static Base64 image — pixel noise was insufficient as `getImageData` reads the raw buffer. |
| **Viewport Dimensions Spoofing** | ✅ Done | `screen.width/height`, `window.innerWidth/innerHeight` report randomized dimensions. The engine renders with real dimensions — DOM is unaffected. |
| **WebGL Spoofing** | ✅ Done | Overrides `UNMASKED_VENDOR_WEBGL` and `UNMASKED_RENDERER_WEBGL` with "Juanita Banana GPU". |
| **Navigator API Spoofing** | ✅ Done | Fake values for `hardwareConcurrency`, `deviceMemory`, `platform`, `vendor`, `userAgent`, `webdriver=false`, `languages`, and `plugins` mock array. |
| **iFrame Sub-frame Injection** | ✅ Done | Script injected into ALL frames via `UserContentInjectedFrames::AllFrames`. Trackers spin up invisible iframes to read the clean OS navigator — this closes that bypass. |
| **Intl / Timezone Leak** | ✅ Done | Overwrites `Intl.DateTimeFormat().resolvedOptions().timeZone`. Without this, the real timezone exposes physical location even when all other signals are spoofed. |
| **Battery API Spoofing** | ✅ Done | Override `navigator.getBattery()` to always report 100% charging and charging status. |
| **Session-Randomized Spoofing** | 📋 Planned | Make spoofed parameters (memory, cores, dimensions, platform) configurable and session-randomized. Values remain static within a session to prevent inconsistency tracking, but mutate on launch within logical boundaries. |
| **Font Enumeration Protection** | ✅ Done | Overrides canvas text measurements and CSS font loading to report that only `Webdings` and `Monospace` are installed (until session-randomized font pools are implemented). |
| **Prototype toString Protection** | 📋 Planned | Override `Function.prototype.toString` to return standard native function representation (`function () { [native code] }`) when called on any spoofed/overridden APIs, avoiding detection by bot detectors verifying `toString` outputs. |
| **Proxy-based API Hooking** | 📋 Planned | Migrate all manually overridden/monkeypatched properties and methods to use ES6 `Proxy` wrappers, creating a more uniform, robust, and clean interception system. |
| **Web Audio API Spoofing** | 🔭 Future | Procedurally generated acoustic signatures (Soviet 1980s sound card) to poison oscillator fingerprinting. |
| **Sensor API Poisoning** | ✅ Done | Poison devicemotion, deviceorientation, and W3C generic sensor APIs (Accelerometer, Gyroscope, etc.) with synthetic spiral walking data to prevent device-movement fingerprinting. |
| **Typing Biometrics Corruption** | 🔭 Future | Introduce random millisecond jitter in `keydown`/`keyup` events to destroy typing cadence AI profiling. |
| **Drunk Mouse Movement** | 🔭 Future | Introduce randomized offsets or slight delays to cursor movement events to sabotage mouse-tracking behavior analysis. |

---

## 🕸️ Network & Tracking Contamination

| Feature | Status | Notes |
|---|---|---|
| **User-Agent Override** | ✅ Done | Honest UA: `JuanitaBanana/0.1 (FOSS; Not-Google; Linux)` via WebKit's network settings. |
| **User-Agent Spoofing Toggle** | 📋 Planned | A config toggle to switch between honest UA and daily rotation of genuine modern UAs. |
| **Cookie Poisoning** | 🔭 Future | Fill non-essential tracking cookies with garbage data during session, wipe on exit. |
| **Cookie Containers** | 🔭 Future | Optional cookie segregation. Mixing cookies is often more toxic/polluting. Only implemented if/when multi-tab support is added (makes no sense with current single-tab, single-session strategy). |
| **Google Tag Manager Poisoning** | 🔭 Future | Intercept `dataLayer.push()` and inject fake events before downstream dispatch. |
| **Meta Pixel Contamination** | 🔭 Future | Intercept `fbq` calls, block real payload, send random events (fake purchases) to ruin lookalike audiences. |
| **Beacon API Interception** | 🔭 Future | Override `navigator.sendBeacon()` to modify outgoing data with noise. |
| **Alibaba Tracking Suite Poisoning** | 🔭 Future | Intercept `aplus.yunpik.com` / `log.mmstat.com` beacons. |
| **CIAM Parallel Session Poisoning** | 🔭 Future | Detect identity trackers (e.g., `dru-id.com`) and launch a sandboxed background page to algorithmically navigate and fire fake schema.org events (simulated purchases, scroll actions) using the authenticated cookies to poison the captured profile. Includes a manual **"Parallel Browse" right-click context menu option** to deploy this background traverse manually on any arbitrary site. |

---

## 📣 Ad Profile Obfuscation (Inverse Advertising Framework)

| Feature | Status | Notes |
|---|---|---|
| **Blind Background Interaction** | ✅ Done | Intercepts ads dynamically via DOM prototype setters and Fetch/XMLHttpRequest monkeypatching. Removes elements from screen and executes sequential (1-by-1) simulated interactions (scrolling, hovers, clicks) in a headless WebKit WebView to pollute target profile metrics. |
| **Manual Ad Reporting & Verification** | ✅ Done | Context-menu action allowing users to mark elements as ads. Presents a verification dialog listing candidate URLs (to prevent false positive blocking of main websites) before learning the domain and triggering surgery/poisoning. |
| **Anti-Ad Blockers (Full-Screen Ads)** | 🔭 Future | Replaces aggressive ad-block walls with a warning banner, respecting internal site timers to prevent breaking functionality while refusing to show the ad itself. |

---

## 🔍 Search Intoxication

| Feature | Status | Notes |
|---|---|---|
| **Local Search Noise** | ✅ Done | Fires a configurable amount (default 20, exposed as `noise_queries_amount` in config) of background searches from a local heterogeneous pool using dynamic RSS n-grams for every real user search, effectively poisoning the data profile. |
| **P2P Gossip Protocol** | 🔭 Future | Decentralized sharing of anonymized searches to use real user data as noise for everyone. |
| **Dumb Pipe TTL Server** | 🔭 Future | Minimalist server for search pool as an alternative to Gossip Protocol. |
| **Background Captcha Solver** | 📋 Planned | Detects when a hidden WebView encounters a Captcha or a "Consent to Cookies" wall. Automated solving strategies include simple auto-click, local VLM heuristics, and integration with the [Juanita Companion](https://github.com/TheRandomConsortium/JuanitaBananaCompanion) Android app (via foreground heartbeat polling to pull reCAPTCHA v3 QR codes and trigger Android Accessibility auto-click). If all automated strategies fail, it triggers a humiliating fallback popup featuring the Juanita Banana icon stating: *"Woohoo [SearchEngine] got a Boo Boo and seems to think solving a captcha will help them"*, displays the captcha, and offers a secondary button: *"Or maybe you prefer to ban?"*. |

---

## 🚫 The Ban System

| Feature | Status | Notes |
|---|---|---|
| **Persistent Ban List** | ✅ Done | Domains saved to `~/.local/share/juanita-banana/banlist.bin` (cryptographically signed). |
| **Local Static Ban Page** | ✅ Done | Banned domains route to a local HTML error page. |
| **UI Ban Button** | ✅ Done | GTK HeaderBar button. Bans current domain and loads ban page immediately. |
| **Toxic Site Warning** | ✅ Done | Injects a fixed bottom marquee warning users when a site exceeds the `toxic_threshold` of combined ads and trackers, with a button to ban the domain immediately. *Future improvement: implement smarter heuristics (e.g., distinguishing raw volume vs content ratios like news articles vs ad blocks) instead of a simple flat threshold.* |
| **Contextual Guilt Trip Overlay** | ✅ Done | Loaded real high-fidelity meme assets (Ceiling Cat, Trump, Fry, Wojak, and Banana fallback) at compile time and injected a semi-transparent, non-blocking contextual meme overlay on pages matching user-configurable keyword rules. |
| **Mathematical Unban** | ✅ Done | Requires solving a randomly generated integral challenge in `juanita://unban?domain=...` to unban a site. Fully integrated with `BanList` and Vengeful Mode. |

---

## 🤖 AI Slop Detection

| Feature | Status | Notes |
|---|---|---|
| **Detect "Written with AI" footer** | 📋 Planned | DOM scan for known AI disclosure strings. |
| **Replace article title** | 📋 Planned | Inject "This newspaper uses AI Slop. Ban?" into the DOM. |
| **confer.to Recommendation** | 📋 Planned | Officially recommend `confer.to` when AI slop is detected, with a message like: "You might as well create this yourself and do it privately in the meantime." |

---

## 📦 Sandboxed Downloads

| Feature | Status | Notes |
|---|---|---|
| **Fake Downloads to Sandbox** | ✅ Done | Clicking a download link doesn't truly download the file. It saves it to a temporary `tmpfs` RAM disk that can only be opened inside a restricted sandbox environment using `bwrap` (Bubblewrap) preventing network and home directory access. |
| **`juanita://downloads` Persistence** | ✅ Done | If the user decides to keep the file permanently and open it on their real OS, they must navigate to `juanita://downloads` and manually persist the file. Otherwise, it is securely deleted. Features native OS notifications and progress bars. |

---

## ⚖️ Weaponized Privacy

| Feature | Status | Notes |
|---|---|---|
| **Aggressive Unsubscribe (GDPR Art. 17)** | ✅ Done | Local crawler to extract contact emails, send formal Right to be Forgotten requests, and generate PDF complaints if ignored. |
| **CAdES Digital Signing (eIDAS)** | ✅ Done | GDPR Art. 77 complaints can be digitally signed in-RAM (private key never touches disk) using any eIDAS-qualified PKCS#12 certificate (FNMT, idCAT, Izenpe, Camerfirma, D-Trust, etc.), producing a legally valid `.p7m` envelope (ETSI TS 101 903 / RFC 5652) accepted by all EU national DPAs. Certificate is stored in the same Argon2id + XChaCha20-Poly1305 encrypted vault. |
| **Non-HTML Legal Document Scanning** | 📋 Planned | Scan linked PDFs, Word documents, and text files for contact/DPO emails using local text extraction libraries. |
| **DPA Online Form Auto-Submission** | 📋 Planned | Auto-submit the signed `.p7m` complaint directly to the national DPA's online portal by parsing their HTML submission forms and auto-filling pre-populated fields from the generated report. Target authorities: AEPD, Garante, CNIL, BfDI, ICO, DPC, APD/GBA, EDPB. |
| **Future Auto-reporting Option** | 📋 Planned | Auto-reporting of reincident domains to Supervisory Authorities by parsing inbox confirmation emails via POP/IMAP and auto-submitting complaints. |
| **POP3 Mail Client & Secure Sandbox (`juanita://mail`)** | 📋 Planned | Fetch incoming email via POP3, display it natively in Juanita with contextual *Add Ban* / *Unsubscribe* banners, and open attachments in an isolated Bubblewrap (`bwrap`) secure sandbox. |

---

## 🌐 Overlay Networks & Decentralised Resolver Stack

> Full architecture: [`docs/OVERLAY_NETWORKS.md`](OVERLAY_NETWORKS.md)

### Overlay Transports

| Feature | Status | Notes |
|---|---|---|
| **Tor (onion routing)** | 📋 Planned | Integrated via [`arti`](https://gitlab.torproject.org/tpo/core/arti) — Tor Project's official Rust crate. No C dependency, no subprocess. Activating it: registers the `.onion` resolver; optionally routes all clearnet through Tor exit nodes. |
| **I2P (garlic routing)** | 🔭 Future | Integrated via `i2p-rs` (Rust) when stable; subprocess fallback to Java I2P router initially. Garlic routing bundles multiple messages per payload — harder to traffic-analyse than Tor. Activating it: registers the `.i2p` resolver; optionally routes clearnet via I2P outproxies. |

### Resolver Stack

| Feature | Status | Notes |
|---|---|---|
| **BIOS-style resolver chain** | ✅ Done | Priority-ordered chain: first resolver with an authoritative answer wins, rest skipped. Default order: Handshake → System DNS. Fully user-reorderable in `juanita://config`. |
| **Handshake (HNS) resolver** | ✅ Done | Permissionless blockchain root DNS, parallel to ICANN. Integrated via local `hnsd` daemon managed automatically by the browser. |
| **Onion resolver** | 📋 Planned | Resolves `.onion` v3 addresses when Tor transport is active. |
| **I2P resolver** | 🔭 Future | Resolves `.i2p` eepsite addresses when I2P transport is active. |
| **Namespace collision handling** | ✅ Done | HNS and ICANN can both define same names. Resolver priority chain is the user's tiebreak — whoever is first in your chain is authoritative for you. |
| **Per-domain pinning rules** | 🔭 Future | User rules in config: `example.bit → always Handshake`, `*.onion → always Tor`. Pinned domains bypass the chain entirely. |
| **Non-blocking resolver fallback retry** | 🔭 Future | Once a resolver in the chain fails its first attempt, allow subsequent resolvers to start trying immediately (liberating the chain) while the initial resolver continues retrying in the background. If it eventually succeeds, it loads; if another resolver completes first or the user navigates away, background retries are stopped. |
| **Navbar resolver override dropdown** | 🔭 Future | Provide a dropdown selector on the navbar showing the available resolvers + search, enabling the user to perform a quick one-off override for the current navigation. |

### Niche Protocols (Under Evaluation)

| Protocol | Address space | Notes |
|---|---|---|
| **Lokinet** | `.loki` | Session messenger's overlay; C++ (`llarp`) |
| **Namecoin** | `.bit` | Original blockchain DNS; largely superseded by HNS |
| **ENS / IPFS** | `.eth` | Ethereum Name Service; requires ETH node or trusted gateway |
| **Yggdrasil** | IPv6 mesh | Encrypted mesh overlay; no special TLD |

---

## 🔒 Privacy Hardening (Engine Level)

| Feature | Status | Notes |
|---|---|---|
| **Disable hyperlink auditing (`ping`)** | ✅ Done | Via WebKit settings. |
| **Disable DNS prefetch** | ✅ Done | Via WebKit settings. |
| **Disable JS popup windows** | ✅ Done | Via WebKit settings. Intercepts `target="_blank"` to force opening in the same window. |
| **Make Default Browser** | ✅ Done | Button in the new General tab of `juanita://config` to register Juanita Banana as the default system web browser using `xdg-settings`. |
| **Choose Competitor (Betrayal Mode)** | ✅ Done | If Juanita is already default, displays a "Choose Competitor" button. Clicking it dynamically loads native system desktop entries (`.desktop` via `gio mime`) and their icons, allowing the user to betray the banana and revert to Firefox/Chromium etc. |
| **RPM Packaging & Version Bumping** | ✅ Done | Included `build_rpm.sh` script to parse Cargo versions, build a `.spec` dynamically, build RPM packages, and auto-increment `major`, `minor`, or `patch` tags automatically. |
| **Integrated Password Manager** | ✅ Done | Native secure credential storage using local Argon2id + XChaCha20-Poly1305 encrypted SQLite. Per-domain username + email saved in the vault. Credential availability is hinted to the browser via an unencrypted `credindex.bin` index (domain names only — no passwords) so no vault decryption is needed to detect that a site has saved credentials. |
| **Address Bar Focus Redaction** | ✅ Done | Hide query parameters completely when the address bar is unfocused. Redact sensitive query parameters (e.g. `unlock_pass`, `session`, `password`) when focused to prevent credential exposure in the browser UI. |
| **Safe Password Suggestion** | ❓ Doubtful | If sufficiently requested, a password generator that picks 25 random words from local RSS feeds using a physical dice roll algorithm (repeats allowed) separated by spaces until the password is at least 100 characters long, then appends `"ThI$I$0nLyH€R€T0Pl€a$€Y0Ur$tUpIdR€G€X"` at the end to satisfy broken complexity validation scripts. |
| **Local HTML File Viewer** | ✅ Done | Opening a local `.html` / `.htm` file shows a dark-themed source editor instead of rendering it blindly. A warning bar instructs the user to inspect the file before trusting it. A **Render** button renders it on demand; a **Save** button writes edits back to disk. Default behaviour (`edit` vs `render`) is configurable in `juanita://config`. |
| **TOTP / Time-Based 2FA** | 🔭 Future | Store TOTP secret keys in the encrypted vault and derive one-time codes on demand. **Intentionally deferred:** storing 2FA on the same device as the account credential undermines the second-factor guarantee. Will only be implemented via a `juanita-companion` mobile app that is notified when a TOTP is requested, generating it on a physically isolated device. |
| **Tab Inactivity TTL (Tab Death)** | ✅ Done | Customizable tab inactivity Time-To-Live (TTL) with deferred cleanup and last tab protection policies. Visually reordering occurs only when user focus is restored. |
| **Per-Domain Exception Profiles** | ❓ Doubtful | Introduce exception configs to allow unhardening specific spoofing/header protections for target websites. Accompanied by a "This is fine" dog meme config UI that dynamically grows more distressing as the user disables protections. |
