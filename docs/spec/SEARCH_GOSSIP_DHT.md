# 🔍 P2P Search Gossip Protocol & Anonymized DHT Sharing

## ⚖️ Architectural Philosophy

The **P2P Search Gossip Protocol** replaces static or centralized ("Dumb Pipe") search noise providers with a 100% peer-to-peer, End-to-End Encrypted (E2EE) search exchange network between active Juanita Banana instances.

Rather than generating purely synthetic dictionary noise or relying on centralized servers, Juanita Banana instances pool anonymized, real human search queries across online peers. By mixing P2P peer searches, RSS n-gram terms, and local queries, every search engine query profile becomes a statistically unprofileable, highly chaotic signal.

```
                  ┌──────────────────────────────┐
                  │   Local Real User Search     │
                  └──────────────┬───────────────┘
                                 │
                   (E2EE Direct 1-Hop Broadcast)
                                 │
          ┌──────────────────────┼──────────────────────┐
          ▼                      ▼                      ▼
  ┌───────────────┐      ┌───────────────┐      ┌───────────────┐
  │ Online Peer A │      │ Online Peer B │      │ Online Peer C │
  └───────────────┘      └───────────────┘      └───────────────┘
```

---

## 🔒 Protocol & Cryptographic Architecture

### 1. Node Identity & Cryptographic Handshake
- **Curve25519 X25519 & ChaCha20-Poly1305 AEAD**: Every Juanita Banana instance generates a random Curve25519 static secret key pair (`x25519-dalek`). Session keys are derived via Diffie-Hellman (`local_secret.diffie_hellman(&peer_public)`), encrypting payloads with ChaCha20-Poly1305 AEAD and a 12-byte random nonce.
- **Wire Magic Header (`JBP1`)**: All P2P datagrams (handshake payloads and E2EE search payloads) begin with the 4-byte wire magic header `b"JBP1"`. Non-matching datagrams are discarded immediately.
- **RAM-Only Node Secret Storage**: Node secret key pairs are stored encrypted inside the user's secure database (`SecureDbManager` / `userdata.enc`). On startup or via `juanita://config`, the master password unlocks the secret strictly into RAM (`GLOBAL_NODE_KEY`) for the duration of the session. Plaintext key files on disk do not exist.
- **Swarm Phonebook & UPnP**: Active peers are persisted in `phonebook.bin` (active if seen in last 24h). Automatic UDP port mapping is attempted on IGD/UPnP routers via the `igd` crate.

### 2. Direct 1-Hop E2EE Broadcast (No Re-broadcasting)
- **E2EE Payload Packaging**: When a local search occurs (or background noise is dispatched), query payloads (`GossipQueryPayload`) are encrypted individually for each active online peer using their unique session key.
- **Strict 1-Hop Limit**: Nodes **never re-broadcast** received queries to third-party peers. Search queries exist only as direct 1-hop exchanges.
- **Live Online Exchange Only**: Search queries are delivered exclusively to currently online peers. No offline message queues or persistent relay servers exist.

### 3. Peer Banning Firewall & Isolation
- **Strict Blacklisting**: Nodes track a local blacklist of banned contributor Node IDs stored separately from web domain bans (`BanList.banned_peers`).
- **Zero Ingress / Egress**: 
  - **No Ingress**: Queries received from banned Node IDs are dropped immediately before decryption.
  - **No Egress**: Local search broadcasts are never transmitted to banned Node IDs.

### 4. Doubtful Functionality: Unbanning Peers
> [!NOTE]
> **Doubtful Functionality:** Unbanning a peer is considered a doubtful / low-value feature. Peers are banned exclusively when they submit unacceptable, toxic, or spam search queries into the swarm. Consequently, unbanning a peer is unlikely to produce any positive result — it merely re-exposes your node to their undesirable search traffic.

---

## ⚙️ Configuration Interface (`juanita://config`)

The P2P Search Gossip Protocol exposes the following granular controls under `juanita://config`:

| Configuration Key | Field Type | Default | User-Facing Description & Warnings |
|---|---|---|---|
| `allow_dht_search_sharing` | Toggle Checkbox | `false` | **Allow P2P DHT Search Sharing (No Leechers Allowed)**<br>*"Enables bi-directional P2P search sharing across online Juanita instances. Once enabled, your node both sends outbound search queries to online peers and receives anonymized queries from peers. All payloads are End-to-End Encrypted (E2EE) and anonymized. Note: No leeching is permitted — participating in the swarm requires two-way exchange."* |
| `rss_search_weight_percent` | Weight Slider (0–100%) | `50%` | **RSS vs P2P Search Noise Ratio**<br>*"Controls the mix ratio of synthetic RSS n-gram terms versus real P2P peer searches injected into background intoxication queries. Adjust this if you want your search profile to maintain recency and organic human traffic patterns."* |
| `contribute_own_searches` | Toggle Checkbox | `false` | **Add Own Searches to Intoxication Pool**<br>*"Includes your own local real search queries in your local intoxication pool for future background noise generation. Warning: If your search volume is small, recycling unique personal searches could assist adversarial correlation profiling. In a larger activity stream, however, it significantly increases profile entropy and confusion."* |
| `search_terms_ttl_days` | Number Input | `30` (Days) | **Search Terms Expiration TTL**<br>*"Expiration window (in days) for stored pool search terms. Stale terms are automatically purged after this period to maintain temporal recency."* |
| `prohibited_keywords_regex` | Text Input (Regex) | `""` (None) | **Prohibited Keywords Regex Filter**<br>*"Regex pattern to filter out sensitive keywords from ever being broadcast or ingested into your local pool. Warning: Filtering specific terms creates a structural bias in your noise distribution, slightly reducing overall anonymity."* |

---

## 💎 The Crown Jewel: Search Term Explorer (`juanita://search-explorer`)

The **Search Term Explorer** is an interactive internal management interface accessible via `juanita://search-explorer` or as a dedicated tab in `juanita://config`.

### 1. Features & Data Columns
The Explorer renders a live, filterable table of all search terms currently stored in the node's local intoxication pool:

| Column | Description |
|---|---|
| **Search Term** | The exact query string available for background intoxication. |
| **Origin Source** | Source category: `Local RSS`, `Local Search`, or `P2P Peer: <Node_ID>`. |
| **Ingested Date** | Timestamp when the term entered the local pool. |
| **Expiration Date** | Calculated TTL expiration date after which the term is purged. |

### 2. Live Filtering & Sorting Controls
To make the pool table fully manageable even with thousands of active search terms, the Explorer provides real-time client-side controls:
* **Filter by Search Term Regex**: Input field allowing real-time regex pattern or string matching on search term query strings (e.g., `^crypto.*`, `privacy`).
* **Filter by Node ID / Origin**: Input field / dropdown to filter terms originating from a specific P2P peer `Node_ID` or source category (`Local RSS`, `Local Search`).
* **Multi-Column Sorting Controls**:
  * **Order by Date**: Sort ascending/descending by Ingestion Date or Expiration Date to audit newly arrived or expiring queries.
  * **Order by Search Term**: Alphabetical ascending/descending sorting on query strings.
  * **Order by Node ID / Origin**: Group and sort search terms by contributor node identity.

### 3. User Actions
- **Delete Individual Term**: One-click removal of any specific search term from the pool.
- **Ban Contributor Node (`Outright Ban`)**: One-click action to ban a P2P contributor Node ID (e.g., if a peer submits spam, abusive content, or "dumb-dumb" queries). Banning a contributor:
  1. Instantly purges all search terms originating from that Node ID from the local pool.
  2. Adds the Node ID to the node's permanent P2P Blacklist (`BanList.banned_peers`).
  3. Evicts the peer from `SwarmPhonebook` and deletes their public key material.
  4. Immediately severs all E2EE gossip connections with that peer.
