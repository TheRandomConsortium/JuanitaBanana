use crate::browsing::internal::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct SearchExplorerPage;

impl InternalPage for SearchExplorerPage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:search-explorer")
            || input.starts_with("juanita://search-explorer")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        (uri.starts_with("juanita://search-explorer") && !uri.starts_with("juanita://search-explorer-page"))
            || uri.starts_with("juanita://search-explorer-delete")
            || uri.starts_with("juanita://search-explorer-ban-node")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool {
        if let Some(query_part) = uri.strip_prefix("juanita://search-explorer-delete?term=") {
            let term = urlencoding::decode(query_part)
                .unwrap_or_default()
                .to_string();
            if let Ok(mut pool) = crate::privacy::search::gossip::GLOBAL_NOISE_POOL.lock() {
                pool.remove_term(&term);
            }
            ctx.webview.load_uri("juanita://search-explorer");
            return true;
        }

        if let Some(query_part) = uri.strip_prefix("juanita://search-explorer-ban-node?node_id=") {
            let node_id = urlencoding::decode(query_part)
                .unwrap_or_default()
                .to_string();
            crate::browsing::ban_peer(&node_id);
            if let Ok(mut pool) = crate::privacy::search::gossip::GLOBAL_NOISE_POOL.lock() {
                pool.remove_by_node(&node_id);
            }
            if let Ok(mut pb) = crate::privacy::search::gossip::GLOBAL_PHONEBOOK.lock() {
                pb.remove_peer(&node_id);
            }
            ctx.webview.load_uri("juanita://search-explorer");
            return true;
        }

        let html_template = include_root_str!(@templates, "search_explorer.html");
        let css = crate::browsing::internal::SHARED_CSS.as_str();
        let js = include_root_str!(@scripts, "search_explorer.js");

        let mut terms = Vec::new();
        if let Ok(mut pool_lock) = crate::privacy::search::gossip::GLOBAL_NOISE_POOL.lock() {
            terms = pool_lock.get_all_terms();
        }
        let total_terms_count = terms.len();

        let mut rows_html = String::new();
        if terms.is_empty() {
            rows_html.push_str(
                r#"<tr><td colspan="5" style="text-align: center; padding: 24px; color: var(--text-secondary, #888);">No search terms currently stored in local intoxication pool.</td></tr>"#
            );
        } else {
            for entry in terms {
                let term_escaped = html_escape(&entry.term);
                let origin_escaped = html_escape(&entry.origin);
                let ingested_date = format_epoch(entry.ingested_epoch);
                let expires_date = format_epoch(entry.expires_epoch);

                rows_html.push_str(&format!(
                    r#"<tr data-term="{}" data-origin="{}" data-date-epoch="{}" style="border-bottom: 1px solid var(--border-color, #222);">
                        <td style="padding: 10px; font-weight: bold;">{}</td>
                        <td style="padding: 10px;"><code>{}</code></td>
                        <td style="padding: 10px;">{}</td>
                        <td style="padding: 10px;">{}</td>
                        <td style="padding: 10px; text-align: right;">
                            <button onclick="deleteTerm('{}')" class="jb-button jb-button-small" style="margin-right: 6px;">Delete</button>
                            <button onclick="banNode('{}')" class="jb-button jb-button-danger jb-button-small">Ban Node</button>
                        </td>
                    </tr>"#,
                    term_escaped, origin_escaped, entry.ingested_epoch,
                    term_escaped, origin_escaped, ingested_date, expires_date,
                    urlencoding::encode(&entry.term), origin_escaped
                ));
            }
        }

        let rendered = html_template
            .replace("{css}", css)
            .replace("{js}", js)
            .replace("{total_terms_count}", &total_terms_count.to_string())
            .replace("{explorer_rows_html}", &rows_html);

        ctx.webview
            .load_html(&rendered, Some("juanita://search-explorer-page/"));
        true
    }
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn format_epoch(epoch_secs: u64) -> String {
    if epoch_secs == 0 {
        return "N/A".to_string();
    }
    format!("Epoch {}", epoch_secs)
}
