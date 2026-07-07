<div align="center">

<img src="assets/icon.png" width="200" alt="Juanita Banana Icon">

```
     ██╗██╗   ██╗ █████╗ ███╗   ██╗██╗████████╗ █████╗
     ██║██║   ██║██╔══██╗████╗  ██║██║╚══██╔══╝██╔══██╗
     ██║██║   ██║███████║██╔██╗ ██║██║   ██║   ███████║
██   ██║██║   ██║██╔══██║██║╚██╗██║██║   ██║   ██╔══██║
╚█████╔╝╚██████╔╝██║  ██║██║ ╚████║██║   ██║   ██║  ██║
 ╚════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝   ╚═╝   ╚═╝  ╚═╝
        ██████╗  █████╗ ███╗   ██╗ █████╗ ███╗   ██╗ █████╗
        ██╔══██╗██╔══██╗████╗  ██║██╔══██╗████╗  ██║██╔══██╗
        ██████╔╝███████║██╔██╗ ██║███████║██╔██╗ ██║███████║
        ██╔══██╗██╔══██║██║╚██╗██║██╔══██║██║╚██╗██║██╔══██║
        ██████╔╝██║  ██║██║ ╚████║██║  ██║██║ ╚████║██║  ██║
        ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝
```

### 🍌 A browser that fights back.

![License](https://img.shields.io/badge/license-MPL--2.0-blue)
![Engine](https://img.shields.io/badge/engine-WebKitGTK-orange)
![Lang](https://img.shields.io/badge/lang-Rust-red)
![Google](https://img.shields.io/badge/Google-NOT%20WELCOME-critical)

</div>

---

## What is this?

Juanita Banana is a **bare-metal browser** built entirely in Rust, currently powered by **WebKitGTK** (with a long-term goal to return to [Servo](https://servo.org) once its HTTP/2 stack matures), with one mission: **make your browsing profile a statistical impossibility.**

It is not trying to be another browser. It is trying to be the browser the surveillance economy fears.

## Philosophy

The modern web has turned the browser into a data collection terminal. Canvas APIs, WebGL, viewport metrics, font enumeration, battery levels, search queries — every detail of your session is harvested, correlated and sold. Juanita Banana does not block this silently. It fights back actively.

> *We do not pretend to waste money or harm any company.*
> *We are making the user's data profile useless as a signal.*
> *Intent: anti-fingerprinting. Not sabotage.*

### § The Disco Ball Philosophy (Why We Do Not Send DNT)

> We do **not** send the `DNT` (Do Not Track) header signal on purpose. We refuse to plead with trackers.
>
> Our objective is **not** to blend in, to hide, or to be a quiet grey dot. Our objective is to be a **motherfucking disco ball with neon signs**. We want to be tracked, but we want the tracker's database to be filled with absolute, premium-grade, statistically-perfect **bullshit**.

### § The Corporate Clause

> "Our mascot is a derpy banana with curly hair. When your fingerprinting fails, you will have to look at it. When your quarterly metrics crash, you will have to explain it. When your profiling model collapses, you will have to project it in your board meeting.
>
> This is not a bug. This is a feature."

Read the full manifesto → [`docs/MANIFESTO.md`](docs/MANIFESTO.md)

## Core Principles

| Principle | Implementation |
|---|---|
| **No Google, ever** | Engine: WebKitGTK (pragmatic fallback). Not Blink. Not CEF. Not WebView2. |
| **Native binary** | Rust compiled with `-O3 -march=native`. CPU gets instructions, not a VM. |
| **Spartan by design** | No Electron. No Node. No Python. No browser decorations we didn't ask for. |
| **Privacy as offense** | We don't just block trackers. We poison their data. |
| **Render real, report fake** | DOM layout uses true viewport. JS tracking APIs see noise. |
| **User sovereignty** | You BAN a site, it's gone. Unban requires solving an equation. |

## Features

### 🛡️ Active Anti-Fingerprinting
- **Canvas:** Intercepts `toDataURL()` / `toBlob()` — trackers get a prerendered image, not your canvas
- **Viewport:** `screen.*` and `window.inner*` report randomized dimensions. Layout is unaffected.
- **Timezone:** `Intl.DateTimeFormat().resolvedOptions().timeZone` is overwritten to a neutral value (UTC) to prevent geographical location leaks.
- **Battery:** Overrides `navigator.getBattery()` to always report 100% charging and charging status.
- **WebGL:** Vendor/renderer strings replaced with Juanita Banana GPU
- **Navigator:** hardwareConcurrency, deviceMemory, platform — all spoofed
- **Fonts:** Overrides canvas text measurement and CSS font loading APIs to report that only `Webdings` and `Monospace` are installed.
- **User-Agent:** Honest by default (`JuanitaBanana/0.1`). In the future, a config toggle will allow switching to a rotating mode that cycles daily through a curated list of genuine, modern User-Agents to blend into the crowd perfectly.

### 🔍 Search Profile Obfuscation
- Every real search fires 20 background searches from a heterogeneous pool
- Future: P2P gossip protocol — your fake searches are real searches from other users

### 📣 Ad Profile Obfuscation (Inverse Advertising Framework)
- **Concealment and Blind Interaction:** Ads are hidden and surgically removed from the DOM. The browser monkeypatches prototype setters, fetch, and XHR APIs to catch background tracking intents.
- **Sequential Queuing:** Simulates realistic interaction (scrolling, hovers, clicks) in background WebKit WebView clones. Processing is strictly sequential (1-by-1) to replicate natural human behavior.
- **Manual Reporting & Verification:** Right-click context menu "Mark as Ad" featuring a candidate URL selector dialog to prevent false positive blocks of legitimate sites.
- **Anti-Ad Blockers (Full-Screen Ads):** Replaces aggressive ad-block walls with a warning banner, respecting internal site timers to prevent breaking functionality while refusing to show the ad itself (Planned).
- **Automated CAPTCHA Solver:** Detects CAPTCHA/consent screens in hidden WebViews. Solving strategies include auto-clicking, local VLM heuristics, and integration with the [Juanita Companion](https://github.com/TheRandomConsortium/JuanitaBananaCompanion) Android app (using foreground heartbeat polling to pull reCAPTCHA v3 QR codes and trigger Android Accessibility verify clicks). If all automated strategies fail, a humiliating fallback popup is displayed offering a domain ban.

### 🚫 The Ban System
- One click BAN from the navbar
- Banned sites load a local HTML page: *"Go look for greener pastures elsewhere."*
- To unban: solve a math equation. No easy toggles.
- **Toxic Websites:** Marks highly suspect sites with intrusive visual elements (Marquees and semi-transparent "Guilt Trip Overlays" like Fake News memes) to constantly remind the user of their poor choices without fully blocking access.

### 🤖 AI Slop Detection
- Detects AI-generated content disclosures → injects warning in the article title

### 🔮 Future Arsenal
GTM poisoning · Meta Pixel contamination · Cookie garbage fill · Beacon API interception · Tor integration · Password manager · P2P Circle Jerk Swarm · GDPR Aggressive Unsubscribe

See [`docs/FUNCTIONALITIES.md`](docs/FUNCTIONALITIES.md) for full status.

## Stack

```
juanita-banana
├── Engine:    WebKitGTK — Pragmatic, production-ready fallback for HTTP/2 SPAs
├── Meta:      Return to Servo once its experimental network stack matures
├── Window:    GTK3      — cross-platform, native UI
└── Spoofing:  UserContentManager — injects noise into WebKit's JS runtime
```

## Build

Builds are extremely fast since we are dynamically linking to your system's GTK3 and WebKit2GTK libraries.

```bash
# Prerequisites (Fedora)
sudo dnf install -y webkit2gtk4.1-devel gtk3-devel gcc-c++ make cmake pkgconf-pkg-config openssl-devel clang-devel

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
source ~/.cargo/env
cargo build --release

# Run
./target/release/juanita-banana
```

## Why WebKitGTK and not Gecko/Blink?

| Engine | Language | Our take |
|---|---|---|
| **Blink** (Chrome) | C++ | Google's engine. Absolutely not. |
| **Gecko** (Firefox) | C++ + Rust | No public embedding API for desktop. |
| **Servo** | **Rust** | Linux Foundation. Our true long-term goal. Currently shelved due to critical HTTP/2 `SendRequest` bugs with modern SPAs. |
| **WebKitGTK** | C++ | Apple/GNOME engine. Pragmatic, open-source fallback. Fast and reliable. |

## 🛑 Supported Platforms (The "Code It Yourself" Clause)

**We build software for sovereign operating systems. We do not support corporate spyware or walled gardens.**

### Desktop Suite (Juanita Banana & Circle Jerk Protocol)
* **Supported:** Linux.
* **Considered:** BSD (Real BSD, not Apple's proprietary bastardization).
* **Rejected:** Windows & macOS.

**Do NOT open issues requesting Windows or Mac ports.** Running a hyper-paranoid, anti-telemetry privacy suite on an operating system that keylogs you by default is a paradox. If you want that shit, fork the repo and code it yourself. And no, BSD being on the table is not an excuse to beg for a Darwin port *"because it's essentially the same"*. It is not.

## Contributing

Read the manifesto. If you agree, you're in.

---

<div align="center">
<sub>🍌 Juanita Banana — because your browsing data is none of their business.</sub>
</div>

