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
Use physical **Hardware Security Keys** (e.g., YubiKey, Nitrokey, SoloKeys). They operate secure
