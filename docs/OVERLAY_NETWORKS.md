# Overlay Networks & Decentralised Resolver Stack

Juanita Banana treats privacy networking the same way it treats fingerprinting: **actively, not defensively**.
Supporting Tor was never enough. The full plan is a composable overlay transport layer paired with a
priority-ordered, multi-root resolver stack — think of it as a BIOS boot order for DNS.

---

## Philosophy

> You should not *need* to use an overlay network. But if you *choose* to, everything should work.
> Protocols you activate unlock their corresponding resolver and address space automatically.
> Nothing is mandatory. Everything composes.

---

## Overlay Transports

Each transport is an independently togglable module. Activating one:
1. Routes the relevant address space through that transport.
2. Registers the corresponding resolver in the resolver stack.
3. Optionally routes **all** clearnet traffic through it (user-configurable per transport).

### 🧅 Tor — Onion Routing

| Property | Value |
|---|---|
| Address space | `.onion` (v3 hidden services) |
| Rust implementation | [`arti`](https://gitlab.torproject.org/tpo/core/arti) — Tor Project's official Rust port |
| Clearnet routing | Optional: all traffic via Tor exit nodes |
| Status | 📋 Planned |

Tor provides onion routing: traffic is encrypted in three layers and relayed through three nodes.
No single node knows both source and destination. `arti` is the official Tor Project Rust implementation
and is the target integration — no C dependency, no subprocess, in-process async circuit management.

### 🧄 I2P — Garlic Routing

| Property | Value |
|---|---|
| Address space | `.i2p` (Eepsites) |
| Rust implementation | [`i2p-rs`](https://github.com/i2p/i2p-rs) (in progress) / subprocess fallback to Java I2P router |
| Clearnet routing | Optional: via I2P outproxies |
| Status | 🔭 Future |

I2P uses **garlic routing**: multiple messages are bundled together ("cloved") into a single encrypted
payload, making traffic analysis harder than Tor's onion model. I2P is also a true darknet —
it is designed for internal `.i2p` services, not primarily for clearnet proxying.

The Rust ecosystem for I2P is less mature than Tor. Initial integration may use a subprocess call
to the Java I2P router, switching to a native Rust implementation once `i2p-rs` reaches stability.

### 🧅🧄 Transport Stacking (Garlic over Onion / I2P over Tor)

When both Tor and I2P are active, the user can optionally configure **transport stacking**:
your I2P client connects to the I2P network *through* Tor circuits rather than directly.
I2P nodes then see a Tor exit as your entry point — not your real IP.

This is a real, documented technique ("I2P over Tor") used when the threat model includes
an adversary who can observe I2P traffic patterns at the network edge but not Tor traffic.

```
Without stacking:   YOU ──(I2P garlic)──► I2P node ──► .i2p destination
With stacking:      YOU ──(Tor onion)──► Tor exit ──(I2P garlic)──► I2P node ──► .i2p destination
```

| Property | Tor alone | I2P alone | I2P over Tor |
|---|---|---|---|
| ISP sees I2P traffic | — | Yes | No (sees Tor) |
| I2P nodes see your IP | — | Yes | No (see Tor exit) |
| Latency | Medium | Medium | High |
| Anonymity set | Tor's | I2P's | **Intersection** (smaller, not larger) |

> **The intersection caveat.** Stacking does not simply multiply privacy. Your anonymity set
> shrinks to the population that is *both* a Tor user *and* an I2P user. This is useful for a
> specific threat model, not a universal improvement. The config should surface this warning.

The reverse — **Tor over I2P** (onion through garlic) — is technically possible but has
even more exotic tradeoffs and is not in scope for the initial implementation.

**Config exposure:** When both transports are enabled, expose a per-transport option:
`I2P entry via: [ Direct | Tor ]`. Default: Direct.

---

## Resolver Stack

The resolver stack operates as a **priority-ordered chain**: the first resolver that returns an
authoritative answer wins and the chain stops. If no resolver answers, it falls through to the next.

This directly solves the Handshake/ICANN namespace collision problem: whoever is first in your
chain owns the ambiguous TLD (e.g. if someone registered `google.com` on Handshake and you have
HNS at priority 1, you get the HNS result — put ICANN first to preserve legacy behaviour).

```
┌─────────────────────────────────────────────────────────┐
│                    RESOLVER CHAIN                        │
│  (User-configurable order — first authoritative wins)    │
├──────────┬──────────────────────────────────────────────┤
│ Slot 1   │  Handshake (HNS)   — decentralised TLD root  │
│ Slot 2   │  I2P               — .i2p address space       │
│ Slot 3   │  Tor / Onion       — .onion address space     │
│ Slot 4   │  System DNS (ICANN)— legacy fallback          │
└──────────┴──────────────────────────────────────────────┘
         ↑ Reorderable via juanita://config
```

### 🤝 Handshake (HNS) — Decentralised Root DNS

| Property | Value |
|---|---|
| Address space | Any TLD (HNS is a parallel root, not a suffix) |
| C implementation | [`hnsd`](https://github.com/handshake-org/hnsd) — lightweight SPV resolver |
| Rust port | Long-term goal: port `hnsd` logic to Rust for in-house maintenance |
| Status | 🔭 Future |

Handshake is a permissionless blockchain-based naming system that replaces the ICANN root.
Anyone can own any TLD (including `.com`, `.org`, etc.) on the HNS blockchain. Resolving
HNS names requires running a lightweight SPV client against the HNS P2P network.

The initial integration will link against or subprocess `hnsd` (C). The long-term goal is
a native Rust port to eliminate the C dependency and bring the resolver fully in-house.

**The namespace collision problem** — if ICANN and HNS both have an answer for `google.com`,
the user's resolver order decides. This is intentional, not a bug. It is the same principle
as `/etc/hosts` overriding DNS: whoever is first in your chain is authoritative for you.

### Per-Domain Pinning Rules

Beyond global resolver order, the user can define **pinning rules** in `juanita://config`:

```
example.bit    → always use: Handshake
mysite.i2p     → always use: I2P
darkservice.onion → always use: Tor
*.example.com  → always use: ICANN
```

Rules are evaluated before the chain. A pinned domain skips the chain entirely and goes
directly to its assigned resolver. This lets the user handle known-ambiguous domains
explicitly without reordering the global chain.

---

## Resolver vs. Transport

These are two separate concepts:

| Concept | What it does | Required for |
|---|---|---|
| **Resolver** | Translates a name into an address | Knowing where to connect |
| **Transport** | Routes your traffic through the overlay | Actually connecting |

You can have the Tor resolver active (to resolve `.onion` addresses) without routing
all clearnet traffic through Tor. Conversely, you can route clearnet through Tor without
activating the `.onion` resolver (though that would be unusual).

---

## Implementation Roadmap

### Phase 1 — Tor via `arti` (📋 Planned)
- Add `arti` as a Cargo dependency
- Toggle in `juanita://config` to enable/disable
- When enabled: register `.onion` resolver, optional clearnet routing
- SOCKS5 compatibility layer for WebKitGTK network stack

### Phase 2 — Handshake resolver (🔭 Future)
- FFI or subprocess bridge to `hnsd`
- Integrate with resolver chain (configurable slot)
- Per-domain pinning UI in config
- Conflict visualisation: show which resolver answered

### Phase 3 — I2P integration (🔭 Future)
- Subprocess to Java I2P router as initial path
- Switch to `i2p-rs` when stable
- Register `.i2p` resolver
- Optional clearnet routing via I2P outproxies

### Phase 4 — Native Rust HNS (🔭 Future)
- Port `hnsd` SPV logic to Rust
- Full in-house maintenance, no C FFI

---

## Other Protocols (Niche / Under Evaluation)

These exist and could theoretically be integrated. They are more niche and their value-to-complexity
ratio needs evaluation before committing to support.

| Protocol | Address space | Notes |
|---|---|---|
| **Lokinet** | `.loki` | Session messenger's onion routing; C++ codebase (`llarp`) |
| **Namecoin** | `.bit` | The original blockchain DNS; largely superseded by HNS |
| **ENS / IPFS** | `.eth` | Ethereum Name Service; requires Ethereum node or trusted gateway |
| **Yggdrasil** | IPv6 mesh | Encrypted mesh overlay, no special TLD; interesting for LAN mesh routing |
| **GNUnet** | `.gnu` | GNU's fully decentralised network; very niche, academic |
| **ZeroNet** | N/A | BitTorrent + Bitcoin blockchain; browser-based, less relevant for native |

If you know of another protocol that belongs here, open an issue.

---

## Configuration Surface (`juanita://config` → Network tab)

```
[ Tor ]         [●] Enabled   [ ] Route all clearnet through Tor
[ I2P ]         [○] Disabled  [ ] Route all clearnet through I2P
[ Handshake ]   [○] Disabled

Resolver order (drag to reorder):
  1. ████████████  Handshake
  2. ████████████  I2P
  3. ████████████  Tor / Onion
  4. ████████████  System DNS

Domain pinning rules:
  +  [ domain or pattern ]  →  [ resolver ▾ ]
```
