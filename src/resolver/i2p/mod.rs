use crate::resolver::DomainResolver;
use crate::util::config::AppConfig;
use std::net::{IpAddr, Ipv4Addr};

/// Sentinel IP used to signal "this is a .i2p address, route via I2P SOCKS5 proxy".
/// 127.0.0.3 is a loopback address that is otherwise unused in the resolution flow.
pub const I2P_SENTINEL_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 3);

/// Resolver for I2P `.i2p` eepsite services.
pub struct I2pResolver;

impl DomainResolver for I2pResolver {
    fn name(&self) -> &'static str {
        "I2P"
    }

    fn resolve(&self, domain: &str) -> Result<IpAddr, String> {
        if !domain.ends_with(".i2p") {
            return Err(format!("I2pResolver: '{}' is not a .i2p address", domain));
        }

        let config = AppConfig::load();
        if !config.i2p_enabled {
            return Err("I2P transport is disabled in configuration".to_string());
        }

        if !crate::i2p::is_i2p_running() {
            return Err(
                "I2P transport is not running — I2P router daemon is not active".to_string(),
            );
        }

        crate::log!(
            Info,
            RESOLVER,
            "I2pResolver: '{}' → sentinel {} (I2P SOCKS proxy will handle destination)",
            domain,
            I2P_SENTINEL_IP
        );
        Ok(IpAddr::V4(I2P_SENTINEL_IP))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i2p_resolver_rejects_non_i2p() {
        let resolver = I2pResolver;
        let result = resolver.resolve("example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a .i2p address"));
    }

    #[test]
    fn test_i2p_resolver_rejects_when_i2p_disabled() {
        let resolver = I2pResolver;
        let result = resolver.resolve("identiguy.i2p");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("disabled") || err.contains("not running"),
            "Unexpected error: {}",
            err
        );
    }
}
