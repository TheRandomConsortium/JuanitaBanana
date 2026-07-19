(function() {
    'use strict';
    const adDomains = AD_DOMAINS_PLACEHOLDER;

    function isAdUrl(url) {
        if (!url) return false;
        try {
            const host = new URL(url).hostname;
            return adDomains.some(domain => host.includes(domain));
        } catch(e) {
            return adDomains.some(domain => url.includes(domain));
        }
    }

    function cleanAdContainer(el) {
        let parent = el.parentElement;
        const adRegexStr = AD_REGEX_PLACEHOLDER;
        const adRegex = new RegExp(adRegexStr, "i");
        const maxDepth = AD_MAX_DEPTH_PLACEHOLDER;
        for (let i = 0; i < maxDepth; i++) {
            if (!parent) break;
            let text = parent.innerText || parent.textContent || "";
            let cleanText = text.replace(/\s+/g, ' ').trim().toLowerCase();
            
            let idOrClass = (parent.id + " " + parent.className).toLowerCase();
            let isAdContainer = adRegex.test(idOrClass) 
                             || (cleanText.length < 30 && adRegex.test(cleanText))
                             || (parent.children.length <= 1 && cleanText.length === 0);
            
            if (isAdContainer) {
                let nextParent = parent.parentElement;
                parent.remove();
                parent = nextParent;
            } else {
                break;
            }
        }
    }

    function reportAd(el, adUrl) {
        if (el.dataset.juanitaProcessed) return;
        el.dataset.juanitaProcessed = "true";

        let selector = el.id ? "#" + el.id : el.tagName.toLowerCase();
        if (el.className) selector += "." + Array.from(el.classList).join(".");
        
        const src = el.getAttribute('src');
        const href = el.getAttribute('href');
        if (src) {
            selector += '[src="' + src + '"]';
        } else if (href) {
            selector += '[href="' + href + '"]';
        }

        try {
            window.top.postMessage({
                type: 'juanita_track_or_ad',
                isAd: true,
                count: 1
            }, '*');
        } catch(e) {}

        if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.juanita) {
            window.webkit.messageHandlers.juanita.postMessage(JSON.stringify({
                type: "ad_detected",
                page_url: window.location.href,
                selector: selector,
                ad_url: adUrl
            }));
        }

        console.log("[Juanita] Surgery on intent: removing element:", selector);
        cleanAdContainer(el);
        if (el.parentElement) {
            el.remove();
        }
    }

    const patchSrc = (proto) => {
        const desc = Object.getOwnPropertyDescriptor(proto, 'src');
        if (desc && desc.set) {
            const originalSet = desc.set;
            Object.defineProperty(proto, 'src', {
                set: function(val) {
                    if (isAdUrl(val)) {
                        reportAd(this, val);
                    }
                    return originalSet.call(this, val);
                },
                get: desc.get,
                configurable: true
            });
        }
    };
    if (window.HTMLImageElement) patchSrc(HTMLImageElement.prototype);
    if (window.HTMLIFrameElement) patchSrc(HTMLIFrameElement.prototype);
    if (window.HTMLScriptElement) patchSrc(HTMLScriptElement.prototype);

    const origSetAttribute = Element.prototype.setAttribute;
    Element.prototype.setAttribute = function(name, value) {
        if ((name === 'src' || name === 'href') && isAdUrl(value)) {
            reportAd(this, value);
        }
        return origSetAttribute.call(this, name, value);
    };

    // Monkeypatch window.fetch to catch fetch intents
    const originalFetch = window.fetch;
    window.fetch = function(input, init) {
        let url = "";
        if (typeof input === 'string') {
            url = input;
        } else if (input && input.url) {
            url = input.url;
        }
        if (isAdUrl(url)) {
            try {
                window.top.postMessage({
                    type: 'juanita_track_or_ad',
                    isAd: true,
                    count: 1
                }, '*');
            } catch(e) {}
            if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.juanita) {
                window.webkit.messageHandlers.juanita.postMessage(JSON.stringify({
                    type: "ad_detected",
                    page_url: window.location.href,
                    selector: "fetch:" + url,
                    ad_url: url
                }));
            }
        }
        return originalFetch.apply(this, arguments);
    };

    // Monkeypatch XMLHttpRequest to catch XHR intents
    const originalOpen = XMLHttpRequest.prototype.open;
    XMLHttpRequest.prototype.open = function(method, url) {
        this._url = url;
        return originalOpen.apply(this, arguments);
    };
    const originalSend = XMLHttpRequest.prototype.send;
    XMLHttpRequest.prototype.send = function() {
        if (isAdUrl(this._url)) {
            try {
                window.top.postMessage({
                    type: 'juanita_track_or_ad',
                    isAd: true,
                    count: 1
                }, '*');
            } catch(e) {}
            if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.juanita) {
                window.webkit.messageHandlers.juanita.postMessage(JSON.stringify({
                    type: "ad_detected",
                    page_url: window.location.href,
                    selector: "xhr:" + this._url,
                    ad_url: this._url
                }));
            }
        }
        return originalSend.apply(this, arguments);
    };

    // Future Planned Feature: navigator.sendBeacon
    // Currently, sendBeacon is not intercepted but will be supported in future versions
    // to poison backend metrics by replacing/adding fake telemetry to beacon payloads.

    function processElement(el) {
        let isAd = false;
        let adUrl = "";

        if (el.tagName === 'IFRAME' || el.tagName === 'IMG' || el.tagName === 'SCRIPT') {
            const src = el.getAttribute('src');
            if (isAdUrl(src)) {
                isAd = true;
                adUrl = src;
            }
        }
        if (el.tagName === 'A') {
            const href = el.getAttribute('href');
            if (isAdUrl(href)) {
                isAd = true;
                adUrl = href;
            }
        }

        if (isAd) {
            reportAd(el, adUrl);
        }
    }

    function scanDOM() {
        const els = document.querySelectorAll('iframe, img, a, script');
        els.forEach(processElement);
    }

    const observer = new MutationObserver((mutations) => {
        observer.disconnect();
        scanDOM();
        observer.observe(document.documentElement || document.body, {
            childList: true,
            subtree: true
        });
    });

    observer.observe(document.documentElement || document.body, {
        childList: true,
        subtree: true
    });

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            scanDOM();
        });
    } else {
        scanDOM();
    }

    function sendRightClick(e) {
        let src = "";
        let href = "";
        let el = e.target;
        if (el) {
            if (el.tagName === 'A') {
                href = el.href || el.getAttribute('href') || "";
            } else if (el.tagName === 'IMG' || el.tagName === 'IFRAME' || el.tagName === 'SCRIPT') {
                src = el.src || el.getAttribute('src') || "";
            }
            
            let parent = el.parentElement;
            for (let i = 0; i < 5; i++) {
                if (!parent) break;
                if (!href && parent.tagName === 'A') {
                    href = parent.href || parent.getAttribute('href') || "";
                }
                parent = parent.parentElement;
            }
        }
        
        if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.juanita) {
            window.webkit.messageHandlers.juanita.postMessage(JSON.stringify({
                type: "right_click_target",
                frame_url: window.location.href,
                target_src: src,
                target_href: href
            }));
        }
    }
    window.addEventListener('mousedown', (e) => {
        if (e.button === 2) {
            sendRightClick(e);
        }
    });
    window.addEventListener('contextmenu', (e) => {
        sendRightClick(e);
    });
})();
