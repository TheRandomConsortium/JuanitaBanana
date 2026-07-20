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
| Rust binary | [`arti`](https://gitlab.torproject.org/tpo/core/arti) — Tor Project's official Rust port |
| Clearnet routing | Optional: all traffic via Tor exit nodes |
| Status | ✅ Implemented (subprocess) |

Tor provides onion routing: traffic is encrypted in three layers and relayed through three nodes.
No single node knows both source and destination.

#### Current implementation — `arti` subprocess + local SOCKS5 proxy helper

> [!WARNING]
> **Deprecated interim architecture.** The current hop chain is a pragmatic workaround for limitations
> of the `arti` subprocess and WebKit's SOCKS5 delegation model. It will be fully replaced in Phase 4
> by the `arti-client` in-process architecture described below.

The current integration spawns `arti proxy --socks-port 9150` as a background subprocess. WebKit
cannot be pointed directly at `arti`'s SOCKS5 port for two reasons:

1. **IP-as-hostname failure**: WebKit delegates *all* hostname resolution to the SOCKS5 proxy. When
   a Handshake domain resolves to an IP, WebKit sends that raw IP string (e.g. `"103.152.197.116"`)
   to the proxy as a domain-type (`0x03`) lookup. `arti` cannot parse it and tries to resolve it over
   Tor's exit resolver — which returns `remote hostname lookup failure`.
2. **Handshake domains need local resolution**: `hnsd` runs locally; Handshake names must be resolved
   in-process, not forwarded to Tor's exit DNS.

The workaround is a **local SOCKS5 proxy helper** (port `9151`) that sits between WebKit and `arti`:

```
WebKit → local helper :9151 → (if .onion or tor_route_all) arti :9150 → Tor exit → destination
                             → (otherwise)                             clearnet → destination
```

The helper:
- Resolves Handshake domains locally via the in-process resolver chain (using `hnsd` on clearnet)
- Converts IP-string targets back to native IPv4/IPv6 before forwarding to `arti`, bypassing the lookup
- Passes `.onion` names directly to `arti` as domain strings (self-authenticating, no lookup needed)
- Proxies raw bytes bidirectionally once the upstream connection is established

**Known tradeoffs of this hop chain:**
- Three hops for every Handshake-over-Tor request: in-process proxy → `arti` → exit node
- Two persistent daemons (`hnsd` + `arti`) running at all times when Tor is active
- HNS subdomain nameserver queries (e.g. `nathan.woodburn` → authoritative NS) still go over
  **clearnet**, because `hnsd` is a C subprocess with no Tor circuit awareness
- The local proxy is a thread-per-connection model with synchronous blocking I/O — not designed
  for high concurrency (acceptable for a browser, not ideal)

#### Target final architecture — in-process `arti-client` (Phase 4)

The final architecture, deferred to the **Phase 4 / native Rust HNS + Tor integration** milestone,
collapes the entire hop chain into a single in-process path with zero external proxy daemons:

```
[Current]  WebKit → helper :9151 → arti :9150 → Tor exit → destination
[Phase 4]  WebKit → arti-client (in-process) → Tor exit → destination
```

- **No external proxy ports**: `arti-client` exposes a `TorClient::connect()` async stream API.
  We intercept WebKit's connection at the Rust network layer and pipe the stream directly — no
  local SOCKS5 helper, no `arti` subprocess, no port juggling.
- **On-demand circuit management**: circuits open only when a `.onion` URL is navigated to, or
  `tor_route_all` is active. No persistent daemon idles at startup.
- **HNS-over-Tor (fully native)**: once `hnsd` is ported to Rust (in-process SPV), its P2P UDP
  header sync and recursive nameserver queries can be forced through `arti-client` circuits natively.
  This is architecturally impossible with the C subprocess model.
- **Dependency note**: `arti-client` pulls in a significant async tree (tokio, rustls). Deferred
  because it would have been disproportionate overhead before the native Rust HNS milestone.

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
```
         ↑ Reorderable via juanita://config

### ⚡ Non-Blocking Resolution & Resilient Fallback

To ensure the browser remains responsive and secure under unstable connections, the resolver stack leverages the following architectures:

- **Asynchronous GUI Decoupling (✅ Implemented)**: Resolvers are executed asynchronously off the main GTK thread using non-blocking channels and `glib::timeout_add_local` ticks. This prevents network latency or dead Handshake daemon connections from freezing the browser GUI.
- **Resilient Non-Blocking Chain Retries (📋 Planned)**: Currently, if a resolver in the chain is slow or times out, it halts downstream evaluation. The planned improvement introduces parallel fallback execution: once a resolver fails its initial attempt, we immediately spawn background retries *without blocking the chain*, allowing downstream resolvers (like System DNS) to try resolving in parallel. If a downstream resolver resolves first, the browser proceeds, but cancels/discards background retries.
- **Navbar Resolver Override (📋 Planned)**: Directly inside the browser navbar, we will display an indicator showing which resolver successfully answered the current page lookup. This indicator doubles as a dropdown selector, enabling the user to perform an instant, one-off override to resolve the current domain using a specific alternative resolver on-the-fly.

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
all navigation traffic through Tor. Conversely, you can route navigation through Tor without
activating the `.onion` resolver (though that would be unusual).

> [!NOTE]
> **Handshake over Tor / I2P Resolution**: When routing navigation through Tor or I2P, resolving Handshake domains presents similar architectural options:
> - **Tor Resolution**:
>   1. **Direct Local hnsd Resolution** *(current implementation)*: Running `hnsd` directly as a clearnet process. Root TLD delegation lookups happen trustlessly and entirely **inside our node** by verifying SPV headers synced from the blockchain. However, Handshake subdomains (e.g. `nathan.woodburn`) are not stored on the blockchain; resolving them requires `hnsd`'s recursive resolver to externally query the authoritative nameservers over the network. Under our current architecture, these nameserver queries go over clearnet, but once the target IP is resolved, all HTTP/HTTPS web traffic is routed securely through Tor exit nodes.
>      * *Why DNS-over-Tor Forwarding failed:* Attempting to forward recursive queries to Tor's local DNS listener (`127.0.0.1:9053`) caused all subdomain lookups to be delegated to Tor. Standard Tor exit resolvers query standard ICANN DNS, which does not recognize Handshake TLDs, resulting in instant resolution failure (`SERVFAIL`).
>      * *Torsocks deprecation:* Wrapping the entire `hnsd` subprocess in `torsocks` is fully deprecated because `torsocks` blocks local UDP loopback socket creation (`Function not implemented` errors) and Tor exit nodes block P2P syncing on port `12013`.
>      * *SOCKS5 IP-as-Hostname Failure:* When Tor Route-All is active, WebKit is configured with a SOCKS5 proxy (`socks5://127.0.0.1:9150`) which delegates all hostname resolution to Tor. When the browser rewrites a Handshake domain to its resolved IP address (e.g. `http://103.152.197.116/`), WebKit sends the raw IP string `"103.152.197.116"` to the proxy as a domain string (SOCKS5 type `0x03`). `arti` fails to parse this string as a raw IP address and attempts to resolve it as a domain name over the Tor network, resulting in `remote hostname lookup failure` and returning an `Internal SOCKSv5 proxy server error` to WebKit.
>   2. **Tor-forced daemon resolution** *(target architecture)*: Forcing the resolver to query the P2P network completely through `arti-client` circuits natively. This is only achievable with a native Rust HNS port (Phase 4) where header sync and nameserver queries can be easily hidden under Tor natively with our own implementation — easy peasy!
> - **I2P Resolution**:
>   1. **Outproxying queries**: Wrapping Handshake P2P requests in garlic encryption and routing them through I2P outproxies to reach the clearnet P2P network.
>   2. **Native I2P tunnel resolution**: Querying Handshake P2P seed nodes that exist directly inside the I2P network (as `.i2p` destinations) via native I2P client tunnels, avoiding outproxy reliance entirely.
>
> ### 🧅 Hosting a Handshake Onion Node (Onion Peer)
> To completely hide blockchain header sync from your ISP and route P2P traffic through Tor without hitches from exit node port blocks, we can connect our client directly to Handshake Onion nodes (peers running as Tor Hidden Services).
> 
> Hosting an Onion Peer is trivial:
> 1. **Configure Tor Hidden Service**: On a server running a Handshake full node (`hsd`), add the following to `/etc/tor/torrc`:
>    ```text
>    HiddenServiceDir /var/lib/tor/hns-peer/
>    HiddenServicePort 12038 127.0.0.1:12038
>    ```
> 2. **Get your Onion Hostname**: Restart Tor to generate the private keys and hostname:
>    ```bash
>    sudo systemctl restart tor
>    sudo cat /var/lib/tor/hns-peer/hostname
>    # Yields: abcdef1234567890.onion
>    ```
> 3. **Run hsd in Onion Mode**: Start your `hsd` full node, binding its public host configuration to the `.onion` address:
>    ```bash
>    hsd --listen --bip37=true --public-host="abcdef1234567890.onion"
>    ```
> 4. **Connect hnsd to Onion Node**: Pass the onion peer directly to `hnsd` via the SOCKS5 proxy:
>    ```bash
>    hnsd -s <peer_identity_key>@abcdef1234567890.onion:12038
>    ```
>    Since Tor handles hidden service routing internally, connections to `.onion` targets never leave the Tor network. Exit node port policies are completely bypassed, providing secure and private P2P block header synchronization.


---

## Implementation Roadmap

### Phase 1 — Tor via `arti` subprocess (✅ Implemented, ⚠️ Interim Architecture)

> [!WARNING]
> The local SOCKS5 proxy helper (port `9151`) introduced in v1.6.8 is a **deprecated interim workaround**
> and will be removed entirely in Phase 4 when `arti-client` is embedded in-process.
> See the *"Current implementation"* section above for the full rationale.

- `arti proxy --socks-port 9150` daemon lifecycle (same pattern as `hnsd`)
- Toggle in `juanita://config` → Resolver Stack tab to enable/disable
- Optional `tor_route_all`: route all clearnet navigation through Tor exit nodes
- **Local SOCKS5 proxy helper** (port `9151`) sits between WebKit and `arti`:
  - Resolves Handshake domains in-process before forwarding
  - Translates IP-as-hostname SOCKS5 requests back to native IP connections
  - Routes `.onion` and `tor_route_all` traffic to `arti :9150`; clearnet traffic directly
- `OnionResolver` registered in resolver chain (returns sentinel IP `127.0.0.2`)
- `policy.rs` sentinel detection: keeps `.onion` URI intact, `decision.use_()` routes via proxy
- HNS subdomain nameserver queries (e.g. `nathan.woodburn`) still go over **clearnet** — hnsd
  is a C subprocess with no Tor circuit awareness. Full privacy requires Phase 4.
- `torsocks` wrapping and DNS-over-Tor forwarding (unbound.conf) fully removed (failed approaches,
  documented in the *"Handshake over Tor"* section above).

### Phase 2 — Handshake resolver (✅ Implemented)
- Subprocess bridge to `hnsd` C binary
- Integrated with resolver chain (configurable slot)
- Per-domain pinning UI in config
- Conflict visualisation: show which resolver answered

### Phase 3 — I2P integration (🔭 Future)
- Subprocess to Java I2P router as initial path
- Switch to `i2p-rs` when stable
- Register `.i2p` resolver
- Optional navigation routing via I2P outproxies

### Phase 4 — Native Rust HNS + in-process `arti-client` (🔭 Future)

This phase **eliminates all interim workarounds** introduced in Phases 1–2:

- **Remove local SOCKS5 proxy helper** (`src/tor/proxy.rs`) — no longer needed once WebKit
  connections are intercepted natively via `arti-client`
- **Remove `arti` subprocess** (`src/tor/mod.rs` daemon lifecycle) — replaced by `arti-client`
  in-process circuit management
- **Remove `hnsd` subprocess** — replaced by native Rust SPV implementation
- Port `hnsd` SPV logic to Rust (full in-house, no C FFI, no subprocess)
- Embed `arti-client` directly:
  - On-demand circuit management (no persistent daemon at idle)
  - Native in-process stream for `.onion` connections — no external SOCKS5 port
  - HNS block header sync and subdomain nameserver queries forced through `arti-client`
    circuits natively — **full HNS-over-Tor privacy for the first time**
- Net result: `WebKit → arti-client (in-process) → Tor exit → destination`
  (three fewer hops, two fewer daemons, zero interim workarounds)

### Phase 5 — Self-Healing DHT-based Tor-HNS Mesh (🔭 Future)

To build a completely decentralized, secure P2P validation network over Tor without relying on any centralized seeding/hosting infrastructure (since hosting is not in our DNA):

- **No `hns-bcoin` Port**: We will **not** be using the `hns-bcoin` codebase. While numerous professional companies have gotten rich using `bcoin` as the foundation for their operations, the codebase itself has been abandoned since forever. This is not a critique against Handshake, but a critique of the corporate world taking value without maintaining the open-source commons.
- **Dual-Boot SPV / Full Node Rust Implementation**: Our native Rust HNS port will support a dual-boot configuration:
  - **SPV Mode**: Verification using block headers only. Light client footprint.
  - **Full Node Mode**: Downloads full blocks and relays network packets to SPV nodes over the overlay.
  - *No Bloat*: Since we have no use for BID, OPEN, wallet, mining, or blockchain transaction drafting mechanics inside the browser context, we will not implement any of those features. The implementation will solely cover what is required to discover the network (preferably without touching the clearnet) and relay blocks/headers.
- **"Use SPV Only" Configuration**: Users can opt out of running a full relay node via `juanita://config`. Enabling this option will display the following disclaimer:
  > "I want to use spv only because I want to remain completely anonimous however I do understand if everyone runs spv only I will not be able to query via tor and will default to clearnet hurting the community and my own desires to remain anonimous"
- **Onion Directory Advertising:** Full-node instances dynamically spin up their own Tor Onion Hidden Services and automatically advertise their `.onion` addresses over Juanita's decentralized search DHT network.
- **Automated Peer Discovery:** SPV light client instances query the DHT to discover active full-node `.onion` addresses and use them as their peer seeds over Tor, creating a self-healing overlay mesh where peer discovery and header sync are fully private and automated.


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
[ Tor ]         [●] Enabled   [ ] Route all navigation through Tor
[ I2P ]         [○] Disabled  [ ] Route all navigation through I2P
[ Handshake ]   [○] Disabled

Resolver order (drag to reorder):
  1. ████████████  Handshake
  2. ████████████  I2P
  3. ████████████  Tor / Onion
  4. ████████████  System DNS

Domain pinning rules:
  +  [ domain or pattern ]  →  [ resolver ▾ ]
```
