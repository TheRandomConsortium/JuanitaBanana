# The Punisher Browser Manifesto
# aka: Juanita Banana 🍌

## Core Philosophy

We adamantly refuse to use Google's engine (Blink/Chromium). This browser is ultra bare-metal, built in native compiled code (Rust) to talk directly to the machine. It runs blazing fast, has the most spartan interface in existence, and shows absolutely no mercy to the modern web.

**Engine:** WebKitGTK (pragmatic fallback — goal is to return to Servo once its HTTP/2 stack matures).
**UI:** GTK3 native — HeaderBar, Entry, Button. Minimal, spartan, no fluff.

---

## ⚖️ Declaration of Intent (Read before Judging)

> **We do not pretend to waste money or harm any company.**
>
> Every feature in this browser — search noise, ad interaction, fingerprint spoofing — exists for one purpose only: **to prevent behavioral profiling of the user**. We are not trying to drain advertiser budgets. We are making the user's data profile useless as a signal. The economic side-effect is incidental to the privacy act, in the same way that Tor's traffic mixing is incidental to the bandwidth it consumes on exit nodes.
>
> This is the same philosophy as [AdNauseam](https://adnauseam.io/) (developed by privacy researchers, banned by Google from the Chrome Web Store — make of that what you will) and academic research on search query obfuscation.
>
> **Intent: anti-fingerprinting. Not sabotage.**

### § The Corporate Clause

> "Our mascot is a derpy banana with curly hair. When your fingerprinting fails, you will have to look at it. When your quarterly metrics crash, you will have to explain it. When your profiling model collapses, you will have to project it in your board meeting.
>
> This is not a bug. This is a feature."

---

## Rules of Engagement

### 1. War on Fingerprinting (Active Anti-Espionage)

**Critical Design Principle:** The DOM always receives real data so that browsing remains indistinguishable from any modern browser (Brave, Firefox). Fake data is only sent to tracking APIs. The user never notices the difference, but the tracker goes blind.

- **Canvas Fingerprinting:** When a website attempts canvas fingerprinting, we intercept `toDataURL()`, `toBlob()`, AND `getImageData()` (all three read paths). We return a static Base64 image — pixel noise alone is insufficient because `getImageData` reads the real underlying buffer. Guaranteed poisoning across all extraction methods.
- **Viewport Fingerprinting:** `screen.width/height`, `window.innerWidth/Height`, and other APIs report randomized dimensions. The engine renders with real dimensions — the DOM renders correctly, the tracker receives garbage.
- **WebGL Fingerprinting:** `UNMASKED_VENDOR_WEBGL` → "Juanita Banana GPU". `UNMASKED_RENDERER_WEBGL` → "JB Renderer (Not-Google)".
- **Navigator Fingerprinting:** `hardwareConcurrency`, `deviceMemory`, `platform`, `vendor`, `userAgent`, `webdriver=false`, `languages` and a mock `plugins` array all return controlled fake values.
- **iFrame Bypass:** The script is injected into ALL sub-frames, not just the top frame. Trackers routinely spin up invisible iframes to read the clean OS navigator and bypass main-frame protections. This closes that vector.
- **Timezone / Geolocation Leak:** `Intl.DateTimeFormat().resolvedOptions().timeZone` is overwritten. Even when all other signals are spoofed, the real timezone pinpoints the user's physical location. We freeze it to a neutral value.
- **Battery API Fingerprinting:** `navigator.getBattery()` is overwritten to always report 100% level and charging status, preventing battery-draining telemetry and device tracking.
- **Session-Randomized Configurable Spoofing (Planned):** Instead of static values or fully dynamic variables that mutate on every API call (exposing inconsistencies), spoofing parameters will be session-randomized. Values (like screen size range, cores, memory) are randomized once per browser session within logical limits, and are configurable in settings.

### 2. Search Profile Obfuscation (Search Intoxication)

For every real search the user makes, the browser will execute 20 additional searches on heterogeneous topics so that the user's search profile becomes statistically useless for any data broker.

*The goal is to turn the profile into pure noise, not to interfere with the search service.*

### 3. Inverse Advertising Framework (Ad Profile Obfuscation)

- **Concealment and Blind Interaction:** Ads are hidden from the user's view. The browser interacts with them in the background so that the user's interest profile gets completely contaminated: if you click on everything, you are nobody.
- *The goal is for the interest profile built around the user to become statistical garbage. Not to financially harm any advertiser.*
- **Full-Screen Ads (Anti-Ad Blockers):** The internal timer will be respected so as not to brick the website's functionality, but visually the screen will be replaced by a banner saying:
  > *"This website is trying to block you with an ad. Ban?"*

### 4. Punishing the Web (The Ban System)

- **Banned Websites:** Any attempt to enter a banned website will route locally to a static HTML: *"You blocked this website before. Go look for greener pastures elsewhere."*
- **Toxic Websites:** If the system declares a website as toxic, an annoying marquee will crawl across the screen: *"This website is toxic. Ban?"* Additionally, a **Guilt Trip Overlay** will be injected—a contextual, semi-transparent image (e.g., a "Fake News" meme for news sites, or "Ceiling Cat" for NSFW pages) covering the screen with a low alpha (e.g., hex `30`), allowing interaction but constantly reminding the user of their poor choices. This module will be toggleable in the configuration.
- **Ban Button:** Visible in the navigation bar at all times.

### 5. "AI Slop" Detection

If an article is detected to have been generated by AI (footer signals, metadata, text patterns), the title will be replaced by: *"This newspaper uses AI Slop. Ban?"*

### 6. Improbable Redemption (Undocumented Config)

- Banned websites are saved in `juanita://config` — hidden, undocumented.
- **Unbanning:** Requires solving a complex mathematical equation or a technical challenge. No easy toggle switches.

---

## 🔭 Future Arsenal (Advanced Tracking Ecosystem Intoxication)

### Tracking Ecosystem Poisoning

#### Google Tag Manager (GTM)
GTM is a universal middleman that allows any website to inject arbitrary tracking code without touching their own server. **Strategy:** Detect the loading of `googletagmanager.com/gtm.js`, intercept calls to the dataLayer, and poison the events with fake data before they reach downstream analytics scripts (GA4, Hotjar, etc.).

#### Meta Pixel (Facebook Tracking Pixel)
Meta's pixel records every pageview, every purchase, every scroll and links it to your Facebook profile even if you aren't logged in. **Strategy:** Detect `connect.facebook.net/signals/`, block the real payload and inject a fake pixel that reports random events (fictitious purchases, fake pageviews) to contaminate the advertiser's lookalike audience.

#### Alibaba / AliExpress Tracking Suite
Alibaba has its own analytics stack (`aplus.yunpik.com`, `log.mmstat.com`) with aggressive fingerprinting. **Strategy:** Same interception as GTM/Meta — block real beacons, send synthetic data.

#### Cookie Poisoning
Tracking cookies (non-functional ones) will be filled with garbage data for the duration of the session. Upon leaving the page (or closing the tab), they are completely wiped. The tracker maintains a cookie that serves absolutely no purpose.

#### Beacon API Interception
`navigator.sendBeacon()` is the preferred mechanism for trackers to send data upon page exit (impossible to cancel with fetch abort). **Strategy:** Override `navigator.sendBeacon` to intercept, modify the outgoing data with noise, and send it.

#### Browser Fingerprint via Font Enumeration
Some trackers enumerate installed fonts to create a fingerprint. **Strategy:** Return a spoofed set of standard fonts.

#### Web Audio API Spoofing
Acoustic signatures generated by audio hardware are used for tracking (AudioContext oscillator fingerprinting). **Strategy:** Return procedurally generated acoustic signatures on the fly to simulate 1980s Soviet sound cards, injecting inaudible but mathematically distinct noise on every render.

#### Sensor API Poisoning
Accelerometers and gyroscopes expose microscopic movement data on mobiles and laptops. **Strategy:** Inject synthetic data so that Meta's algorithms (and others) believe the user is in constant freefall or walking in an infinite spiral.

#### Typing Biometrics Corruption
The timing between keystrokes (keydown, keyup) is a unique biometric that identifies a user. **Strategy:** Intercept keyboard events and introduce random millisecond delays (jitter) to destroy the AIs trying to profile your typing cadence.

### Identity & Search

#### Search Intoxication — P2P Upgrade
**Phase 1 (current):** Local list of 1000+ heterogeneous searches from which 20 are randomly chosen per real search.

**Phase 2 — P2P Search Pool:**
Users wishing to participate contribute their real (anonymized) searches to a shared pool. Two possible implementations:
- **Dumb Pipe with TTL:** Minimalist server (pastebin style) where clients post searches with a short TTL (24-48h). No logs, no persistence, no IPs. The server is just a dumb pipe.
- **Gossip Protocol:** No central server. Each Juanita Banana node opting in propagates searches to its peers with a hop TTL. Similar to how BitTorrent DHTs work. Completely decentralized.

The result: your fake searches are the real searches of other users — maximum authenticity for the algorithm, maximum chaos for the profile.

#### Identity Management
- **Integrated Password Manager** — native, no extensions, no cloud.
- **Tor Integration** — optional SOCKS5 proxy. We aren't on Tor all day, but we can be.

#### Weaponized Privacy (GDPR Art. 17)
- **Aggressive Unsubscribe Button** — A local crawler that scans subdomains, legal texts, and footers to extract contact emails. It automatically sends formal legal requests demanding the deletion of all personal data (Right to be Forgotten, GDPR Article 17) and keeps a local registry of unsubscribed services. If a service fails to respond or comply, the tool generates a PDF with a formal complaint ready to be sent to European data protection authorities. Requires a local form with the user's data (EU citizens only).

### ⚠️ The Circle Jerk Protocol integration (Ecosystem upgrade)
The browser's upcoming architecture features direct integration with the `circlejerk-cli` daemon and the P2P swarm, transforming it into `juanita-circlejerk`.
- **Swarm Control:** A physical UI toggle in the settings allows the user to opt-in and spin up/terminate the P2P daemon.
- **Hot Potato Ephemerality:** Shared files are stored solely in volatile `Arc<RwLock<HashMap>>` memory buffers, avoiding forensically auditable physical disk writes.
- **Censorship Immunity (Judges Consortium):** Local 4-bit Vision-Language Models (VLM) running JIT abliteration audit transfers blindly via 1024-dimensional semantic embeddings (`Vec<f32>`).
- **Monero Spam Prevention:** Ties file distribution to self-addressed Monero transactions (`tx_extra` field holding file SHA-256 hashes) to prevent network pollution.
