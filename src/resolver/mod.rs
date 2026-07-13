use crate::util::config::AppConfig;
use lazy_static::lazy_static;
use std::net::{IpAddr, UdpSocket};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;

lazy_static! {
    static ref HNSD_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
}

/// A generic trait representing a domain name resolver.
/// This allows implementing different DNS backends and reordering them dynamically.
pub trait DomainResolver {
    fn resolve(&self, domain: &str) -> Result<IpAddr, String>;
    fn name(&self) -> &'static str;
}

/// Locates the hnsd binary in standard locations.
fn find_hnsd_path() -> Option<PathBuf> {
    // 1. Check relative to current working directory: bin/hnsd
    let local = PathBuf::from("bin/hnsd");
    if local.exists() {
        return Some(local);
    }
    // 2. Check next to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let next_to_exe = parent.join("hnsd");
            if next_to_exe.exists() {
                return Some(next_to_exe);
            }
            let next_to_exe_bin = parent.join("bin").join("hnsd");
            if next_to_exe_bin.exists() {
                return Some(next_to_exe_bin);
            }
        }
    }
    // 3. Fallback to /usr/bin/hnsd
    let usr_bin = PathBuf::from("/usr/bin/hnsd");
    if usr_bin.exists() {
        return Some(usr_bin);
    }
    None
}

fn base_data_dir() -> PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
        });
    let path = base.join("juanita-banana");
    std::fs::create_dir_all(&path).ok();
    path
}

/// Initializes the resolver system. Starts the hnsd daemon if Handshake is enabled.
pub fn init_resolver() {
    let config = AppConfig::load();
    if !config.resolver_order.contains(&"Handshake".to_string()) {
        crate::log!(
            Info,
            RESOLVER,
            "Handshake resolver not in resolver_order, skipping hnsd daemon startup"
        );
        return;
    }

    let hnsd_bin = match find_hnsd_path() {
        Some(path) => path,
        None => {
            crate::log!(
                Info,
                RESOLVER,
                "hnsd binary not found, Handshake resolver will not function"
            );
            return;
        }
    };

    let state_dir = base_data_dir().join("hnsd_state");
    std::fs::create_dir_all(&state_dir).ok();

    crate::log!(
        Info,
        RESOLVER,
        "Starting hnsd daemon from {:?}",
        hnsd_bin.to_string_lossy()
    );

    match Command::new(&hnsd_bin)
        .arg("-r")
        .arg("127.0.0.1:5349")
        .arg("-p")
        .arg("8")
        .arg("-x")
        .arg(&state_dir)
        .spawn()
    {
        Ok(child) => {
            let mut lock = HNSD_PROCESS.lock().unwrap();
            *lock = Some(child);
            crate::log!(Info, RESOLVER, "hnsd daemon started on 127.0.0.1:5349");
        }
        Err(e) => {
            crate::log!(Info, RESOLVER, "Failed to start hnsd daemon: {}", e);
        }
    }
}

/// Shuts down the local hnsd daemon if it was started by us.
pub fn shutdown_resolver() {
    let mut lock = HNSD_PROCESS.lock().unwrap();
    if let Some(mut child) = lock.take() {
        crate::log!(Info, RESOLVER, "Terminating hnsd daemon...");
        let _ = child.kill();
        let _ = child.wait();
        crate::log!(Info, RESOLVER, "hnsd daemon terminated");
    }
}

/// Performs a raw DNS query over UDP.
pub fn resolve_dns_udp(domain: &str, dns_server: &str) -> Result<Vec<IpAddr>, String> {
    let socket =
        UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("Failed to bind UDP socket: {}", e))?;
    socket.set_read_timeout(Some(Duration::from_secs(3))).ok();

    // DNS Header
    let mut packet = Vec::with_capacity(512);
    packet.extend_from_slice(&[0x12, 0x34]); // Transaction ID
    packet.extend_from_slice(&[0x01, 0x00]); // Standard query, recursion desired
    packet.extend_from_slice(&[0x00, 0x01]); // 1 Question
    packet.extend_from_slice(&[0x00, 0x00]); // 0 Answers
    packet.extend_from_slice(&[0x00, 0x00]); // 0 Authority
    packet.extend_from_slice(&[0x00, 0x00]); // 0 Additional

    // QNAME
    for part in domain.split('.') {
        if part.is_empty() {
            continue;
        }
        if part.len() > 63 {
            return Err("DNS label too long".to_string());
        }
        packet.push(part.len() as u8);
        packet.extend_from_slice(part.as_bytes());
    }
    packet.push(0x00); // Terminating null byte

    packet.extend_from_slice(&[0x00, 0x01]); // QTYPE: A (1)
    packet.extend_from_slice(&[0x00, 0x01]); // QCLASS: IN (1)

    socket
        .send_to(&packet, dns_server)
        .map_err(|e| format!("Failed to send DNS request: {}", e))?;

    let mut buf = [0; 512];
    let (amt, _) = socket
        .recv_from(&mut buf)
        .map_err(|e| format!("Failed to receive DNS response: {}", e))?;

    if amt < 12 {
        return Err("DNS response too short".to_string());
    }

    if buf[0] != 0x12 || buf[1] != 0x34 {
        return Err("DNS Transaction ID mismatch".to_string());
    }

    let rcode = buf[3] & 0x0F;
    if rcode != 0 {
        return Err(format!("DNS Server returned error code: {}", rcode));
    }

    let q_count = ((buf[4] as u16) << 8) | (buf[5] as u16);
    let ans_count = ((buf[6] as u16) << 8) | (buf[7] as u16);

    if ans_count == 0 {
        return Ok(Vec::new());
    }

    let mut pos = 12;
    for _ in 0..q_count {
        if pos >= amt {
            return Err("Truncated DNS question section".to_string());
        }
        // Skip domain labels
        while pos < amt {
            let len = buf[pos] as usize;
            if len == 0 {
                pos += 1;
                break;
            }
            if (len & 0xC0) == 0xC0 {
                pos += 2;
                break;
            }
            pos += 1 + len;
        }
        pos += 4; // Skip QTYPE and QCLASS
    }

    let mut ips = Vec::new();
    for _ in 0..ans_count {
        if pos >= amt {
            break;
        }
        // Skip NAME
        while pos < amt {
            let len = buf[pos] as usize;
            if len == 0 {
                pos += 1;
                break;
            }
            if (len & 0xC0) == 0xC0 {
                pos += 2;
                break;
            }
            pos += 1 + len;
        }
        if pos + 10 > amt {
            break;
        }
        let atype = ((buf[pos] as u16) << 8) | (buf[pos + 1] as u16);
        let rdlength = ((buf[pos + 8] as usize) << 8) | (buf[pos + 9] as usize);
        pos += 10;

        if atype == 1 && rdlength == 4 && pos + 4 <= amt {
            let ip = IpAddr::V4(std::net::Ipv4Addr::new(
                buf[pos],
                buf[pos + 1],
                buf[pos + 2],
                buf[pos + 3],
            ));
            ips.push(ip);
        }
        pos += rdlength;
    }

    Ok(ips)
}

/// Resolver implementation for Handshake (using local hnsd recursive server).
pub struct HandshakeResolver {
    port: u16,
}

impl HandshakeResolver {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

impl DomainResolver for HandshakeResolver {
    fn name(&self) -> &'static str {
        "Handshake"
    }

    fn resolve(&self, domain: &str) -> Result<IpAddr, String> {
        let dns_server = format!("127.0.0.1:{}", self.port);
        match resolve_dns_udp(domain, &dns_server) {
            Ok(ips) => {
                if let Some(ip) = ips.into_iter().next() {
                    Ok(ip)
                } else {
                    Err("No IP addresses returned by Handshake resolver".to_string())
                }
            }
            Err(e) => Err(e),
        }
    }
}

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

/// Resolves a domain name using the dynamic resolver priority chain configured by the user.
pub fn resolve_domain_with_chain(domain: &str) -> Result<IpAddr, String> {
    if let Ok(ip) = domain.parse::<IpAddr>() {
        return Ok(ip);
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
                resolvers.push(Box::new(HandshakeResolver::new(5349)));
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
                return Ok(ip);
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
}
