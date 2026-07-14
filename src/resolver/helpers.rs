use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref RESOLVED_IPS: Mutex<std::collections::HashMap<String, String>> =
        Mutex::new(std::collections::HashMap::new());
}

pub fn register_resolved_ip(ip: String, domain: String) {
    if let Ok(mut lock) = RESOLVED_IPS.lock() {
        lock.insert(ip, domain);
    }
}

pub fn get_original_domain(ip: &str) -> Option<String> {
    RESOLVED_IPS
        .lock()
        .ok()
        .and_then(|lock| lock.get(ip).cloned())
}

pub fn clean_host(domain: &str) -> String {
    // 1. Remove userinfo if present (e.g. "user:pass@host")
    let mut host = domain.split('@').next_back().unwrap_or(domain).to_string();

    // 2. Remove port if present (e.g. "host:8080" or "[ipv6]:8080")
    if host.starts_with('[') {
        // IPv6 with brackets, port could be after the closing bracket
        if let Some(closing_idx) = host.find(']') {
            host = host[1..closing_idx].to_string();
        }
    } else {
        // IPv4 or hostname, port is after the last ':' (or the only ':')
        if let Some(colon_idx) = host.rfind(':') {
            // Check if it's not a raw IPv6 without brackets (which would have multiple colons)
            if host.chars().filter(|&c| c == ':').count() == 1 {
                host = host[..colon_idx].to_string();
            }
        }
    }

    host
}

pub fn rewrite_uri_host(uri: &str, old_host: &str, new_host: &str) -> String {
    if let Some(pos) = uri.find("://") {
        let scheme_part = &uri[..pos + 3];
        let rest = &uri[pos + 3..];
        let slash_pos = rest.find('/').unwrap_or(rest.len());
        let authority = &rest[..slash_pos];
        let path_part = &rest[slash_pos..];

        let new_authority = authority.replace(old_host, new_host);
        format!("{}{}{}", scheme_part, new_authority, path_part)
    } else {
        uri.replace(old_host, new_host)
    }
}

pub fn restore_original_domain_in_uri(uri: &str) -> String {
    let domain = crate::browsing::browser::extract_domain(uri);
    let host = clean_host(&domain);
    if !host.is_empty() {
        if let Some(orig) = get_original_domain(&host) {
            return rewrite_uri_host(uri, &host, &orig);
        }
    }
    uri.to_string()
}
