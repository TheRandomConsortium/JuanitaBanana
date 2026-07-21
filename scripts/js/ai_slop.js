(function() {
    if (window.__juanita_ai_slop_checked) return;
    window.__juanita_ai_slop_checked = true;

    const phrases = AI_PHRASES_PLACEHOLDER;
    if (!phrases || !Array.isArray(phrases) || phrases.length === 0) return;

    function ensureStyles() {
        if (document.getElementById("jb-ai-slop-styles")) return;
        const style = document.createElement("style");
        style.id = "jb-ai-slop-styles";
        style.textContent = `
            TOKENS_CSS_PLACEHOLDER

            .jb-ai-slop-neutralizer {
                background: var(--jb-surface-sidebar, #141419) !important;
                color: var(--jb-text-primary, #f3f4f6) !important;
                border: 2px solid var(--jb-accent-yellow, #facc15) !important;
                border-radius: var(--jb-radius-md, 12px) !important;
                padding: var(--jb-space-xl, 20px) !important;
                margin: var(--jb-space-xl, 20px) 0 !important;
                font-family: var(--jb-font-family, Outfit, system-ui, sans-serif) !important;
                box-shadow: var(--jb-shadow-card, 0 10px 30px -10px rgba(0, 0, 0, 0.7)) !important;
                display: flex !important;
                flex-direction: column !important;
                gap: var(--jb-space-md, 10px) !important;
                z-index: 999999 !important;
                box-sizing: border-box !important;
                width: 100% !important;
            }
            .jb-ai-slop-neutralizer .jb-slop-title {
                font-weight: var(--jb-font-weight-bold, 700) !important;
                font-size: var(--jb-font-size-lg, 1.1em) !important;
                color: var(--jb-accent-yellow, #facc15) !important;
                display: flex !important;
                align-items: center !important;
                gap: var(--jb-space-xs, 6px) !important;
                margin: 0 !important;
            }
            .jb-ai-slop-neutralizer .jb-slop-text {
                color: var(--jb-text-primary, #f3f4f6) !important;
                font-size: var(--jb-font-size-base, 0.95em) !important;
                line-height: 1.5 !important;
                margin: 0 !important;
            }
            .jb-ai-slop-neutralizer .jb-slop-hint {
                color: var(--jb-text-secondary, #9ca3af) !important;
                font-size: var(--jb-font-size-xs, 0.85em) !important;
                font-style: italic !important;
                margin: 0 !important;
            }
            .jb-ai-slop-neutralizer .jb-slop-btn-group {
                display: flex !important;
                gap: var(--jb-space-md, 12px) !important;
                margin-top: var(--jb-space-xs, 4px) !important;
            }
            .jb-ai-slop-neutralizer .jb-slop-btn-alt {
                background: var(--jb-accent-yellow, #facc15) !important;
                color: var(--jb-accent-yellow-dark, #0d0d11) !important;
                border: none !important;
                padding: 8px 16px !important;
                border-radius: var(--jb-radius-md, 8px) !important;
                font-weight: var(--jb-font-weight-bold, 700) !important;
                cursor: pointer !important;
                font-size: var(--jb-font-size-sm, 0.88em) !important;
                font-family: inherit !important;
                transition: var(--jb-transition-base) !important;
            }
            .jb-ai-slop-neutralizer .jb-slop-btn-alt:hover {
                background: var(--jb-accent-yellow-hover, #fde047) !important;
            }
        `;
        (document.head || document.documentElement).appendChild(style);
    }

    function detectSlop() {
        const textContent = document.body ? document.body.innerText.toLowerCase() : "";
        let detected = false;
        let matchedPhrase = "";

        for (const phrase of phrases) {
            const p = phrase.toLowerCase().trim();
            if (p && textContent.includes(p)) {
                detected = true;
                matchedPhrase = phrase;
                break;
            }
        }

        if (!detected) return;

        // Inject tokens & component styles onto 3rd party web page
        ensureStyles();

        // Post message to Juanita script message handler
        try {
            if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.juanita) {
                window.webkit.messageHandlers.juanita.postMessage({
                    type: "ai_slop_detected",
                    matched: matchedPhrase
                });
            }
        } catch (e) {}

        // Inject neutralization banner using design system tokenized rules
        const banner = document.createElement("div");
        banner.className = "jb-ai-slop-neutralizer";

        const title = document.createElement("div");
        title.className = "jb-slop-title";
        title.innerHTML = "<span>⚠️ AI Slop Detected</span>";

        const msg = document.createElement("div");
        msg.className = "jb-slop-text";
        msg.textContent = 'This page relies on AI-generated content (matched: "' + matchedPhrase + '"). You might as well generate this yourself and do it privately in the meantime.';

        const banHint = document.createElement("div");
        banHint.className = "jb-slop-hint";
        banHint.textContent = "Or if you've had enough of this slop factory, hit that shiny red BAN button up top!";

        const btnGroup = document.createElement("div");
        btnGroup.className = "jb-slop-btn-group";

        const altBtn = document.createElement("button");
        altBtn.className = "jb-slop-btn-alt";
        altBtn.textContent = "Explore Privacy-First AI Alternatives";
        altBtn.onclick = function() {
            window.location.href = "juanita://ai-alternatives";
        };

        btnGroup.appendChild(altBtn);
        banner.appendChild(title);
        banner.appendChild(msg);
        banner.appendChild(banHint);
        banner.appendChild(btnGroup);

        const targetContainer = document.querySelector("article") || document.querySelector("main") || document.body.firstElementChild || document.body;
        if (targetContainer && targetContainer.parentNode) {
            targetContainer.parentNode.insertBefore(banner, targetContainer);
        } else if (document.body) {
            document.body.prepend(banner);
        }
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", detectSlop);
    } else {
        detectSlop();
    }
})();
