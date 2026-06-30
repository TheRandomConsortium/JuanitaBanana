// ── Anti-Fingerprinting Module ────────────────────────────────
//
// Here live all the spoofing strategies.
// They are injected before the page's JS executes.
//
// The mechanism: Servo calls EmbedderMethods::get_user_agent()
// and similar hooks. We return fake/noisy data.
// For canvas and viewport, we inject an override script that
// runs in the page context before any other script
// (equivalent to about:blank → document_begin).

/// JS script injected into EVERY page before its own JS.
/// Overwrites fingerprinting APIs with noisy values.
pub fn anti_fingerprint_script() -> &'static str {
    r#"
    (function() {
        'use strict';

        // ── Viewport Fingerprinting ──────────────────────────
        // We return slightly randomized dimensions.
        // The variation is small so as not to break layouts but
        // enough to make the fingerprint useless.
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
        // We intercept toDataURL() and toBlob() to inject
        // minimal visual noise into the canvas before exporting.
        const _origToDataURL = HTMLCanvasElement.prototype.toDataURL;
        const _origToBlob    = HTMLCanvasElement.prototype.toBlob;
        const _addNoise = (canvas) => {
            const ctx = canvas.getContext('2d');
            if (!ctx) return;
            // We modify 1 random pixel with minimal alpha variation
            const x = _randInt(canvas.width  || 1);
            const y = _randInt(canvas.height || 1);
            const d = ctx.getImageData(x, y, 1, 1);
            d.data[0] = (d.data[0] + _randInt(3)) & 0xFF;
            ctx.putImageData(d, x, y);
        };
        HTMLCanvasElement.prototype.toDataURL = function(...args) {
            _addNoise(this);
            return _origToDataURL.apply(this, args);
        };
        HTMLCanvasElement.prototype.toBlob = function(cb, ...args) {
            _addNoise(this);
            return _origToBlob.call(this, cb, ...args);
        };

        // ── WebGL Fingerprinting ─────────────────────────────
        const _origGetParam = WebGLRenderingContext.prototype.getParameter;
        WebGLRenderingContext.prototype.getParameter = function(param) {
            // UNMASKED_VENDOR_WEBGL and UNMASKED_RENDERER_WEBGL
            if (param === 37445) return 'Juanita Banana GPU';
            if (param === 37446) return 'JB Renderer (Not-Google)';
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

        console.log('[JuanitaBanana] Anti-fingerprint active 🍌');
    })();
    "#
}

/// User-Agent that servers see.
/// We are what we are. No disguises.
pub const USER_AGENT: &str =
    "Mozilla/5.0 JuanitaBanana/0.1 (FOSS; Not-Google; Linux) AppleWebKit/605.1.15";
