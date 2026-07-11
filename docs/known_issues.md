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
