use super::DomainResolver;
use std::net::IpAddr;

/// Resolver implementation for System DNS (ICANN).
pub struct SystemResolver;

impl DomainResolver for SystemResolver {
    fn name(&self) -> &'static str {
        "System"
    }

    fn resolve(&self, domain: &str) -> Result<IpAddr, String> {
        use std::net::ToSocketAddrs;
        let host_port = format!("{}:80", domain);
        match host_port.to_socket_addrs() {
            Ok(addrs) => {
                if let Some(addr) = addrs.map(|a| a.ip()).next() {
                    Ok(addr)
                } else {
                    Err("No IP addresses returned by System resolver".to_string())
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
