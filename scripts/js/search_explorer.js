function filterAndSortTable() {
    const termRegexStr = document.getElementById('filter-term-regex').value.trim();
    const originStr = document.getElementById('filter-origin').value.trim().toLowerCase();
    const sortCol = document.getElementById('sort-col').value;
    const sortDir = document.getElementById('sort-dir').value;

    let termRegex = null;
    if (termRegexStr) {
        try {
            termRegex = new RegExp(termRegexStr, 'i');
        } catch (e) {
            termRegex = null;
        }
    }

    const tbody = document.getElementById('explorer-tbody');
    const rows = Array.from(tbody.querySelectorAll('tr'));
    let visibleCount = 0;

    rows.forEach(row => {
        const term = row.dataset.term || '';
        const origin = row.dataset.origin || '';

        let termMatch = true;
        if (termRegexStr) {
            if (termRegex) {
                termMatch = termRegex.test(term);
            } else {
                termMatch = term.toLowerCase().includes(termRegexStr.toLowerCase());
            }
        }

        let originMatch = true;
        if (originStr) {
            originMatch = origin.toLowerCase().includes(originStr);
        }

        if (termMatch && originMatch) {
            row.style.display = '';
            visibleCount++;
        } else {
            row.style.display = 'none';
        }
    });

    document.getElementById('visible-count').textContent = visibleCount;

    rows.sort((a, b) => {
        let valA = a.dataset[sortCol] || '';
        let valB = b.dataset[sortCol] || '';

        if (sortCol === 'date') {
            valA = parseInt(a.dataset.dateEpoch || '0', 10);
            valB = parseInt(b.dataset.dateEpoch || '0', 10);
        }

        let cmp = 0;
        if (valA < valB) cmp = -1;
        if (valA > valB) cmp = 1;

        return sortDir === 'asc' ? cmp : -cmp;
    });

    rows.forEach(row => tbody.appendChild(row));
}

function deleteTerm(encodedTerm) {
    if (confirm("Remove this search term from the intoxication pool?")) {
        window.location.href = "juanita://search-explorer-delete?term=" + encodeURIComponent(encodedTerm);
    }
}

function banNode(nodeId) {
    if (confirm("Outright ban contributor node " + nodeId + "? This will purge all terms from this node, blacklist the peer, and sever gossip channels.")) {
        window.location.href = "juanita://search-explorer-ban-node?node_id=" + encodeURIComponent(nodeId);
    }
}
