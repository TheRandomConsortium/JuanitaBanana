# Inverse Advertising Framework (Ad Profile Obfuscation) 📣

Juanita Banana implements an **Inverse Advertising Framework** designed to neutralize tracking profiles by polluting the interest-telemetry collected by ad exchanges, data brokers, and advertisers. 

---

## ⚖️ Ethics & Intent (The "Model Citizen" Stance)

We are **not** committing fraud, nor are we attempting to cause direct financial harm to advertisers or publishers. We understand that advertising is a foundational funding mechanism for the open web, and that relevant, targeted ads can be useful.

Our fight is **not** against the advertising industry itself, but against:
1. **Pervasive Surveillance:** The silent, non-consensual building of behavioral profiles.
2. **Broken Opt-Out Policies:** Standard "opt-out" links, DNT headers, and consent banners that are either routinely ignored or act as additional fingerprinting markers.
3. **Dark Patterns:** Websites blocking content unless users agree to invasive tracking scripts.
4. **Uncontrolled Data Sharing:** First-party sites sharing behavioral events with third-party exchanges, who then resell them to fourth-party brokers.

By poisoning the click-through data with randomized, high-entropy interactions, we render the tracking profiles statistically useless. If a user appears to be interested in *everything*, they are effectively profiled as *nobody*.

---

## 🛠️ Technical Architecture

The framework consists of four core components:

```
  ┌──────────────────────────────────────────────────────────┐
  │                   1. Intent Interception                 │
  │  (Hooking DOM setters, window.fetch, and XMLHttpRequest) │
  └────────────────────────────┬─────────────────────────────┘
                               │ (Match found)
                               ▼
  ┌──────────────────────────────────────────────────────────┐
  │                    2. DOM Surgery                        │
  │  (Locates elements in DOM & removes them permanently)   │
  └────────────────────────────┬─────────────────────────────┘
                               │ (Report selector to Rust)
                               ▼
  ┌──────────────────────────────────────────────────────────┐
  │                 3. Background Queue                      │
  │  (1-by-1 sequential processing with random click delay) │
  └────────────────────────────┬─────────────────────────────┘
                               │ (Dice roll = YES)
                               ▼
  ┌──────────────────────────────────────────────────────────┐
  │             4. Headless WebKit Clone                     │
  │  (Loads page, runs Ghost Mouse hover & click sequence)   │
  └──────────────────────────────────────────────────────────┘
```

### 1. Origin & Intent-Based Interception
Unlike simple element-blocking (which relies on CSS selectors or element classes that change frequently), detection is performed at the **origin/intent level** (similar to uBlock Origin). 
* **Dynamic Property Patching:** The browser patches `HTMLImageElement.prototype.src`, `HTMLIFrameElement.prototype.src`, and `HTMLScriptElement.prototype.src` property descriptors. Setters are intercepted to catch dynamic ad assignments before they execute.
* **Element Attribute Monitoring:** The browser intercepts calls to `Element.prototype.setAttribute` for `src` and `href` values.
* **XHR and Fetch Monkeypatching:** The browser overrides `window.fetch` and `XMLHttpRequest.prototype.send`/`open` to capture background asynchronous tracking endpoints, even if no static DOM element exists.
* **Beacon API Interception (Future Work):** Override/interception of `navigator.sendBeacon()` is planned for future versions to capture and replace/poison background telemetry payloads before they leave the browser.
* **DOM Fallback:** A mutation observer monitors late-injected elements.

### 2. DOM Surgery (Removal)
Instead of hiding ads using CSS (`display: none;`), which still allows hidden scripts to execute and report presence:
* Once an ad resource/origin is identified, the browser performs **DOM Surgery**.
* The corresponding element (or its containing iframe/wrapper) is permanently deleted from the page using `element.remove()`.
* The user's screen remains clean, and the ad is never displayed.

### 3. Background Queue & Dice Roll
Every identified ad is sent to a background processing queue.
* **1-by-1 Sequential Processing:** To emulate natural human interaction, the queue is processed strictly **sequentially (one-by-one)**. Humans do not click multiple ads simultaneously; parallel clicked transactions would flag the profile as a bot.
* **No Click Duplication:** Ads are tracked by their target URL and identifier; each ad is processed at most once per page load to prevent click spamming and avoid economic damage.
* **The Dice Roll:** To prevent simple click-through rate (CTR) anomaly detection, the browser rolls a dice (e.g. 15% probability) to decide whether to click the ad. 
* **Jitter:** A randomized delay (jitter) is introduced between queue items, ensuring that requests do not appear as automated bursts.

### 4. Headless WebKit Clone & Ghost Mouse Simulation
If the dice dictates a click:
* The browser instantiates a hidden, headless WebKit `WebView` using the exact same settings, cookie context, and anti-fingerprinting rules as the main browser.
* The headless WebView loads the original page.
* A **Ghost Mouse** script is injected:
  1. It locates the ad element.
  2. It scrolls the element smoothly into view.
  3. It simulates realistic mouse navigation by dispatching `mouseover` and `mouseenter` events.
  4. It hovers for a random interval (e.g., 800ms to 2000ms).
  5. It dispatches `mousedown`, `mouseup`, and `click` events.
* The headless session remains active, following all redirect chains until a final, non-ad destination page is reached. Once the landing page loads, the session is safely destroyed.

---

## 🖱️ User-Driven Learning & Target Verification

Users can report ads manually by right-clicking on elements (links, images, etc.) and selecting **"Mark as Ad (Learn & Remove)"**.

### Candidate Verification (Preventing False Positives)
Because right-clicked elements can contain multiple URL attributes (e.g. a link tag pointing to a legitimate page like `reddit.com` enclosing an image tag hosted on a tracker like `taboola.com`), a GTK confirmation dialog is presented:
1. **Single Target:** If only one candidate URL is identified, the browser asks for confirmation: *"Are you sure you want to block & poison target X?"*.
2. **Multiple Targets:** If multiple candidates are found, the browser displays a dropdown selector listing all detected URLs/domains, allowing the user to select the specific tracking domain to poison (e.g. selecting `taboola.com` while leaving `reddit.com` unblocked).
3. **Cancellation:** The user can cancel the action entirely to prevent false positives.
