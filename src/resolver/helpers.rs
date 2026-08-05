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

pub fn depunycode_host(host: &str) -> String {
    host.split('.')
        .map(|label| {
            if let Some(stripped) = label.strip_prefix("xn--") {
                if let Some(decoded_chars) = idna::punycode::decode(stripped) {
                    return decoded_chars.into_iter().collect::<String>();
                }
            }
            label.to_string()
        })
        .collect::<Vec<_>>()
        .join(".")
}

pub fn restore_original_domain_in_uri(uri: &str) -> String {
    let mut restored = uri.to_string();
    if let Ok(parsed) = url::Url::parse(uri) {
        if let Some(host_str) = parsed.host_str() {
            if let Ok(ip) = host_str.parse::<std::net::IpAddr>() {
                if let Some(domain) = crate::resolver::get_sentinel_domain(&ip) {
                    restored = uri.replace(host_str, &domain);
                    crate::log!(Trace, RESOLVER, "Restored {} to {}", uri, restored);
                } else {
                    crate::log!(Trace, RESOLVER, "No sentinel domain found for IP {}", ip);
                }
            }
        }
    }

    let host_part = crate::browsing::browser::extract_domain(&restored);
    if !host_part.is_empty() {
        let unicode_host = depunycode_host(&host_part);
        if unicode_host != host_part {
            restored = restored.replace(&host_part, &unicode_host);
        }
    }

    restored
}
