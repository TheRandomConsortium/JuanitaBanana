import re

with open('src/spoof.rs', 'r') as f:
    content = f.read()

# We need to replace the entire anti_fingerprint_script function
# to correctly use format!() and escape all curly braces.

# Find the start and end of the function
start_idx = content.find('pub fn anti_fingerprint_script() -> String {')
end_idx = content.find('pub fn get_daily_user_agent() -> &\'static str {')

if start_idx != -1 and end_idx != -1:
    new_func = """pub fn anti_fingerprint_script() -> String {
    let ua = get_daily_user_agent();
    format!(
        r#"
    (function() {{
        'use strict';

        // ── Viewport Fingerprinting ──────────────────────────
        const _randInt = (n) => Math.floor(Math.random() * n);
        const fakeWidth  = screen.width  + _randInt(50) - 25;
        const fakeHeight = screen.height + _randInt(50) - 25;

        Object.defineProperty(screen, 'width',       {{ get: () => fakeWidth }});
        Object.defineProperty(screen, 'height',      {{ get: () => fakeHeight }});
        Object.defineProperty(screen, 'availWidth',  {{ get: () => fakeWidth }});
        Object.defineProperty(screen, 'availHeight', {{ get: () => fakeHeight }});
        Object.defineProperty(window, 'innerWidth',  {{ get: () => fakeWidth }});
        Object.defineProperty(window, 'innerHeight', {{ get: () => fakeHeight }});

        // ── Canvas Fingerprinting ────────────────────────────
        const _origToDataURL = HTMLCanvasElement.prototype.toDataURL;
        const _origToBlob    = HTMLCanvasElement.prototype.toBlob;
        const _addNoise = (canvas) => {{
            const ctx = canvas.getContext('2d');
            if (!ctx) return;
            const x = _randInt(canvas.width  || 1);
            const y = _randInt(canvas.height || 1);
            const d = ctx.getImageData(x, y, 1, 1);
            d.data[0] = (d.data[0] + _randInt(3)) & 0xFF;
            ctx.putImageData(d, x, y);
        }};
        HTMLCanvasElement.prototype.toDataURL = function(...args) {{
            _addNoise(this);
            return _origToDataURL.apply(this, args);
        }};
        HTMLCanvasElement.prototype.toBlob = function(cb, ...args) {{
            _addNoise(this);
            return _origToBlob.call(this, cb, ...args);
        }};

        // ── WebGL Fingerprinting ─────────────────────────────
        const _origGetParam = WebGLRenderingContext.prototype.getParameter;
        WebGLRenderingContext.prototype.getParameter = function(param) {{
            if (param === 37445) return 'Juanita Banana GPU'; // UNMASKED_VENDOR_WEBGL
            if (param === 37446) return 'Juanita Banana Graphics API'; // UNMASKED_RENDERER_WEBGL
            return _origGetParam.call(this, param);
        }};

        // ── Navigator Fingerprinting ─────────────────────────
        Object.defineProperty(navigator, 'hardwareConcurrency',
            {{ get: () => 4 + _randInt(4) }});
        Object.defineProperty(navigator, 'deviceMemory',
            {{ get: () => 8 }});
        Object.defineProperty(navigator, 'platform',
            {{ get: () => 'Linux x86_64' }});
        Object.defineProperty(navigator, 'vendor',
            {{ get: () => 'Juanita Banana' }});
        Object.defineProperty(navigator, 'userAgent',
            {{ get: () => '{0}' }});

        let navbar = document.createElement('div');
        navbar.innerHTML = `
            <div style="display: flex; flex-direction: row; align-items: center; width: 100%; height: 100%;">
                <span style="font-weight: bold; margin-right: 10px; font-family: sans-serif;">🍌 Juanita</span>
                <input type="text" id="j-nav-input" placeholder="Search or enter address..." style="flex-grow: 1; padding: 5px; border-radius: 4px; border: 1px solid #ccc; color: black; background: white;">
                <button id="j-nav-ban" style="margin-left: 10px; background: red; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-weight: bold;">BAN</button>
            </div>
        `;
        navbar.style.cssText = 'position:fixed; top:0; left:0; width:100%; height:40px; background:#222; color:white; z-index:2147483647; padding: 5px; box-sizing: border-box;';
        
        // Wait for body to be available
        let observer = new MutationObserver((mutations, obs) => {{
            if (document.body) {{
                document.body.appendChild(navbar);
                document.body.style.marginTop = "40px"; // Push content down
                
                document.getElementById('j-nav-ban').onclick = function() {{
                    window.location.href = 'juanita://ban';
                }};
                document.getElementById('j-nav-input').onkeydown = function(e) {{
                    if (e.key === 'Enter') {{
                        window.location.href = 'juanita://nav?url=' + encodeURIComponent(this.value);
                    }}
                }};
                obs.disconnect();
            }}
        }});
        observer.observe(document.documentElement, {{ childList: true, subtree: true }});

        console.log('[JuanitaBanana] Anti-fingerprint & Navbar active 🍌');
    }})();
    "#,
        ua
    )
}

"""
    # Replace
    # end_idx needs to backtrack to "use std::time::"
    real_end = content.find("use std::time::", start_idx)
    content = content[:start_idx] + new_func + content[real_end:]
    
    with open('src/spoof.rs', 'w') as f:
        f.write(content)
    print("Fixed spoof.rs")
