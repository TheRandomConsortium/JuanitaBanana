(function() {
    'use strict';

    let adCount = 0;
    let trackingCount = 0;
    let marqueeEl = null;
    const threshold = TOXIC_THRESHOLD_PLACEHOLDER;

    function createMarquee() {
        if (marqueeEl) return;
        marqueeEl = document.createElement('div');
        marqueeEl.id = 'juanita-toxic-marquee';
        const style = document.createElement('style');
        style.textContent = `
            #juanita-toxic-marquee {
                position: fixed;
                bottom: 0;
                left: 0;
                right: 0;
                height: 40px;
                background: rgba(185, 28, 28, 0.85);
                backdrop-filter: blur(12px);
                -webkit-backdrop-filter: blur(12px);
                color: #ffffff;
                font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
                font-size: 14px;
                font-weight: 600;
                z-index: 2147483647;
                display: flex;
                align-items: center;
                border-top: 1px solid rgba(255, 255, 255, 0.2);
                box-shadow: 0 -10px 25px -5px rgba(0, 0, 0, 0.5);
                pointer-events: auto;
                overflow: hidden;
                user-select: none;
            }
            #juanita-toxic-marquee .marquee-content-wrapper {
                display: flex;
                width: 100%;
                align-items: center;
                justify-content: space-between;
                padding: 0 20px;
            }
            #juanita-toxic-marquee .marquee-text-container {
                overflow: hidden;
                white-space: nowrap;
                flex-grow: 1;
                margin-right: 20px;
            }
            #juanita-toxic-marquee .marquee-text {
                display: inline-block;
                white-space: nowrap;
                animation: juanita-marquee-anim 25s linear infinite;
            }
            #juanita-toxic-marquee .ban-btn {
                background: #ffffff;
                color: #dc2626;
                border: none;
                padding: 6px 16px;
                border-radius: 6px;
                cursor: pointer;
                font-weight: 700;
                font-size: 12px;
                transition: background 0.2s ease, transform 0.1s ease;
                flex-shrink: 0;
                box-shadow: 0 2px 4px rgba(0,0,0,0.15);
            }
            #juanita-toxic-marquee .ban-btn:hover {
                background: #f3f4f6;
                transform: scale(1.05);
            }
            #juanita-toxic-marquee .ban-btn:active {
                transform: scale(0.95);
            }
            @keyframes juanita-marquee-anim {
                0% { transform: translateX(100%); }
                100% { transform: translateX(-100%); }
            }
        `;
        document.head.appendChild(style);

        const wrapper = document.createElement('div');
        wrapper.className = 'marquee-content-wrapper';

        const textContainer = document.createElement('div');
        textContainer.className = 'marquee-text-container';
        const textSpan = document.createElement('span');
        textSpan.className = 'marquee-text';
        textSpan.id = 'juanita-marquee-text';
        textContainer.appendChild(textSpan);

        const banBtn = document.createElement('button');
        banBtn.className = 'ban-btn';
        banBtn.id = 'juanita-ban-btn';
        banBtn.textContent = '⚠️ Ban Domain';

        wrapper.appendChild(textContainer);
        wrapper.appendChild(banBtn);
        marqueeEl.appendChild(wrapper);

        document.body.appendChild(marqueeEl);
        document.getElementById('juanita-ban-btn').addEventListener('click', () => {
            if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.juanita) {
                window.webkit.messageHandlers.juanita.postMessage(JSON.stringify({
                    type: "ban_domain",
                    domain: window.location.hostname
                }));
            }
        });
    }

    function updateMarquee() {
        const total = adCount + trackingCount;
        if (total >= threshold) {
            if (!document.body) return;
            createMarquee();
            const textEl = document.getElementById('juanita-marquee-text');
            if (textEl) {
                textEl.textContent = `This website is a toxic wasteland! Detected ${adCount} ads and ${trackingCount} tracking attempts. You are invited to ban this domain.`;
            }
        }
    }

    window.addEventListener('message', (event) => {
        if (event.data && event.data.type === 'juanita_track_or_ad') {
            if (event.data.isAd) {
                adCount += event.data.count || 0;
            } else {
                trackingCount += event.data.count || 0;
            }
            updateMarquee();
        }
    });

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', updateMarquee);
    } else {
        updateMarquee();
    }
})();
