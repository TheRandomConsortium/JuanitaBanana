use crate::util::config::AppConfig;
use std::net::IpAddr;

pub mod helpers;
pub mod hns;
pub mod system;

// Re-export helper functions and cached state
pub use helpers::{
    clean_host, register_resolved_ip, restore_original_domain_in_uri, rewrite_uri_host,
};
pub use hns::daemon::{init_resolver, shutdown_resolver};
pub use hns::HandshakeResolver;
pub use system::SystemResolver;

/// A generic trait representing a domain name resolver.
/// This allows implementing different DNS backends and reordering them dynamically.
pub trait DomainResolver {
    fn resolve(&self, domain: &str) -> Result<IpAddr, String>;
    fn name(&self) -> &'static str;
}

/// Resolves a domain name using the dynamic resolver priority chain configured by the user.
pub fn resolve_domain_with_chain(domain: &str) -> Result<(IpAddr, String), String> {
    if let Ok(ip) = domain.parse::<IpAddr>() {
        return Ok((ip, "System".to_string()));
    }

    let config = AppConfig::load();
    let order = if config.resolver_order.is_empty() {
        vec!["Handshake".to_string(), "System".to_string()]
    } else {
        config.resolver_order.clone()
    };

    let mut resolvers: Vec<Box<dyn DomainResolver>> = Vec::new();
    for name in order {
        match name.as_str() {
            "Handshake" => {
                resolvers.push(Box::new(HandshakeResolver::new(5350)));
            }
            "System" => {
                resolvers.push(Box::new(SystemResolver));
            }
            _ => {
                // Ignore other resolvers for now
            }
        }
    }

    let mut errors = Vec::new();
    for r in resolvers {
        match r.resolve(domain) {
            Ok(ip) => {
                crate::log!(
                    Info,
                    RESOLVER,
                    "Resolved {} -> {} using {}",
                    domain,
                    ip,
                    r.name()
                );
                return Ok((ip, r.name().to_string()));
            }
            Err(e) => {
                crate::log!(
                    Debug,
                    RESOLVER,
                    "{} failed to resolve {}: {}",
                    r.name(),
                    domain,
                    e
                );
                errors.push(format!("{}: {}", r.name(), e));
            }
        }
    }

    Err(format!(
        "Resolution failed for {}: {}",
        domain,
        errors.join("; ")
    ))
}

pub fn is_system_resolvable(domain: &str) -> bool {
    SystemResolver.resolve(domain).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_system() {
        let resolver = SystemResolver;
        let ip = resolver.resolve("localhost");
        assert!(ip.is_ok());
    }

    #[test]
    fn test_resolve_domain_with_chain() {
        let ip = resolve_domain_with_chain("localhost");
        assert!(ip.is_ok());
    }

    #[test]
    fn test_handshake_disabled_resolver() {
        let resolver = HandshakeResolver::new(5350);
        // Save temporary config with handshake disabled
        let mut config = AppConfig::load();
        let original_val = config.handshake_enabled;

        config.handshake_enabled = false;
        config.save();

        let res = resolver.resolve("localhost");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "Handshake resolution is disabled in configuration"
        );

        // Restore original config
        config.handshake_enabled = original_val;
        config.save();
    }

    #[test]
    fn test_is_system_resolvable() {
        assert!(is_system_resolvable("localhost"));
        assert!(!is_system_resolvable("nonexistentdomain.woodburn"));
    }

    #[test]
    fn test_clean_host() {
        assert_eq!(clean_host("103.152.197.116"), "103.152.197.116");
        assert_eq!(clean_host("103.152.197.116:8080"), "103.152.197.116");
        assert_eq!(clean_host("[2001:db8::1]"), "2001:db8::1");
        assert_eq!(clean_host("[2001:db8::1]:8080"), "2001:db8::1");
        assert_eq!(
            clean_host("user:pass@103.152.197.116:8080"),
            "103.152.197.116"
        );
        assert_eq!(clean_host("nathan.woodburn:8080"), "nathan.woodburn");
    }

    #[test]
    fn test_rewrite_uri_host() {
        assert_eq!(
            rewrite_uri_host(
                "http://nathan.woodburn/path/to/nathan.woodburn.html",
                "nathan.woodburn",
                "103.152.197.116"
            ),
            "http://103.152.197.116/path/to/nathan.woodburn.html"
        );
        assert_eq!(
            rewrite_uri_host(
                "http://nathan.woodburn:8080/path",
                "nathan.woodburn",
                "103.152.197.116"
            ),
            "http://103.152.197.116:8080/path"
        );
    }
}
