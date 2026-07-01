# Contributing to Juanita Banana 🍌

So you read the manifesto and you want to fight back. Welcome to the resistance.

Before you submit a Pull Request, understand one thing: **This is not a corporate project. We do not care about advertisers, we do not care about metrics, and we do not care about "playing nice" with the modern web.** 

Our only metric is how much noise we can inject into the surveillance economy.

## 📜 The Golden Rules

1. **Anti-Fingerprinting is Offensive, Not Defensive.** 
   We don't block requests. We poison them. If you add a feature to block a tracker, your PR will be rejected. If you add a feature to feed that tracker mathematically perfect garbage data, your PR will be merged.
   
2. **No Google Engines.** 
   Do not even suggest migrating to Blink/Chromium. WebKitGTK is our pragmatic engine for now. Servo is our end-goal.

3. **Spartan Code, Spartan UI.** 
   No bloatware. No Electron. Native Rust, native GTK. If your feature requires an embedded Node.js server, take it to Google Chrome.

4. **The Corporate Clause applies to your code.**
   If your code makes the browser look weird to a fingerprinting script, it's working. We want their models to collapse.

## 🛠️ How to Contribute

1. **Pick a Target:** Check `docs/FUNCTIONALITIES.md`. Anything marked as 🔭 *Future* or 📋 *Planned* is fair game. (e.g., Audio Spoofing, Background Captcha Solver, P2P Gossip Protocol).
2. **Branching Strategy:** 
   - `main`: Stable, compiling, usable browser.
   - `chaos`: The bleeding edge. Point your PRs here if you are experimenting with new tracking poison.
3. **Keep it Rusty:** Run `cargo fmt` and `cargo clippy` before you push. We write chaotic software, not spaghetti code.
4. **Document your poison:** If you introduce a new spoofing mechanic, document exactly *what* tracker it targets and *how* it contaminates the data.
5. **Testing & QA:** We have unit tests for the pure Rust business logic (`src/config.rs`, `src/noise.rs`, `src/spoof.rs`, etc.). Run `cargo test` before submitting. **Note:** GTK/WebKit-specific UI tests (`src/gui.rs`, `src/intoxication.rs`) are currently missing due to the complexity of headless X11/Wayland testing. If you are an expert in GTK headless testing, we highly welcome contributions in this area!

## 🧠 Brainstorming & Communication

If you are coming from `/g/`, `/sec/`, or anywhere else:
- Post your ideas, patches, or forks.
- We don't have a Discord. Discord is spyware. Use the GitHub Issues or post your patches on the thread.

> *"This is not a bug. This is a feature."*
