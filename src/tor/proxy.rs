use std::net::{TcpListener, TcpStream, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use crate::util::config::AppConfig;
use crate::log;

pub const LOCAL_PROXY_PORT: u16 = 9151;

/// Starts the local SOCKS5 proxy server in a background thread.
pub fn start_local_proxy() {
    thread::spawn(move || {
        let listener = match TcpListener::bind(("127.0.0.1", LOCAL_PROXY_PORT)) {
            Ok(l) => l,
            Err(e) => {
                log!(Error, TOR, "Failed to bind local SOCKS5 proxy to port {}: {}", LOCAL_PROXY_PORT, e);
                return;
            }
        };

        log!(Info, TOR, "Local SOCKS5 proxy listening on 127.0.0.1:{}", LOCAL_PROXY_PORT);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream) {
                            log!(Debug, TOR, "Local proxy connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    log!(Error, TOR, "Error accepting connection in local SOCKS5 proxy: {}", e);
                }
            }
        }
    });
}

fn handle_connection(mut client: TcpStream) -> Result<(), String> {
    client.set_read_timeout(Some(Duration::from_secs(30))).ok();
    client.set_write_timeout(Some(Duration::from_secs(30))).ok();

    // 1. Handshake Greeting
    let mut greeting = [0u8; 2];
    client.read_exact(&mut greeting).map_err(|e| format!("Failed to read SOCKS5 greeting: {}", e))?;

    if greeting[0] != 0x05 {
        return Err(format!("Unsupported SOCKS version: {}", greeting[0]));
    }

    let num_methods = greeting[1] as usize;
    let mut methods = vec![0u8; num_methods];
    client.read_exact(&mut methods).map_err(|e| format!("Failed to read SOCKS5 methods: {}", e))?;

    // Respond with No Authentication required (0x00)
    client.write_all(&[0x05, 0x00]).map_err(|e| format!("Failed to write SOCKS5 greeting response: {}", e))?;

    // 2. Read Request
    let mut request_header = [0u8; 4];
    client.read_exact(&mut request_header).map_err(|e| format!("Failed to read SOCKS5 request header: {}", e))?;

    if request_header[0] != 0x05 {
        return Err(format!("Invalid SOCKS request version: {}", request_header[0]));
    }

    let cmd = request_header[1];
    if cmd != 0x01 {
        // Command not supported
        client.write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).ok();
        return Err(format!("Unsupported SOCKS command: {}", cmd));
    }

    let atyp = request_header[3];
    let (dest_host, dest_port) = match atyp {
        0x01 => {
            // IPv4 Address
            let mut ipv4 = [0u8; 4];
            client.read_exact(&mut ipv4).map_err(|e| format!("Failed to read IPv4 address: {}", e))?;
            let mut port_bytes = [0u8; 2];
            client.read_exact(&mut port_bytes).map_err(|e| format!("Failed to read port: {}", e))?;
            let port = u16::from_be_bytes(port_bytes);
            let ip = IpAddr::V4(Ipv4Addr::new(ipv4[0], ipv4[1], ipv4[2], ipv4[3]));
            (DestHost::Ip(ip), port)
        }
        0x03 => {
            // Domain Name / Hostname string
            let mut len_buf = [0u8; 1];
            client.read_exact(&mut len_buf).map_err(|e| format!("Failed to read domain length: {}", e))?;
            let len = len_buf[0] as usize;
            let mut domain_bytes = vec![0u8; len];
            client.read_exact(&mut domain_bytes).map_err(|e| format!("Failed to read domain name: {}", e))?;
            let mut port_bytes = [0u8; 2];
            client.read_exact(&mut port_bytes).map_err(|e| format!("Failed to read port: {}", e))?;
            let port = u16::from_be_bytes(port_bytes);
            let domain = String::from_utf8(domain_bytes).map_err(|_| "Invalid UTF-8 in domain name".to_string())?;
            
            // Try to parse string as an IP address
            if let Ok(ip) = domain.parse::<IpAddr>() {
                (DestHost::Ip(ip), port)
            } else {
                (DestHost::Domain(domain), port)
            }
        }
        0x04 => {
            // IPv6 Address
            let mut ipv6 = [0u8; 16];
            client.read_exact(&mut ipv6).map_err(|e| format!("Failed to read IPv6 address: {}", e))?;
            let mut port_bytes = [0u8; 2];
            client.read_exact(&mut port_bytes).map_err(|e| format!("Failed to read port: {}", e))?;
            let port = u16::from_be_bytes(port_bytes);
            let ip = IpAddr::V6(Ipv6Addr::from(ipv6));
            (DestHost::Ip(ip), port)
        }
        _ => {
            // Address type not supported
            client.write_all(&[0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).ok();
            return Err(format!("Unsupported SOCKS address type: {}", atyp));
        }
    };

    // 3. Resolve if Hostname
    let target_ip = match dest_host {
        DestHost::Ip(ip) => Some(ip),
        DestHost::Domain(ref domain) => {
            if domain.ends_with(".onion") {
                // Do not resolve .onion domains locally; let Tor handle them
                None
            } else {
                log!(Debug, TOR, "Local SOCKS5 proxy resolving '{}'...", domain);
                match crate::resolver::resolve_domain_with_chain(domain) {
                    Ok((resolved_ip, _)) => {
                        log!(Debug, TOR, "Local SOCKS5 proxy resolved '{}' -> {}", domain, resolved_ip);
                        Some(resolved_ip)
                    }
                    Err(e) => {
                        log!(Error, TOR, "Local SOCKS5 proxy failed to resolve '{}': {}", domain, e);
                        // Host unreachable
                        client.write_all(&[0x05, 0x04, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).ok();
                        return Err(format!("Host unreachable: {}", e));
                    }
                }
            }
        }
    };

    // 4. Establish Outbound Connection
    let config = AppConfig::load();
    let use_tor = if config.tor_enabled {
        if config.tor_route_all {
            true
        } else {
            match &dest_host {
                DestHost::Domain(domain) => domain.ends_with(".onion"),
                DestHost::Ip(_) => false,
            }
        }
    } else {
        false
    };

    let mut outbound = if use_tor {
        // Connect via Tor SOCKS5 proxy
        let tor_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), crate::tor::ARTI_SOCKS_PORT);
        let mut tor_stream = TcpStream::connect(tor_addr)
            .map_err(|e| format!("Failed to connect to Tor SOCKS5 proxy: {}", e))?;
        
        tor_stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
        tor_stream.set_write_timeout(Some(Duration::from_secs(30))).ok();

        // Handshake with Tor SOCKS5
        tor_stream.write_all(&[0x05, 0x01, 0x00]).map_err(|e| format!("Tor SOCKS5 greeting failed: {}", e))?;
        let mut tor_greeting = [0u8; 2];
        tor_stream.read_exact(&mut tor_greeting).map_err(|e| format!("Tor SOCKS5 greeting read failed: {}", e))?;
        if tor_greeting[0] != 0x05 || tor_greeting[1] != 0x00 {
            return Err("Tor SOCKS5 proxy rejected handshake".to_string());
        }

        // Send Tor CONNECT request
        let mut connect_req = Vec::new();
        connect_req.extend_from_slice(&[0x05, 0x01, 0x00]); // VER, CMD, RSV
        match target_ip {
            Some(IpAddr::V4(ipv4)) => {
                connect_req.push(0x01); // ATYP: IPv4
                connect_req.extend_from_slice(&ipv4.octets());
            }
            Some(IpAddr::V6(ipv6)) => {
                connect_req.push(0x04); // ATYP: IPv6
                connect_req.extend_from_slice(&ipv6.octets());
            }
            None => {
                if let DestHost::Domain(ref domain) = dest_host {
                    connect_req.push(0x03); // ATYP: Domain
                    connect_req.push(domain.len() as u8);
                    connect_req.extend_from_slice(domain.as_bytes());
                } else {
                    return Err("Missing target address for Tor proxy".to_string());
                }
            }
        }
        connect_req.extend_from_slice(&dest_port.to_be_bytes());
        tor_stream.write_all(&connect_req).map_err(|e| format!("Failed to write CONNECT request to Tor: {}", e))?;

        // Read Tor CONNECT response
        let mut tor_resp = [0u8; 4];
        tor_stream.read_exact(&mut tor_resp).map_err(|e| format!("Failed to read CONNECT response from Tor: {}", e))?;
        if tor_resp[0] != 0x05 || tor_resp[1] != 0x00 {
            // Tor failed to connect
            client.write_all(&[0x05, tor_resp[1], 0x00, 0x01, 0, 0, 0, 0, 0, 0]).ok();
            return Err(format!("Tor SOCKS5 proxy failed to connect, status: {}", tor_resp[1]));
        }

        // Skip remainder of response address fields (6 bytes for IPv4/port)
        let atyp_resp = tor_resp[3];
        let skip_len = match atyp_resp {
            0x01 => 6,
            0x03 => {
                let mut len_buf = [0u8; 1];
                tor_stream.read_exact(&mut len_buf).map_err(|e| format!("Failed to read domain response length: {}", e))?;
                len_buf[0] as usize + 2
            }
            0x04 => 18,
            _ => return Err(format!("Unsupported SOCKS address type in Tor response: {}", atyp_resp)),
        };
        let mut skip_buf = vec![0u8; skip_len];
        tor_stream.read_exact(&mut skip_buf).map_err(|e| format!("Failed to skip Tor response address: {}", e))?;

        tor_stream
    } else {
        // Direct clearnet connection
        let ip = match target_ip {
            Some(ip) => ip,
            None => {
                if let DestHost::Domain(ref domain) = dest_host {
                    // Resolve domain via standard system resolver since Tor is disabled
                    let addrs_iter = format!("{}:{}", domain, dest_port).to_socket_addrs()
                        .map_err(|e| format!("Failed to resolve '{}' for direct connection: {}", domain, e))?;
                    if let Some(addr) = addrs_iter.into_iter().next() {
                        addr.ip()
                    } else {
                        client.write_all(&[0x05, 0x04, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).ok();
                        return Err(format!("Hostname resolution returned zero addresses for direct connection: {}", domain));
                    }
                } else {
                    return Err("Missing target address for direct connection".to_string());
                }
            }
        };

        TcpStream::connect(SocketAddr::new(ip, dest_port))
            .map_err(|e| {
                // Connection refused
                client.write_all(&[0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).ok();
                format!("Direct connection failed: {}", e)
            })?
    };

    // 5. Send Success Response to WebKit
    client.write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
        .map_err(|e| format!("Failed to send success response: {}", e))?;

    // 6. Bidirectional Copy Tunneling
    let mut client_clone = client.try_clone().map_err(|e| format!("Failed to clone client socket: {}", e))?;
    let mut outbound_clone = outbound.try_clone().map_err(|e| format!("Failed to clone outbound socket: {}", e))?;

    let t = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        while let Ok(n) = client_clone.read(&mut buf) {
            if n == 0 { break; }
            if outbound_clone.write_all(&buf[..n]).is_err() { break; }
        }
        let _ = outbound_clone.shutdown(std::net::Shutdown::Both);
        let _ = client_clone.shutdown(std::net::Shutdown::Both);
    });

    let mut buf = [0u8; 8192];
    while let Ok(n) = outbound.read(&mut buf) {
        if n == 0 { break; }
        if client.write_all(&buf[..n]).is_err() { break; }
    }
    let _ = client.shutdown(std::net::Shutdown::Both);
    let _ = outbound.shutdown(std::net::Shutdown::Both);
    let _ = t.join();

    Ok(())
}

enum DestHost {
    Ip(IpAddr),
    Domain(String),
}
