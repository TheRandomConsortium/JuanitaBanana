use regex::Regex;
use reqwest::blocking::Client;
use std::collections::HashSet;
use std::thread;

#[allow(dead_code)]
pub struct CrawlResult {
    pub domain: String,
    pub emails: Vec<String>,
}

pub fn search_ddg_domains(query: &str) -> Vec<String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0")
        .build()
        .unwrap_or_default();

    let url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        urlencoding::encode(query)
    );
    let mut domains = Vec::new();

    if let Ok(resp) = client.get(&url).send() {
        if let Ok(html) = resp.text() {
            // Find uddg redirection links
            let re = Regex::new(r"uddg=([^&]+)").unwrap();
            for cap in re.captures_iter(&html) {
                if let Some(encoded_url) = cap.get(1) {
                    if let Ok(decoded_url) = urlencoding::decode(encoded_url.as_str()) {
                        let domain = crate::browsing::browser::extract_domain(&decoded_url);
                        if !domain.is_empty()
                            && !domain.contains("duckduckgo.com")
                            && !domains.contains(&domain)
                        {
                            domains.push(domain);
                            if domains.len() >= 5 {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    domains
}

#[allow(dead_code)]
pub fn start_crawl_thread<F>(domain: String, callback: F)
where
    F: FnOnce(CrawlResult) + Send + 'static,
{
    thread::spawn(move || {
        let result = crawl_domain(domain);
        let mut callback_opt = Some(callback);
        let mut result_opt = Some(result);
        gtk::glib::idle_add_local(move || {
            if let (Some(cb), Some(res)) = (callback_opt.take(), result_opt.take()) {
                cb(res);
            }
            gtk::glib::ControlFlow::Break
        });
    });
}

pub fn crawl_domain(domain: String) -> CrawlResult {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0")
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .unwrap_or_default();

    let base_url = if domain.starts_with("http://") || domain.starts_with("https://") {
        domain.clone()
    } else {
        format!("https://{}", domain)
    };

    let mut emails = HashSet::new();
    let mut visited_links = HashSet::new();
    visited_links.insert(base_url.clone());

    // Fetch homepage
    if let Ok(resp) = client.get(&base_url).send() {
        if let Ok(html) = resp.text() {
            extract_emails_from_text(&html, &mut emails);

            // Extract subpages to crawl (privacy, contact, etc.)
            let subpage_links = extract_subpage_links(&base_url, &html);
            for link in subpage_links {
                if visited_links.len() >= 6 {
                    break;
                }
                if visited_links.insert(link.clone()) {
                    if let Ok(sub_resp) = client.get(&link).send() {
                        if let Ok(sub_html) = sub_resp.text() {
                            extract_emails_from_text(&sub_html, &mut emails);
                        }
                    }
                }
            }
        }
    }

    let mut emails_list: Vec<String> = emails.into_iter().collect();
    emails_list.sort();

    CrawlResult {
        domain,
        emails: emails_list,
    }
}

fn extract_emails_from_text(text: &str, emails: &mut HashSet<String>) {
    let re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            let email = m.as_str().to_lowercase();
            // Filter out common image/binary extension noise inside text
            if !email.ends_with(".png")
                && !email.ends_with(".jpg")
                && !email.ends_with(".gif")
                && !email.ends_with(".svg")
            {
                emails.insert(email);
            }
        }
    }
}

fn extract_subpage_links(base_url: &str, html: &str) -> Vec<String> {
    let mut links = Vec::new();
    let re = Regex::new(r#"href=["']([^"']+)["']"#).unwrap();

    let keywords = [
        "privacy", "contact", "legal", "about", "policy", "terms", "aviso", "contacto", "politica",
    ];

    for cap in re.captures_iter(html) {
        if let Some(href) = cap.get(1) {
            let href_str = href.as_str();
            let matches_keyword = keywords.iter().any(|k| href_str.to_lowercase().contains(k));

            if matches_keyword {
                let absolute_url =
                    if href_str.starts_with("http://") || href_str.starts_with("https://") {
                        href_str.to_string()
                    } else if href_str.starts_with('/') {
                        // Combine with domain base
                        let mut parts = base_url.split("://");
                        let proto = parts.next().unwrap_or("https");
                        let rest = parts.next().unwrap_or(base_url);
                        let domain_only = rest.split('/').next().unwrap_or(rest);
                        format!("{}://{}{}", proto, domain_only, href_str)
                    } else {
                        format!("{}/{}", base_url.trim_end_matches('/'), href_str)
                    };

                if !links.contains(&absolute_url) {
                    links.push(absolute_url);
                }
            }
        }
    }

    links
}
