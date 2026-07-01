// ── Anti-Fingerprinting Module ────────────────────────────────
//
// Here live all the spoofing strategies.
// They are injected before the page's JS executes via
// WebKit's UserContentManager, into ALL frames (including
// iframes) to prevent sub-frame bypass.
//
// Do NOT inject this into a single top-frame only — trackers
// spin up invisible iframes to read the clean OS navigator.

/// JS payload injected into EVERY page and EVERY sub-frame
/// before any page script executes.
pub fn anti_fingerprint_script() -> &'static str {
    r#"
    (function() {
        'use strict';

        // ── Viewport Fingerprinting ──────────────────────────
        const _randInt = (n) => Math.floor(Math.random() * n);
        const fakeWidth  = screen.width  + _randInt(50) - 25;
        const fakeHeight = screen.height + _randInt(50) - 25;

        Object.defineProperty(screen, 'width',       { get: () => fakeWidth });
        Object.defineProperty(screen, 'height',      { get: () => fakeHeight });
        Object.defineProperty(screen, 'availWidth',  { get: () => fakeWidth });
        Object.defineProperty(screen, 'availHeight', { get: () => fakeHeight });
        Object.defineProperty(window, 'innerWidth',  { get: () => fakeWidth });
        Object.defineProperty(window, 'innerHeight', { get: () => fakeHeight });

        // ── Canvas Fingerprinting ────────────────────────────
        // We inject a static base64 image instead of noisy pixels.
        // Pixel noise is insufficient — getImageData reads the real buffer.
        // We intercept ALL three read paths to guarantee no leaks.
        const _fakeDataURL = 'data:image/jpeg;base64,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 '
        HTMLCanvasElement.prototype.toDataURL = function() { return _fakeDataURL; };
        HTMLCanvasElement.prototype.toBlob    = function(cb) { fetch(_fakeDataURL).then(r=>r.blob()).then(cb); };
        const _origGetImageData = CanvasRenderingContext2D.prototype.getImageData;
        CanvasRenderingContext2D.prototype.getImageData = function(x, y, w, h) {
            const d = _origGetImageData.call(this, x, y, w, h);
            for (let i = 0; i < d.data.length; i += 4) { d.data[i] ^= _randInt(3); }
            return d;
        };

        // ── WebGL Fingerprinting ─────────────────────────────
        const _origGetParam = WebGLRenderingContext.prototype.getParameter;
        WebGLRenderingContext.prototype.getParameter = function(param) {
            if (param === 37445) return 'Juanita Banana GPU';            // UNMASKED_VENDOR_WEBGL
            if (param === 37446) return 'Juanita Banana Graphics API';   // UNMASKED_RENDERER_WEBGL
            return _origGetParam.call(this, param);
        };

        // ── Navigator Fingerprinting ─────────────────────────
        Object.defineProperty(navigator, 'hardwareConcurrency',
            { get: () => 4 + _randInt(4) });
        Object.defineProperty(navigator, 'deviceMemory',
            { get: () => 8 });
        Object.defineProperty(navigator, 'platform',
            { get: () => 'Linux x86_64' });
        Object.defineProperty(navigator, 'vendor',
            { get: () => 'Juanita Banana' });
        Object.defineProperty(navigator, 'userAgent',
            { get: () => 'JuanitaBanana/0.1 (FOSS; Not-Google; Linux)' });

        // Bot detection bypass: these fields expose automation
        Object.defineProperty(navigator, 'webdriver',
            { get: () => false });
        Object.defineProperty(navigator, 'languages',
            { get: () => ['en-US', 'en'] });
        Object.defineProperty(navigator, 'plugins', {
            get: () => Object.setPrototypeOf([
                { name: 'PDF Viewer', filename: 'internal-pdf-viewer', description: 'Portable Document Format' },
                { name: 'Chrome PDF Viewer', filename: 'mhjfbmdgcfjbbpaeojofohoefgiehjai', description: '' },
            ], PluginArray.prototype)
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

        console.log('[JuanitaBanana] Anti-fingerprint active 🍌');
    })();
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anti_fingerprint_script_contains_overrides() {
        let script = anti_fingerprint_script();

        // Viewport
        assert!(script.contains("Object.defineProperty(screen, 'width'"));
        assert!(script.contains("Object.defineProperty(window, 'innerHeight'"));

        // GPU
        assert!(script.contains("Juanita Banana GPU"));
        assert!(script.contains("Juanita Banana Graphics API"));

        // Navigator
        assert!(script.contains("JuanitaBanana/0.1"));
        assert!(script.contains("webdriver"));

        // Timezone
        assert!(script.contains("Intl.DateTimeFormat"));
        assert!(script.contains("Europe/London"));
    }
}
