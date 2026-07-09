use super::{InternalPage, PageContext};
use webkit2gtk::WebViewExt;

pub struct UnbanPage;

impl InternalPage for UnbanPage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:unban")
            || input.starts_with("juanita://unban")
            || input.starts_with("juanita://submit-unban")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        (uri.starts_with("juanita://unban") && !uri.starts_with("juanita://unban-page"))
            || uri.starts_with("juanita://submit-unban")
            || (uri.starts_with("juanita://banned") && !uri.starts_with("juanita://banned-page"))
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool {
        if let Some(domain_query) = uri.strip_prefix("juanita://unban?domain=") {
            use crate::util::ban::{BasicIntegralEquationProvider, EquationProvider};
            let provider = BasicIntegralEquationProvider;
            let (equation, answer) = provider.generate_challenge();

            let domain = domain_query.to_string();
            *ctx.expected_unban.borrow_mut() = Some((domain.clone(), answer));

            let unban_html = crate::util::ban::unban_page(&domain, &equation);
            let base_uri = uri.replace("juanita://unban", "juanita://unban-page");
            ctx.webview.load_html(&unban_html, Some(&base_uri));
            return true;
        }

        if uri.starts_with("juanita://unban") {
            let domains = ctx.banlist.borrow().banned_domains.clone();
            let list_html = crate::util::ban::unban_list_page(&domains);
            ctx.webview
                .load_html(&list_html, Some("juanita://unban-page/"));
            return true;
        }

        if let Some(query) = uri.strip_prefix("juanita://submit-unban?") {
            let parts: Vec<&str> = query.split('&').collect();
            let mut domain = String::new();
            let mut answer = String::new();
            for p in parts {
                if let Some(d) = p.strip_prefix("domain=") {
                    domain = d.to_string();
                }
                if let Some(a) = p.strip_prefix("answer=") {
                    answer = a.to_string();
                }
            }

            if let Some((expected_domain, expected_ans)) = ctx.expected_unban.borrow().as_ref() {
                if *expected_domain == domain && answer == expected_ans.to_string() {
                    println!("[UNBAN] User solved the math! Unbanning {}", domain);
                    let mut bl = ctx.banlist.borrow_mut();
                    bl.unban(&domain);
                    bl.save();
                    ctx.webview.load_uri(&format!("https://{}", domain));
                    return true;
                }
            }

            println!("[UNBAN] Incorrect math or tampered domain. Access denied.");
            let banned_html = crate::util::ban::banned_page(&domain);
            ctx.webview
                .load_html(&banned_html, Some("juanita://banned"));
            return true;
        }

        if uri.starts_with("juanita://banned") {
            let domain = if let Some(current_uri) = ctx.webview.uri() {
                current_uri.to_string()
            } else {
                "unknown".to_string()
            };
            let banned_html = crate::util::ban::banned_page(&domain);
            ctx.webview
                .load_html(&banned_html, Some("juanita://banned"));
            return true;
        }

        false
    }
}
