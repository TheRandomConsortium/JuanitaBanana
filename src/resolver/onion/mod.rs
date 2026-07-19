use crate::resolver::DomainResolver;
use crate::util::config::AppConfig;
use std::net::{IpAddr, Ipv4Addr};

/// Sentinel IP used to signal "this is a .onion address, route via Tor SOCKS5 proxy".
/// 127.0.0.2 is a loopback address that is otherwise unused in the resolution flow.
pub const ONION_SENTINEL_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 2);

/// Resolver for Tor `.onion` v3 hidden services.
///
/// `.onion` addresses are **self-authenticating** — the 56-character base32 string
/// encodes the Ed25519 public key of the hidden service. There is no DNS lookup to
/// perform; the address is passed directly to the Tor SOCKS5 proxy (arti) which
/// handles the rendezvous internally.
///
/// This resolver returns a sentinel IP (`127.0.0.2`) so that `policy.rs` can detect
/// that the domain resolved via the Onion resolver and route the WebKit navigation
/// through the SOCKS5 proxy rather than rewriting the URI to a bare IP.
pub struct OnionResolver;

impl DomainResolver for OnionResolver {
    fn name(&self) -> &'static str {
        "Onion"
    }

    fn resolve(&self, domain: &str) -> Result<IpAddr, String> {
        // Only handle .onion addresses — fast path rejection for everything else.
        // This check comes first so non-.onion domains always get a clear error
        // regardless of whether Tor is enabled.
        if !domain.ends_with(".onion") {
            return Err(format!(
                "OnionResolver: '{}' is not a .onion address",
                domain
            ));
        }

        let config = AppConfig::load();
        if !config.tor_enabled {
            return Err("Tor transport is disabled in configuration".to_string());
        }

        // Tor must be actively running (arti daemon alive)
        if !crate::tor::is_tor_running() {
            return Err("Tor transport is not running — arti daemon is not active".to_string());
        }

        // No DNS lookup: return sentinel so policy.rs routes via SOCKS5
        crate::log!(
            Info,
            RESOLVER,
            "OnionResolver: '{}' → sentinel {} (arti SOCKS5 will handle rendezvous)",
            domain,
            ONION_SENTINEL_IP
        );
        Ok(IpAddr::V4(ONION_SENTINEL_IP))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onion_resolver_rejects_non_onion() {
        let resolver = OnionResolver;
        let result = resolver.resolve("example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a .onion address"));
    }

    #[test]
    fn test_onion_resolver_rejects_when_tor_disabled() {
        // Tor is disabled by default in test config — should get config error
        let resolver = OnionResolver;
        let result =
            resolver.resolve("facebookwkhpilnemxj7ascrwvxw3cn5vpvngh4jhzs4ml4i6bxm7xoe2ad.onion");
        assert!(result.is_err());
        // Either "disabled" or "not running" is acceptable depending on config state
        let err = result.unwrap_err();
        assert!(
            err.contains("disabled") || err.contains("not running"),
            "Unexpected error: {}",
            err
        );
    }
}
