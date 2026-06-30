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
- **WebGL:** Vendor/renderer strings replaced with Juanita Banana GPU
- **Navigator:** hardwareConcurrency, deviceMemory, platform — all spoofed
- **User-Agent:** Honest by default (`JuanitaBanana/0.1`). In the future, a config toggle will allow switching to a rotating mode that cycles daily through a curated list of genuine, modern User-Agents to blend into the crowd perfectly.

### 🔍 Search Profile Obfuscation
- Every real search fires 20 background searches from a heterogeneous pool
- Future: P2P gossip protocol — your fake searches are real searches from other users

### 📣 Ad Profile Obfuscation
- Ads are hidden from view but interacted with in background
- If you click on everything, you are nobody

### 🚫 The Ban System
- One click BAN from the navbar
- Banned sites load a local HTML page: *"Go look for greener pastures elsewhere."*
- To unban: solve a math equation. No easy toggles.

### 🤖 AI Slop Detection
- Detects AI-generated content disclosures → injects warning in the article title

### 🔮 Future Arsenal
GTM poisoning · Meta Pixel contamination · Cookie garbage fill · Beacon API interception · Tor integration · Password manager · P2P search pool · GDPR Aggressive Unsubscribe

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

## Contributing

Read the manifesto. If you agree, you're in.

---

<div align="center">
<sub>🍌 Juanita Banana — because your browsing data is none of their business.</sub>
</div>

