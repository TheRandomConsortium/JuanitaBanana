# Known Issues & Security Stance

Here lies the compendium of known issues where security and convenience clash, and where we refuse to compromise.

## Proton Mail Authentication Failure

### The Issue
Proton Mail refuses to allow login and raises a generic authentication error under Juanita Banana.

### The Cause
According to Proton Mail's official documentation, their login mechanism requires specific request headers to proceed. Specifically:
> "Disable Authorization headers" must be unchecked to use Proton Mail.

Juanita Banana's browser engine enforces strict tracking-prevention policies and strips/sanitizes request headers to prevent user fingerprinting and cross-site authorization leaks. 

### Our Stance
**We will not unharden our stack to appease Proton Mail.** 

Asking users to voluntarily weaken browser security, disable header controls, or permit leaking authorization metadata is unacceptable friction. A password manager and secure browser should never compromise its host's defense posture to support single-provider compatibility.

### Resolution
If you must use Proton Mail:
1. **Use the native Proton Mail applications** instead of accessing their services via the browser.
2. Accept this as a design choice in our quest for a hardened ecosystem.

### Future/Alternative Workaround: Per-Domain Exceptions (Planned/Doubtful)
A potential compromise under consideration is introducing an **Exceptions Panel** in `juanita://config`, allowing users to configure custom profiles or selectively deactivate specific spoofing layers/HTTP header protections on a per-domain basis.

To ensure the user is fully aware of the compromised state they are choosing, the configuration page will show a "This is fine" dog meme. The visual representation will dynamically escalate to a more distressing, fiery scene the more security techniques and spoofing layers the user disables.

---

## Google Pay & Payment Gateway Failures

### The Issue
Google Pay screens and specific proprietary payment gateways fail to load or process transactions within the browser.

### Our Stance
While the exact root cause is currently unconfirmed (highly suspected to be Play Integrity checks and missing telemetry hooks), our position is clear: **we will not disguise Juanita Banana as Chrome** to bypass these checks if doing so forces us to weaken our privacy perimeter or leak environmental data. We are exploring potential emulation or data-safe tunneling solutions, but only under the strict condition that zero user data is leaked. 

### Resolution
If safe tunneling is not viable, these pages will remain broken by design. We advise users:
1. **Do not use these payment pages** in the browser, as their strict integrity requirements act as telemetry spyware.
2. Use alternative, less invasive payment methods.
3. Complete the transaction on a native smartphone app (since your mobile OS is already compromised by design, there is no additional privacy loss).

---

## Cloud-Assisted Passkeys (CaBLE) 

### The Issue
Using a smartphone as a passkey via the CaBLE (Cloud-assisted Bluetooth Low Energy) protocol is unsupported and fails by default.

### The Cause
The CaBLE protocol routes authentication handshakes through Google or Apple's cloud infrastructure to verify device integrity via their respective Play/Mobile Services.

### Our Stance
**We will never support CaBLE.** Even if technically feasible within WebKitGTK, we fundamentally oppose protocols that exfiltrate user authentication data and device metadata to first-party clouds for "integrity verification". We will not facilitate double-spying.

### Resolution
Use physical **Hardware Security Keys** (e.g., YubiKey, Nitrokey, SoloKeys). They operate securely via local USB/NFC hardware polling without relaying authentication payloads through corporate cloud proxies.

---

## YouTube Search & Recommendation Degradation

### The Issue
YouTube loads and plays individual video URLs directly without issue, but in-page video search, recommendations, and feed interactions fail to load or get blocked. Cloned/frontend YouTube interfaces (Invidious, Piped) work perfectly with search, recommendations, and playback.

### The Cause
Undetermined / Under active investigation. However, because video streaming payloads load cleanly when given a direct URL, this behavior strongly resembles voluntary, client-side degradation by Google against browsers that actively enforce anti-fingerprinting controls and refuse to leak user tracking telemetry.

### Our Stance
**Most likely Google being angry because they cannot spy on your watch history.** 

PeerTube works natively. Privacy frontends and clones work natively. Do we truly need to work overtime to solve Google's confidence issues? If the only ultimate fix ends up requiring unhardening the browser's privacy controls, do you really want to lower your pants to the monopoly?

### Resolution & Workarounds
Until we determine if a zero-compromise fix exists, we recommend the following privacy-preserving alternatives:

1. **Search Elsewhere & Jump In**: Perform video searches in DuckDuckGo or Google Search and click directly into the target YouTube video URL.
2. **Use PeerTube**: Migrate to decentralized video platforms like PeerTube that respect user sovereignty.
3. **Use Dedicated Privacy Apps / Frontends**: Use dedicated apps or privacy frontends (Invidious, Piped) so you only leak data through an isolated sandbox.
4. **RSS Feeds & Direct `youtubei` API Searching**: Export your YouTube subscriptions to standard RSS feeds. Execute lightweight video searches directly against YouTube's `youtubei/v1/search` endpoint:
   ```bash
   curl 'https://www.youtube.com/youtubei/v1/search?prettyPrint=false' \
     -X POST \
     -H 'Content-Type: application/json' \
     -H 'User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36' \
     -H 'Accept: application/json' \
     -H 'Accept-Language: en-US,en;q=0.5' \
     --data-raw '{"context":{"client":{"clientName":"WEB","clientVersion":"2.20240110.01.00","hl":"en","gl":"US"}},"query":"opensource"}'
   ```
5. **Future Zero-Backend Streamer (`omnistreamer.randºm`)**: Rather than bloating the browser core with internal protocol handlers, The Random Consortium is evaluating **`omnistreamer.randºm`**—a zero-backend, minimal-code frontend wrapper that parses user-supplied RSS subscription feeds at launch, executes direct cURL/HTTP search queries (e.g. `youtubei` API & PeerTube REST), and renders video embeds directly (or via stream fallback borrowing) with zero accounts, zero data saving/tracking, and minimal code.

