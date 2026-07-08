(function() {
    'use strict';

    let _attempts = 0;
    function _reportTrack(method) {
        _attempts++;
        console.log('[JuanitaBanana] Fingerprinting attempt detected via ' + method + ' (Total: ' + _attempts + ')');
        try {
            window.top.postMessage({
                type: 'juanita_track_or_ad',
                isAd: false,
                count: 1
            }, '*');
        } catch(e) {}
    }

    function _reportAndExecute(method, fn) {
        _reportTrack(method);
        return fn();
    }

    // ── Viewport Fingerprinting ──────────────────────────
    const _randInt = (n) => Math.floor(Math.random() * n);
    const fakeWidth  = screen.width  + _randInt(50) - 25;
    const fakeHeight = screen.height + _randInt(50) - 25;

    Object.defineProperty(screen, 'width',       { get: () => _reportAndExecute('screen.width', () => fakeWidth) });
    Object.defineProperty(screen, 'height',      { get: () => _reportAndExecute('screen.height', () => fakeHeight) });
    Object.defineProperty(screen, 'availWidth',  { get: () => _reportAndExecute('screen.availWidth', () => fakeWidth) });
    Object.defineProperty(screen, 'availHeight', { get: () => _reportAndExecute('screen.availHeight', () => fakeHeight) });
    Object.defineProperty(window, 'innerWidth',  { get: () => _reportAndExecute('window.innerWidth', () => fakeWidth) });
    Object.defineProperty(window, 'innerHeight', { get: () => _reportAndExecute('window.innerHeight', () => fakeHeight) });

    // ── Canvas Fingerprinting ────────────────────────────
    // We inject a static base64 image instead of noisy pixels.
    // Pixel noise is insufficient — getImageData reads the real buffer.
    // We intercept ALL three read paths to guarantee no leaks.
    const _fakeDataURL = 'data:image/jpeg;base64,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';
    HTMLCanvasElement.prototype.toDataURL = function() { return _reportAndExecute('canvas.toDataURL', () => _fakeDataURL); };
    HTMLCanvasElement.prototype.toBlob    = function(cb) { return _reportAndExecute('canvas.toBlob', () => { fetch(_fakeDataURL).then(r=>r.blob()).then(cb); }); };
    const _origGetImageData = CanvasRenderingContext2D.prototype.getImageData;
    CanvasRenderingContext2D.prototype.getImageData = function(x, y, w, h) {
        return _reportAndExecute('canvas.getImageData', () => {
            const d = _origGetImageData.call(this, x, y, w, h);
            for (let i = 0; i < d.data.length; i += 4) { d.data[i] ^= _randInt(3); }
            return d;
        });
    };

    // ── WebGL Fingerprinting ─────────────────────────────
    const _origGetParam = WebGLRenderingContext.prototype.getParameter;
    WebGLRenderingContext.prototype.getParameter = function(param) {
        if (param === 37445) return _reportAndExecute('webgl.vendor', () => 'Juanita Banana GPU');            // UNMASKED_VENDOR_WEBGL
        if (param === 37446) return _reportAndExecute('webgl.renderer', () => 'Juanita Banana Graphics API');   // UNMASKED_RENDERER_WEBGL
        return _origGetParam.call(this, param);
    };

    // ── Navigator Fingerprinting ─────────────────────────
    Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => _reportAndExecute('navigator.hardwareConcurrency', () => 4 + _randInt(4)) });
    Object.defineProperty(navigator, 'deviceMemory',        { get: () => _reportAndExecute('navigator.deviceMemory', () => 8) });
    Object.defineProperty(navigator, 'platform',            { get: () => _reportAndExecute('navigator.platform', () => 'Linux x86_64') });
    Object.defineProperty(navigator, 'vendor',              { get: () => _reportAndExecute('navigator.vendor', () => 'Juanita Banana') });
    Object.defineProperty(navigator, 'userAgent',           { get: () => _reportAndExecute('navigator.userAgent', () => 'JuanitaBanana/0.1 (FOSS; Not-Google; Linux)') });

    // Bot detection bypass: these fields expose automation
    Object.defineProperty(navigator, 'webdriver', { get: () => _reportAndExecute('navigator.webdriver', () => false) });
    Object.defineProperty(navigator, 'languages', { get: () => _reportAndExecute('navigator.languages', () => ['en-US', 'en']) });
    Object.defineProperty(navigator, 'plugins', {
        get: () => _reportAndExecute('navigator.plugins', () => Object.setPrototypeOf([
            { name: 'PDF Viewer', filename: 'internal-pdf-viewer', description: 'Portable Document Format' },
            { name: 'Chrome PDF Viewer', filename: 'mhjfbmdgcfjbbpaeojofohoefgiehjai', description: '' },
        ], PluginArray.prototype))
    });

    // ── Intl / Geolocation Leak ──────────────────────────
    // Timezone exposes physical location even when all other
    // signals are spoofed. We freeze it to a neutral value.
    const _origDateTimeFormat = Intl.DateTimeFormat;
    Intl.DateTimeFormat = function(locales, options) {
        if (options) { options.timeZone = 'Europe/London'; }
        else { options = { timeZone: 'Europe/London' }; }
        return new _origDateTimeFormat(locales, options);
    };
    Intl.DateTimeFormat.prototype = _origDateTimeFormat.prototype;
    Intl.DateTimeFormat.supportedLocalesOf = _origDateTimeFormat.supportedLocalesOf;

    // ── Battery API Spoofing ─────────────────────────────
    navigator.getBattery = function() {
        return _reportAndExecute('navigator.getBattery', () => Promise.resolve({
            charging: true,
            chargingTime: 0,
            dischargingTime: Infinity,
            level: 1.0,
            addEventListener: function() {},
            removeEventListener: function() {},
            dispatchEvent: function() { return true; },
            onchargingchange: null,
            onchargingtimechange: null,
            ondischargingtimechange: null,
            onlevelchange: null
        }));
    };

    // ── Font Enumeration Protection ──────────────────────
    const _origMeasureText = CanvasRenderingContext2D.prototype.measureText;
    CanvasRenderingContext2D.prototype.measureText = function(text) {
        return _reportAndExecute('canvas.measureText', () => {
            const originalFont = this.font;
            let forcedFont = 'monospace';
            if (originalFont.toLowerCase().includes('webdings')) {
                forcedFont = 'webdings';
            }
            let size = '10px';
            const sizeMatch = originalFont.match(/(\d+(?:\.\d+)?(?:px|em|pt|rem))/i);
            if (sizeMatch) { size = sizeMatch[1]; }
            this.font = size + ' ' + forcedFont;
            const metrics = _origMeasureText.call(this, text);
            this.font = originalFont;
            return metrics;
        });
    };

    if (window.FontFaceSet && FontFaceSet.prototype.check) {
        FontFaceSet.prototype.check = function(font) {
            return _reportAndExecute('FontFaceSet.check', () => {
                const lower = font.toLowerCase();
                return lower.includes('webdings') || lower.includes('monospace') || lower.includes('mono') || lower.includes('sans-serif') || lower.includes('serif');
            });
        };
    }

    console.log('[JuanitaBanana] Anti-fingerprint active 🍌');
})();
