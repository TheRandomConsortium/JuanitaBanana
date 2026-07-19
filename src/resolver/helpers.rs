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

pub fn restore_original_domain_in_uri(uri: &str) -> String {
    uri.to_string()
}
