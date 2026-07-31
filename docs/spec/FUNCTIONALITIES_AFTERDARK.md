# 🍌 Juanita Banana — Features Tracker (After Dark Edition)

***

## ⚖️ GIANT LEGAL DISCLAIMER & DECLARATION OF SEMI-CONSCIOUSNESS

### PLEASE READ THIS NOTICE IN ITS ENTIRETY BEFORE PROCEEDING OR JUDGING THE MENTAL SANITY OF THE DEVELOPMENT TEAM

**SECTION 1.0: PREAMBLE ON ALTERED STATES**  
This document (hereinafter referred to as "The After Dark Roadmap," "The Hallucinated Specifications," or "The 3:00 AM Git Push") outlines architectural designs, protocol integrations, and feature specifications that **DO NOT EXIST** in the stable, sober, or legally defensible branches of the Juanita Banana browser project. The features detailed herein represent concepts that may or may not be practically realizable, technically sane, or compliant with standard thermodynamics. 

**SECTION 1.1: CAUSE OF ACTION AND COGNITIVE DECLINE**  
The contents of this file were compiled, authored, committed, and pushed while the primary developers were operating under severe, non-standard physiological states. These states include, but are not limited to:
1.  **Extreme Sleep Deprivation:** Surpassing the 72-hour wakefulness threshold, wherein the terminal begins to display visual artifacts and compiler warnings start whispering sweet nothings.
2.  **Ultra-Marathon Cognitive Drift:** Authorship occurring during or immediately following excessive physical endurance exercises (e.g., running 100+ kilometers through rugged terrain), during which oxygen supply to the cerebral cortex was temporarily redirected to the quadriceps.
3.  **Chemical Hyper-Stimulation:** Consumption of highly concentrated bean-water (caffeine), taurine-infused carbonated solutions, and raw sugars in quantities that would alarm the World Health Organization.
4.  **Altered Spatial Awareness:** Typing while lying flat on the floor, upside down, or in dream-like states where Rust's borrow checker is perceived as a benevolent deity rather than a strict static analyzer.

**SECTION 1.2: INDEMNIFICATION OF SANITY**  
By reading beyond this paragraph, the reader (hereinafter "The Intrigued Bystander") agrees to hold harmless the authors, their families, and their pets from any liability arising from:
*   Attempting to compile code that requires a local Monero node and a 4-bit VLM running in a RAM disk.
*   Expecting a Soviet-era sound card emulator to run natively inside a WebKitGTK wrapper.
*   Explaining to their corporate board why their tracking cookies have been replaced by coordinates of an infinite spiral walk.

*We are not saying these features are impossible. We are saying that if they break your computer, summon a local demon, or cause your ISP to send you a cease-and-desist letter written in Latin, you were warned.*

***

## 🧬 Altered State & Speculative Features

### 🍌 The Banana Peel Router (BPR) & Resolver (BPR-R)

A fully decentralized, probabilistic, non-blocking onion-style overlay routing protocol and domain name resolution layer designed to operate purely inside the peer-to-peer swarm of active Juanita Banana instances.

```
       [ Client ] 
           │
           │ (Encrypts: Domain + MyPubKey with Random Key)
           ▼
     [ Random Node ] ──(Can't decrypt?)──► [ Random Node ] ──(Can't decrypt?)──► [ Random Node ]
                                                                                   │
                                                                           (Decrypts Succeeded!)
                                                                                   │
                                                                                   ▼
                                                                           [ Decrypting Node ]
                                                                           (Resolves IP!)
                                                                                   │
                                                                      (Encrypts IP with Client's Key)
                                                                                   │
                                                                                   ▼
     [ Random Node ] ◄──(Not Client?)─── [ Random Node ] ◄──(Not Client?)─── [ Random Node ]
           │
     (Match Client!)
           ▼
      [ Client ] ──► System Notification: "Someone slipped on your banana peel!" ──► Website opens
```

#### 1. Banana Peel Router (BPR) Core Routing Model
*   **Keys and DHT**: Every Juanita browser node generates a public/private key pair and advertises its presence to a globally distributed hash table (DHT), which is shared with other subsystems (such as the P2P Search Intoxication pool).
*   **The Slippery Packet (Query Encryption)**: To look up or route to a destination domain via BPR:
    1.  The client packages the query payload containing the `target_domain` and the client's own `public_key`.
    2.  The client selects a random public key from the DHT pool and encrypts the payload with it.
    3.  The encrypted packet is dispatched to a randomly selected node in the DHT.
*   **Probabilistic Packet Hopping**:
    *   **Decryption Failure**: When a node receives a BPR packet, it attempts to decrypt it using its private key. If decryption fails, the node immediately forwards the unaltered packet to a randomly chosen peer in the DHT.
    *   **Decryption Success**: If a node successfully decrypts the packet, it extracts the target domain and the requester's public key:
        *   **Successful Resolution**: The node attempts to resolve the domain. If it successfully resolves the target IP, it encrypts the IP mapping with the requester's public key and sends the response packet back into the random-walk network.
        *   **Failed Resolution**: If the resolving node cannot resolve the domain (e.g. timeout, DNS failure), it re-encrypts the original payload with a new random public key from the DHT and forwards it to another random node to let other peers try.
*   **Ultra-Latent Async Execution**: Because routing is a directionless random walk, resolution is expected to take hours, days, or even weeks. BPR runs entirely in the background without blocking the UI. When a response packet finally returns and is decrypted by the client, the browser fires a desktop notification:
    > *"Someone slipped on your banana peel!"*
    The destination is cached, and the website loads automatically.

#### 2. Banana Peel Router Resolver (BPR-R)
Triggered whenever navigating to a custom `juanita://` address that is not a registered internal page (e.g. `juanita://secretwiki`). It acts as a decentralized DNS and resource registry:
*   **Registration (Create Mode)**:
    - To host a resource under a custom name, the author packages the IP mapping and the name, encrypts the record, and launches it into the BPR.
    - The node in the network that successfully decrypts the packet stores the record. If it already has a record for that name, it appends a UUID to the record to handle the collision and sends a name resolution warning in the response. This node becomes the authoritative hosting custodian for that name.
*   **Resolution (Query Mode)**:
    - Clients seeking to browse a `juanita://` address send a cryptographically wrapped `dig`-equivalent request over the BPR.
    - The packet hops randomly through peers.
    - If a peer decrypts the request and holds the corresponding host mapping, it encrypts the resolution mapping with the client's public key and sends the answer back.
    - If the node decrypts the request but does not hold that name record, it re-encrypts the request with a random public key and forwards it to a random peer.

---

### 🧪 IASL — Isoamyl Acetate Scripting Language (`.bg` / `text/iasl`)

**"HTML, we hate you. Long live the Banana Gummy."**

IASL is Juanita Banana's native, anti-HTML domain-specific document and UI language (DSL). Named after **isoamyl acetate** (the organic ester responsible for artificial banana flavor and banana gummy scent), IASL eliminates HTML5 tag soup, DOM complexity, and CSS bloat by introducing a 100% banana-centric markup syntax and a native GTK/Cairo rendering pipeline that skips HTML transpilation entirely.

```
MIME Type: text/iasl
File Extension: .bg (Banana Gummy)
Status: 📋 Planned (Afterdark Feature Stack)
Official Language: Banana Peel Republic (BPR) & Internal Page Engine
Rendering Engine: Zero-HTML Native GTK/Cairo Layout Engine
```

#### 1. The Banana Lexicon (Banana-Centric Vocabulary)
IASL rejects generic web vocabulary (`<div>`, `<button>`, `class="card"`) in favor of organic banana anatomy and lifecycle terminology:

| IASL Keyword | Traditional Equivalent | Description |
|---|---|---|
| `peel` | `<html>` / `<main>` | Root document container (`peel yellow`). |
| `bunch` | `<section>` / `<div>` | Structural layout container / card block. |
| `stem` | `<h1>` / `<header>` | Document title or section header. |
| `pulp` | `<p>` / `<span>` | Main text content body. |
| `dark-spot` | `<button>` | Interactive action button (clickable spot on the peel). |
| `fiber` | `<input>` / `<textarea>` | Form text entry field / input line. |
| `bruise` | `<div class="alert">` | Warning / alert callout banner. |
| `slice` | `<hr>` | Horizontal divider / rule. |
| `ripeness` | `@theme` / color scheme | Theme setting (`green`, `yellow`, `spotted`, `overripe`). |

#### 2. Sample `.bg` Document (The Banana Gummy Markup)

```iasl
# Welcome to the Banana Peel Republic
@ripeness yellow
@stem "Juanita Banana Central Hub"

peel
  [bunch]
    stem: "Banana Peel Republic — Official Network Node"
    pulp: "HTML is deprecated. Long live the Isoamyl Acetate Scripting Language."

    [bruise]
      pulp: "Warning: Entering un-peeled overlay territory over Tor/I2P."
    [/bruise]

    fiber(id: "handle", type: "text"): "Enter BPR Node Handle"
    dark-spot(action: "juanita://config"): "Adjust Ripeness Settings"
    dark-spot(action: "juanita://downloads"): "Inspect Harvest Queue"
  [/bunch]
/peel
```

#### 3. Native Zero-HTML Engine Architecture
*   **Rust AST Parser (`src/browsing/iasl/parser.rs`)**: High-performance Lexer/Parser that compiles `.bg` source into a native IASL Abstract Syntax Tree (`IaslAst`).
*   **Direct GTK/Cairo Layout Renderer (`src/browsing/iasl/render.rs`)**: Bypasses WebKit HTML/CSS/JS rendering entirely! Converts the `IaslAst` directly into native GTK widgets (`gtk::Box`, `gtk::Button`, `gtk::Label`, `gtk::Entry`) or custom Cairo-painted surface primitives with Juanita Banana design tokens.
*   **Universal Protocol Handler**: Automatically intercepts any response with `Content-Type: text/iasl` or `.bg` extensions across **Tor (`.onion`)**, **I2P (`.i2p`)**, **Handshake (`.hns`)**, **clearnet**, or **`juanita://`** addresses.
*   **Internal Page Migration (MVP Goal)**: Recreate internal pages (e.g. `juanita://home` and `juanita://downloads`) in native IASL (`.bg`) to declare IASL as the official rendering language of the BPR.

#### 4. Legacy W3C Compatibility & Panama Disease Quarantine (`[panama-disease]`)

During initial IASL deployment phase (until 100% IASL feature completeness is achieved), legacy W3C web standards (HTML, CSS, JavaScript) are permitted as transitional fallbacks — but **only under strict containment in Panama Disease blocks**.

```iasl
[panama-disease:html]
  <div class="legacy-wrap">Legacy HTML Fallback Content</div>
[/panama-disease:html]

[panama-disease:js dep="legacy_logic"]
  console.log("Legacy JS execution attempt");
[/panama-disease:js]
```

##### Strict Containment Rules:
1.  **Single-Language Enclosures**: Each `[panama-disease]` block must contain **strictly one** legacy language (`:html`, `:css`, or `:js`). Inline `<script>` tags inside HTML blocks or inline `<style>` tags are strictly forbidden and trigger immediate parsing rejection (`PanamaContaminationError`).
2.  **Infection Dependency System**: If a `[panama-disease:html]` block references or depends on a `[panama-disease:js]` block (or external JS resource), the browser **refuses to execute or render the document automatically**.
3.  **Deferred Quarantine Warning Screen**: Contaminated documents with JS dependencies are halted and presented in a hazardous quarantine viewer:
    > 🦠 **PANAMA DISEASE INFECTION DETECTED!**  
    > *"This IASL document contains legacy W3C JavaScript dependencies that could infect and destroy your banana farm! Review the raw code below before approving execution."*
    *   **User Action Options**:
        *   `[ Risk Farm Infection & Render ]` → Executes the legacy script in an isolated sandbox.
        *   `[ Sanitize & Strip JS ]` → Purges all `[panama-disease:js]` blocks and renders the pure IASL + HTML structure safely.

---

### 🐧 Stallman Mode Easter Egg (`juanita://st-ignutius`)

**"Free Software, Free Mind, Carpal Tunnel Syndrome."**

Navigating to `juanita://st-ignutius` (or launching with the `--saint-ignutius` flag) forces the browser into **Stallman Mode** — a hyper-dogmatic, anti-proprietary, keyboard-only terminal presentation environment inspired by the teachings of Saint Ignutius.

```
Status: 📋 Planned (Afterdark Easter Egg)
Internal URI: juanita://st-ignutius
Motto: "There is no system but GNU, and Linux is one of its kernels."
```

#### 1. Visual & Rendering Constraints
*   **VT100 / TTY Monospace Rendering**: Overrides WebKitGTK/Cairo styles. All web pages are stripped of CSS and rendered in a green-on-black monospace font (`Courier New`, `Liberation Mono`, or `VT100`).
*   **ASCII Media Transpilation**: Images and `<video>` streams are converted in real-time to ASCII art matrices (`@#S%?*+;:,. `) rendered directly into the terminal view.
*   **Proprietary License Purge**: Any page content containing DRM, `Copyright ©`, `All Rights Reserved`, `DMCA`, or proprietary EULAs is redacted and replaced with:
    > `[PROPRIETARY SLOP PURGED BY SAINT IGNUTIUS]`

#### 2. Ergonomic Punishment & Keyboard-Only Control
*   **Zero-Mouse Policy**: Mouse events, pointer tracking, clicks, context menus, and scroll wheels are **100% disabled**. The cursor is hidden, and any attempt to move or click the mouse triggers a system bell and displays:
    > `ERR: MOUSE IS A PROPRIETARY POINTER DEVICE. USE EMACS KEYBINDINGS.`
*   **Carpal Tunnel Keybindings**: All browser navigation and UI interactions require multi-modifier Emacs / GNU Info key combinations designed to maximize finger strain:
    *   `C-x C-f` → Focus URL address bar
    *   `C-x C-s` → Save web page
    *   `C-u 42 C-v` → Scroll down 42 lines
    *   `C-M-s` → Incremental search
    *   `C-x k` → Kill active tab
    *   `C-x C-c` → Close browser
*   **CLI File Management**: GUI file pickers are banned. All file persistence actions must be typed via mocked terminal CLI commands (e.g. `mv <sandboxed_file> /target/location --paranoid`).



