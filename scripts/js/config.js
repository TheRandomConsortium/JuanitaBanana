if (window.location.hash === '#unban') {
    showTab('unban');
}

if (window.location.href.includes('saved=true')) {
    const btn = document.querySelector('button[onclick="saveConfig()"]');
    if(btn) {
        btn.innerText = "Saved!";
        setTimeout(() => { btn.innerText = "Save Configuration"; }, 3000);
    }
}

if (window.location.href.includes('unlock_pass=') || window.location.href.includes('unlock_error=') || window.location.href.includes('saved_secure=') || window.location.href.includes('secure_init=')) {
    showTab('secure-db');
}

// Update guilt trip opacity label in real-time
const opacitySlider = document.getElementById('guilt-trip-opacity');
if (opacitySlider) {
    opacitySlider.addEventListener('input', (e) => {
        const valSpan = document.getElementById('opacity-val');
        if (valSpan) valSpan.textContent = e.target.value;
    });
}

function showTab(tabId) {
    if (tabId !== 'unban' && window.location.hash === '#unban') return; // Enforce lock
    document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
    document.getElementById(tabId).classList.add('active');
    
    document.querySelectorAll('#sidebar li').forEach(el => el.classList.remove('active'));
    const activeLi = document.getElementById('li-' + tabId);
    if (activeLi) activeLi.classList.add('active');
}

function checkUnban() {
    const ans = document.getElementById('math-answer').value;
    const result = document.getElementById('unban-result');
    result.textContent = '';
    const br = document.createElement('br');
    result.appendChild(br);
    if (ans.trim() === "2") {
        const a = document.createElement('a');
        a.href = 'juanita://unban-page';
        a.textContent = 'Manage Bans';
        result.appendChild(a);
    } else {
        const span = document.createElement('span');
        span.style.color = 'red';
        span.textContent = 'Incorrect. Back to the abyss.';
        result.appendChild(span);
    }
}

function saveConfig() {
    const rssRows = document.querySelectorAll('#rss-tbody tr');
    const newRss = [];
    rssRows.forEach(row => {
        const name = row.cells[0].textContent;
        const url = row.cells[1].textContent;
        newRss.push({ name, url });
    });
    
    const enginesRows = document.querySelectorAll('#engines-tbody tr');
    const newEngines = [];
    enginesRows.forEach(row => {
        const name = row.cells[0].textContent;
        const domain_regex = row.cells[1].textContent;
        const query_params = row.cells[2].textContent.split(',').map(s => s.trim()).filter(s => s.length > 0);
        newEngines.push({ name, domain_regex, query_params });
    });

    const configData = {json_data};
    configData.rss_sources = newRss;
    configData.search_engines = newEngines;
    configData.max_concurrent_searches = parseInt(document.getElementById('max-concurrent').value, 10);
    configData.min_delay_ms = parseInt(document.getElementById('min-delay').value, 10);
    configData.max_delay_ms = parseInt(document.getElementById('max-delay').value, 10);
    configData.noise_queries_amount = parseInt(document.getElementById('noise-amount').value, 10);

    configData.ad_click_probability = parseFloat(document.getElementById('ad-click-prob').value);
    configData.ad_jitter_min_secs = parseInt(document.getElementById('ad-jitter-min').value, 10);
    configData.ad_jitter_max_secs = parseInt(document.getElementById('ad-jitter-max').value, 10);
    configData.ad_intox_max_depth = parseInt(document.getElementById('ad-intox-max-depth').value, 10);
    configData.ad_intox_regex = document.getElementById('ad-intox-regex').value;
    configData.toxic_threshold = parseInt(document.getElementById('toxic-threshold').value, 10);
    configData.deep_crawl_max_pages = parseInt(document.getElementById('deep-crawl-max-pages').value, 10);
    configData.guilt_trip_enabled = document.getElementById('guilt-trip-enabled').checked;
    configData.guilt_trip_opacity = parseFloat(document.getElementById('guilt-trip-opacity').value);
    configData.guilt_trip_threshold = parseInt(document.getElementById('guilt-trip-threshold').value, 10);
    configData.guilt_trip_nsfw_rules = document.getElementById('guilt-trip-nsfw-rules').value.split(',').map(s => s.trim()).filter(Boolean);
    configData.guilt_trip_news_rules = document.getElementById('guilt-trip-news-rules').value.split(',').map(s => s.trim()).filter(Boolean);
    configData.guilt_trip_shopping_rules = document.getElementById('guilt-trip-shopping-rules').value.split(',').map(s => s.trim()).filter(Boolean);
    configData.guilt_trip_social_rules = document.getElementById('guilt-trip-social-rules').value.split(',').map(s => s.trim()).filter(Boolean);

    const adRows = document.querySelectorAll('#ad-domains-tbody tr');
    const newAdDomains = [];
    adRows.forEach(row => {
        const domain = row.cells[0].textContent.trim();
        if (domain) newAdDomains.push(domain);
    });
    configData.ad_domains = newAdDomains;
    
    const resolverItems = document.querySelectorAll('#resolver-list .resolver-item');
    const newResolverOrder = [];
    resolverItems.forEach(item => {
        const name = item.getAttribute('data-name');
        if (name) newResolverOrder.push(name);
    });
    configData.resolver_order = newResolverOrder;
    configData.handshake_enabled = document.getElementById('handshake-enabled').checked;
    configData.tor_enabled = document.getElementById('tor-enabled').checked;
    configData.tor_route_all = document.getElementById('tor-route-all').checked;
    configData.tab_inactivity_ttl = parseInt(document.getElementById('tab-inactivity-ttl').value, 10);
    configData.last_tab_nuke_action = document.getElementById('last-tab-nuke-action').value;

    window.location.href = "juanita://save-config?data=" + encodeURIComponent(JSON.stringify(configData));
}

function addRss() {
    const name = document.getElementById('new-rss-name').value;
    const url = document.getElementById('new-rss-url').value;
    if (!name || !url) return;

    const tbody = document.getElementById('rss-tbody');
    const row = document.createElement('tr');

    const tdName = document.createElement('td');
    tdName.textContent = name;
    const tdUrl = document.createElement('td');
    tdUrl.textContent = url;
    const tdBtn = document.createElement('td');
    const btn = document.createElement('button');
    btn.textContent = 'X';
    btn.style.cssText = 'margin:0; padding: 5px;';
    btn.addEventListener('click', () => row.remove());
    tdBtn.appendChild(btn);

    row.appendChild(tdName);
    row.appendChild(tdUrl);
    row.appendChild(tdBtn);
    tbody.appendChild(row);

    document.getElementById('new-rss-name').value = '';
    document.getElementById('new-rss-url').value = '';
}

function addEngine() {
    const name = document.getElementById('new-engine-name').value;
    const regex = document.getElementById('new-engine-regex').value;
    const params = document.getElementById('new-engine-params').value;
    if (!name || !regex || !params) return;

    const tbody = document.getElementById('engines-tbody');
    const row = document.createElement('tr');

    const tdName = document.createElement('td');
    tdName.textContent = name;
    const tdRegex = document.createElement('td');
    tdRegex.textContent = regex;
    const tdParams = document.createElement('td');
    tdParams.textContent = params;
    const tdBtn = document.createElement('td');
    const btn = document.createElement('button');
    btn.textContent = 'X';
    btn.style.cssText = 'margin:0; padding: 5px;';
    btn.addEventListener('click', () => row.remove());
    tdBtn.appendChild(btn);

    row.appendChild(tdName);
    row.appendChild(tdRegex);
    row.appendChild(tdParams);
    row.appendChild(tdBtn);
    tbody.appendChild(row);

    document.getElementById('new-engine-name').value = '';
    document.getElementById('new-engine-regex').value = '';
    document.getElementById('new-engine-params').value = '';
}

function addAdDomain() {
    const domain = document.getElementById('new-ad-domain').value.trim();
    if (!domain) return;
    const tbody = document.getElementById('ad-domains-tbody');
    const row = document.createElement('tr');

    const tdDomain = document.createElement('td');
    tdDomain.textContent = domain;
    const tdBtn = document.createElement('td');
    const btn = document.createElement('button');
    btn.textContent = 'X';
    btn.style.cssText = 'margin:0; padding: 5px;';
    btn.addEventListener('click', () => row.remove());
    tdBtn.appendChild(btn);

    row.appendChild(tdDomain);
    row.appendChild(tdBtn);
    tbody.appendChild(row);
    document.getElementById('new-ad-domain').value = '';
}

function unlockSecureDb() {
    const pass = document.getElementById('secure-unlock-pass').value;
    if (!pass) return;
    window.location.href = "juanita://config?unlock_pass=" + encodeURIComponent(pass);
}

function saveSecureConfig(isInit) {
    let pass;
    if (isInit) {
        pass = document.getElementById('secure-init-pass').value;
        if (!pass) { alert('Master Password is required to initialize.'); return; }
    } else {
        pass = document.getElementById('secure-pass').value;
    }
    const name = document.getElementById('secure-name').value;
    const id = document.getElementById('secure-id').value;
    if (!name || !id) { alert('Full Name and National ID are required.'); return; }

    const smtp_server = document.getElementById('secure-smtp-server').value;
    const smtp_port = document.getElementById('secure-smtp-port').value;
    const smtp_user = document.getElementById('secure-smtp-user').value;
    const smtp_pass = document.getElementById('secure-smtp-pass').value;

    const pop_server = document.getElementById('secure-pop-server').value;
    const pop_port = document.getElementById('secure-pop-port').value;
    const pop_user = document.getElementById('secure-pop-user').value;
    const pop_pass = document.getElementById('secure-pop-pass').value;

    const query = "?pass=" + encodeURIComponent(pass) +
                  "&name=" + encodeURIComponent(name) +
                  "&id=" + encodeURIComponent(id) +
                  "&smtp_server=" + encodeURIComponent(smtp_server) +
                  "&smtp_port=" + encodeURIComponent(smtp_port) +
                  "&smtp_user=" + encodeURIComponent(smtp_user) +
                  "&smtp_pass=" + encodeURIComponent(smtp_pass) +
                  "&pop_server=" + encodeURIComponent(pop_server) +
                  "&pop_port=" + encodeURIComponent(pop_port) +
                  "&pop_user=" + encodeURIComponent(pop_user) +
                  "&pop_pass=" + encodeURIComponent(pop_pass);

    window.location.href = "juanita://save-secure-config" + query;
}

function moveResolverUp(button) {
    const li = button.closest('.resolver-item');
    const prev = li.previousElementSibling;
    if (prev) {
        li.parentNode.insertBefore(li, prev);
    }
}

function moveResolverDown(button) {
    const li = button.closest('.resolver-item');
    const next = li.nextElementSibling;
    if (next) {
        li.parentNode.insertBefore(next, li);
    }
}
