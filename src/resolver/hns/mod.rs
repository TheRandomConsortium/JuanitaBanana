use super::DomainResolver;
use crate::util::config::AppConfig;
use std::net::{IpAddr, UdpSocket};
use std::time::Duration;

pub mod daemon;

/// Performs a raw DNS query over UDP.
pub fn resolve_dns_udp(domain: &str, dns_server: &str) -> Result<Vec<IpAddr>, String> {
    let socket =
        UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("Failed to bind UDP socket: {}", e))?;
    socket.set_read_timeout(Some(Duration::from_secs(5))).ok();

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

    let mut attempts = 0;
    let mut buf = [0; 512];
    let amt;

    loop {
        attempts += 1;
        socket
            .send_to(&packet, dns_server)
            .map_err(|e| format!("Failed to send DNS request: {}", e))?;

        match socket.recv_from(&mut buf) {
            Ok((size, _)) => {
                amt = size;
                break;
            }
            Err(e) => {
                if (e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut)
                    && attempts < 3
                {
                    crate::log!(
                        Debug,
                        RESOLVER,
                        "DNS query for {} timed out, retrying (attempt {}/3)...",
                        domain,
                        attempts
                    );
                    continue;
                }
                return Err(format!("Failed to receive DNS response: {}", e));
            }
        }
    }

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
        let config = AppConfig::load();
        if !config.handshake_enabled {
            return Err("Handshake resolution is disabled in configuration".to_string());
        }
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
