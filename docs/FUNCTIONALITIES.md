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
| **Canvas Spoofing** | ✅ Done | Intercepts `toDataURL` and `toBlob`. Returns a pre-rendered `phallus.jpg` to poison the fingerprint reliably without breaking DOM layout. |
| **Viewport Dimensions Spoofing** | ✅ Done | `screen.width/height`, `window.innerWidth/innerHeight` report randomized dimensions while Servo continues to render with real dimensions. |
| **WebGL Spoofing** | ✅ Done | Overrides `UNMASKED_VENDOR_WEBGL` and `UNMASKED_RENDERER_WEBGL` with "Juanita Banana GPU". |
| **Navigator API Spoofing** | ✅ Done | Hardcoded fake values for `hardwareConcurrency`, `deviceMemory`, `platform`, and `vendor`. |
| **Battery API Spoofing** | 📋 Planned | Override `navigator.getBattery()` to always report 100% charging. |
| **Font Enumeration Protection** | 📋 Planned | Override canvas text measurements and CSS font loading to report a standard, fake set of fonts. |
| **Web Audio API Spoofing** | 🔭 Future | Procedurally generated acoustic signatures (Soviet 1980s sound card) to poison oscillator fingerprinting. |
| **Sensor API Poisoning** | 🔭 Future | Inject synthetic accelerometer/gyroscope data simulating constant freefall or spiral walking. |
| **Typing Biometrics Corruption** | 🔭 Future | Introduce random millisecond jitter in `keydown`/`keyup` events to destroy typing cadence AI profiling. |

---

## 🕸️ Network & Tracking Contamination

| Feature | Status | Notes |
|---|---|---|
| **User-Agent Override** | ✅ Done | Custom UA: `JuanitaBanana/0.1 (FOSS; Not-Google; Linux)` via Servo `EmbedderMethods`. |
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
| **UI Ban Button** | 🔨 WIP | Needs integration into the egui toolbar. |
| **Toxic Site Warning** | 🔭 Future | Inject an annoying marquee across sites identified as toxic. |
| **Mathematical Unban** | 🔭 Future | Require solving a complex equation in `juanita://config` to unban a site. |

---

## 🤖 AI Slop Detection

| Feature | Status | Notes |
|---|---|---|
| **Detect "Written with AI" footer** | 📋 Planned | DOM scan for known AI disclosure strings. |
| **Replace article title** | 📋 Planned | Inject "This newspaper uses AI Slop. Ban?" into the DOM. |

---

## 🔒 Privacy Hardening (Engine Level)

| Feature | Status | Notes |
|---|---|---|
| **Disable hyperlink auditing (`ping`)** | ✅ Done | Via Servo settings. |
| **Disable DNS prefetch** | ✅ Done | Via Servo settings. |
| **Disable JS popup windows** | ✅ Done | Via Servo settings. |
| **Tor integration** | 🔭 Future | SOCKS5 proxy toggle in config. |
| **Integrated Password Manager** | 🔭 Future | Native credential management, no cloud. |
