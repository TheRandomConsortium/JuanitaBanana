use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum DestHost {
    Ip(IpAddr),
    Domain(String),
}

pub fn handle_i2p_connection(
    client: &mut TcpStream,
    dest_host: &DestHost,
    target_ip: Option<IpAddr>,
    dest_port: u16,
) -> Result<(), String> {
    let socks_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        crate::i2p::I2P_SOCKS_PORT,
    );

    let is_i2p_domain = match dest_host {
        DestHost::Domain(ref domain) => domain.ends_with(".i2p"),
        DestHost::Ip(_) => false,
    };

    // 1. Try SOCKS5 proxy first (port 4447)
    if let Ok(mut socks_stream) = TcpStream::connect(socks_addr) {
        socks_stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .ok();
        socks_stream
            .set_write_timeout(Some(Duration::from_secs(30)))
            .ok();

        if socks_stream.write_all(&[0x05, 0x01, 0x00]).is_ok() {
            let mut socks_greeting = [0u8; 2];
            if socks_stream.read_exact(&mut socks_greeting).is_ok()
                && socks_greeting[0] == 0x05
                && socks_greeting[1] == 0x00
            {
                let mut connect_req = Vec::new();
                connect_req.extend_from_slice(&[0x05, 0x01, 0x00]);
                match (target_ip, dest_host) {
                    (Some(ip), _) => match ip {
                        IpAddr::V4(ipv4) => {
                            connect_req.push(0x01);
                            connect_req.extend_from_slice(&ipv4.octets());
                        }
                        IpAddr::V6(ipv6) => {
                            connect_req.push(0x04);
                            connect_req.extend_from_slice(&ipv6.octets());
                        }
                    },
                    (None, DestHost::Domain(ref domain)) => {
                        connect_req.push(0x03);
                        connect_req.push(domain.len() as u8);
                        connect_req.extend_from_slice(domain.as_bytes());
                    }
                    (None, DestHost::Ip(ip)) => match ip {
                        IpAddr::V4(ipv4) => {
                            connect_req.push(0x01);
                            connect_req.extend_from_slice(&ipv4.octets());
                        }
                        IpAddr::V6(ipv6) => {
                            connect_req.push(0x04);
                            connect_req.extend_from_slice(&ipv6.octets());
                        }
                    },
                }
                connect_req.extend_from_slice(&dest_port.to_be_bytes());
                if socks_stream.write_all(&connect_req).is_ok() {
                    let mut socks_resp = [0u8; 4];
                    if socks_stream.read_exact(&mut socks_resp).is_ok()
                        && socks_resp[0] == 0x05
                        && socks_resp[1] == 0x00
                    {
                        let skip_len = match socks_resp[3] {
                            0x01 => 6,
                            0x03 => {
                                let mut len_buf = [0u8; 1];
                                if socks_stream.read_exact(&mut len_buf).is_ok() {
                                    len_buf[0] as usize + 2
                                } else {
                                    0
                                }
                            }
                            0x04 => 18,
                            _ => 0,
                        };
                        if skip_len > 0 {
                            let mut skip_buf = vec![0u8; skip_len];
                            let _ = socks_stream.read_exact(&mut skip_buf);
                        }

                        // Send success to WebKit client
                        client
                            .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                            .map_err(|e| format!("Failed to send success response: {}", e))?;

                        return tunnel_streams(client, &mut socks_stream);
                    }
                }
            }
        }
    }

    // 2. Fallback to I2P HTTP Proxy on port 4444 (Java I2P default)
    let http_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        crate::i2p::daemon::I2P_HTTP_PORT,
    );
    let mut http_stream = TcpStream::connect(http_addr).map_err(|e| {
        format!(
            "Failed to connect to I2P HTTP proxy on port {}: {}",
            crate::i2p::daemon::I2P_HTTP_PORT,
            e
        )
    })?;

    http_stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .ok();
    http_stream
        .set_write_timeout(Some(Duration::from_secs(30)))
        .ok();

    client
        .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
        .map_err(|e| format!("Failed to send SOCKS5 success to client: {}", e))?;

    let host_header = match dest_host {
        DestHost::Domain(d) => d.clone(),
        DestHost::Ip(ip) => ip.to_string(),
    };

    let outbound_host = if is_i2p_domain {
        host_header.clone()
    } else if let Some(ip) = target_ip {
        ip.to_string()
    } else {
        host_header.clone()
    };

    let mut buf = [0u8; 8192];
    let n = client
        .read(&mut buf)
        .map_err(|e| format!("Failed to read initial request from client: {}", e))?;
    if n == 0 {
        return Ok(());
    }

    let req_data = &buf[..n];
    let req_str = String::from_utf8_lossy(req_data);

    if dest_port == 443 || req_data.starts_with(b"\x16\x03") || req_str.starts_with("CONNECT ") {
        let connect_cmd = format!(
            "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\nCache-Control: no-cache\r\nPragma: no-cache\r\n\r\n",
            outbound_host, dest_port, host_header, dest_port
        );
        http_stream
            .write_all(connect_cmd.as_bytes())
            .map_err(|e| format!("Failed to send CONNECT to I2P HTTP proxy: {}", e))?;

        let mut resp_buf = Vec::new();
        let mut byte_buf = [0u8; 1];
        while http_stream.read_exact(&mut byte_buf).is_ok() {
            resp_buf.push(byte_buf[0]);
            if resp_buf.ends_with(b"\r\n\r\n") {
                break;
            }
        }
        let resp_str = String::from_utf8_lossy(&resp_buf);
        if !resp_str.contains(" 200 ") {
            client.write_all(&resp_buf).ok();
            return Err(format!(
                "I2P HTTP CONNECT failed: {}",
                resp_str.lines().next().unwrap_or("")
            ));
        }

        if !req_str.starts_with("CONNECT ") {
            http_stream.write_all(req_data).ok();
        }
    } else {
        let rewritten_req = rewrite_http_proxy_request(&req_str, &outbound_host);
        http_stream
            .write_all(rewritten_req.as_bytes())
            .map_err(|e| format!("Failed to send request to I2P HTTP proxy: {}", e))?;
    }

    tunnel_streams(client, &mut http_stream)
}

pub fn tunnel_streams(client: &mut TcpStream, outbound: &mut TcpStream) -> Result<(), String> {
    let mut client_clone = client
        .try_clone()
        .map_err(|e| format!("Failed to clone client socket: {}", e))?;
    let mut outbound_clone = outbound
        .try_clone()
        .map_err(|e| format!("Failed to clone outbound socket: {}", e))?;

    let t = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        while let Ok(n) = client_clone.read(&mut buf) {
            if n == 0 {
                break;
            }
            if outbound_clone.write_all(&buf[..n]).is_err() {
                break;
            }
        }
        let _ = outbound_clone.shutdown(std::net::Shutdown::Both);
        let _ = client_clone.shutdown(std::net::Shutdown::Both);
    });

    let mut buf = [0u8; 8192];
    while let Ok(n) = outbound.read(&mut buf) {
        if n == 0 {
            break;
        }
        if client.write_all(&buf[..n]).is_err() {
            break;
        }
    }
    let _ = client.shutdown(std::net::Shutdown::Both);
    let _ = outbound.shutdown(std::net::Shutdown::Both);
    let _ = t.join();

    Ok(())
}

fn rewrite_http_proxy_request(req_str: &str, outbound_host: &str) -> String {
    let methods = ["GET ", "POST ", "HEAD ", "PUT ", "DELETE ", "OPTIONS "];
    if let Some(_method) = methods.iter().find(|m| req_str.starts_with(**m)) {
        if let Some((first_line, rest)) = req_str.split_once("\r\n") {
            let relative_target = " /";
            let absolute_target = format!(" http://{}/", outbound_host);
            let rewritten_first = first_line.replacen(relative_target, &absolute_target, 1);
            return format!(
                "{}\r\nCache-Control: no-cache\r\nPragma: no-cache\r\n{}",
                rewritten_first, rest
            );
        }
    }
    req_str.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_http_proxy_request() {
        let raw = "GET /index.html HTTP/1.1\r\nHost: reg.i2p\r\n\r\n";
        let rewritten = rewrite_http_proxy_request(raw, "reg.i2p");
        assert!(rewritten.starts_with("GET http://reg.i2p/index.html HTTP/1.1\r\nCache-Control: no-cache"));
        assert!(rewritten.contains("Host: reg.i2p"));
    }
}
