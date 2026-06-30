# 🍌 Juanita Banana — Features Tracker

| Status | Meaning |
|:---:|---|
| ✅ | **Done**: Fully implemented and tested. |
| 🔨 | **WIP**: Currently being worked on. |
| 📋 | **Planned**: Accepted into the roadmap, pending implementation. |
| 🔭 | **Future**: Conceptual idea, needs research or architecture design. |

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
| **Battery API Spoofing** | 📋 Planned | Override `navigator.getBattery()` to always report 100% charging. |
| **Font Enumeration Protection** | 📋 Planned | Override canvas text measurements and CSS font loading to report a standard, fake set of fonts. |
| **Web Audio API Spoofing** | 🔭 Future | Procedurally generated acoustic signatures (Soviet 1980s sound card) to poison oscillator fingerprinting. |
| **Sensor API Poisoning** | 🔭 Future | Inject synthetic accelerometer/gyroscope data simulating constant freefall or spiral walking. |
| **Typing Biometrics Corruption** | 🔭 Future | Introduce random millisecond jitter in `keydown`/`keyup` events to destroy typing cadence AI profiling. |
| **Drunk Mouse Movement** | 🔭 Future | Introduce randomized offsets or slight delays to cursor movement events to sabotage mouse-tracking behavior analysis. |

---

## 🕸️ Network & Tracking Contamination

| Feature | Status | Notes |
|---|---|---|
| **User-Agent Override** | ✅ Done | Honest UA: `JuanitaBanana/0.1 (FOSS; Not-Google; Linux)` via WebKit's network settings. |
| **User-Agent Spoofing Toggle** | 📋 Planned | A config toggle to switch between honest UA and daily rotation of genuine modern UAs. |
| **Cookie Poisoning** | 🔭 Future | Fill non-essential tracking cookies with garbage data during session, wipe on exit. |
| **Google Tag Manager Poisoning** | 🔭 Future | Intercept `dataLayer.push()` and inject fake events before downstream dispatch. |
| **Meta Pixel Contamination** | 🔭 Future | Intercept `fbq` calls, block real payload, send random events (fake purchases) to ruin lookalike audiences. |
| **Beacon API Interception** | 🔭 Future | Override `navigator.sendBeacon()` to modify outgoing data with noise. |
| **Alibaba Tracking Suite Poisoning** | 🔭 Future | Intercept `aplus.yunpik.com` / `log.mmstat.com` beacons. |

---

## 🔍 Search Intoxication

| Feature | Status | Notes |
|---|---|---|
| **Local Search Noise** | 📋 Planned | Fire 20 background searches from a local heterogeneous pool for every real user search. |
| **P2P Gossip Protocol** | 🔭 Future | Decentralized sharing of anonymized searches to use real user data as noise for everyone. |
| **Dumb Pipe TTL Server** | 🔭 Future | Minimalist server for search pool as an alternative to Gossip Protocol. |

---

## 🚫 The Ban System

| Feature | Status | Notes |
|---|---|---|
| **Persistent Ban List** | ✅ Done | Domains saved to `~/.local/share/juanita-banana/banlist.json`. |
| **Local Static Ban Page** | ✅ Done | Banned domains route to a local HTML error page. |
| **UI Ban Button** | ✅ Done | GTK HeaderBar button. Bans current domain and loads ban page immediately. |
| **Toxic Site Warning** | 🔭 Future | Inject an annoying marquee across sites identified as toxic. |
| **Mathematical Unban** | 🔭 Future | Require solving a complex equation in `juanita://config` to unban a site. |

---

## 🤖 AI Slop Detection

| Feature | Status | Notes |
|---|---|---|
| **Detect "Written with AI" footer** | 📋 Planned | DOM scan for known AI disclosure strings. |
| **Replace article title** | 📋 Planned | Inject "This newspaper uses AI Slop. Ban?" into the DOM. |

---

## ⚖️ Weaponized Privacy

| Feature | Status | Notes |
|---|---|---|
| **Aggressive Unsubscribe (GDPR Art. 17)** | 🔭 Future | Local crawler to extract contact emails, send formal Right to be Forgotten requests, and generate PDF complaints if ignored. |

---

## 🔒 Privacy Hardening (Engine Level)

| Feature | Status | Notes |
|---|---|---|
| **Disable hyperlink auditing (`ping`)** | ✅ Done | Via WebKit settings. |
| **Disable DNS prefetch** | ✅ Done | Via WebKit settings. |
| **Disable JS popup windows** | ✅ Done | Via WebKit settings. |
| **Tor integration** | 🔭 Future | SOCKS5 proxy toggle in config. |
| **Integrated Password Manager** | 🔭 Future | Native credential management, no cloud. |
