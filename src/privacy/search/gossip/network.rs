use serde::{Deserialize, Serialize};

use super::crypto::{
    decrypt_query_with_session_key, derive_shared_session_key, encrypt_query_with_session_key,
    GossipQueryPayload,
};
use super::phonebook::GLOBAL_PHONEBOOK;
use super::pool::GLOBAL_NOISE_POOL;

const PROTOCOL_MAGIC: &[u8; 4] = b"JBP1";

fn attempt_upnp_hole_punching(port: u16) {
    std::thread::spawn(move || {
        let local_ipv4 = match std::net::UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => {
                if socket.connect(("9.9.9.9", 53)).is_ok() {
                    match socket.local_addr() {
                        Ok(std::net::SocketAddr::V4(addr)) => *addr.ip(),
                        _ => return,
                    }
                } else {
                    return;
                }
            }
            Err(_) => return,
        };

        if let Ok(gateway) = igd::search_gateway(Default::default()) {
            let local_endpoint = std::net::SocketAddrV4::new(local_ipv4, port);
            let _ = gateway.add_port(
                igd::PortMappingProtocol::UDP,
                port,
                local_endpoint,
                0,
                "Juanita Banana P2P",
            );
        }
    });
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerHandshakePayload {
    pub source_node_id: String,
    pub public_key: [u8; 32],
    pub listen_endpoint: String,
}

pub fn get_readable_node_id(config: &crate::util::config::AppConfig) -> String {
    let raw_secret = config.expected_secret_id();
    let prefix = if raw_secret.len() >= 8 {
        &raw_secret[..8]
    } else {
        "anon"
    };
    format!("node-juanita-{}", prefix)
}

pub fn is_prohibited_query(query: &str, prohibited_regex: &str) -> bool {
    let regex_trimmed = prohibited_regex.trim();
    if regex_trimmed.is_empty() {
        return false;
    }
    if let Ok(re) = regex::Regex::new(regex_trimmed) {
        re.is_match(query)
    } else {
        false
    }
}

pub fn is_node_banned(node_id: &str) -> bool {
    crate::browsing::is_peer_banned(node_id)
}

pub struct P2pGossipNetwork {
    pub socket_port: u16,
}

impl P2pGossipNetwork {
    pub fn new(port: u16) -> Self {
        Self { socket_port: port }
    }

    pub fn start_daemon(&self, config: crate::util::config::AppConfig) {
        if !config.allow_dht_search_sharing {
            return;
        }

        let port = self.socket_port;
        attempt_upnp_hole_punching(port);

        let config_clone = config.clone();
        std::thread::spawn(move || {
            let addr = format!("0.0.0.0:{}", port);
            if let Ok(socket) = std::net::UdpSocket::bind(&addr) {
                let _ = socket.set_read_timeout(Some(std::time::Duration::from_secs(2)));
                let mut buf = [0u8; 4096];
                let mut sent_initial_handshake = false;

                loop {
                    let local_key = match super::crypto::get_unlocked_node_key() {
                        Some(k) => k,
                        None => {
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            continue;
                        }
                    };

                    if !sent_initial_handshake {
                        sent_initial_handshake = true;
                        let hs_payload = PeerHandshakePayload {
                            source_node_id: get_readable_node_id(&config_clone),
                            public_key: local_key.public_key,
                            listen_endpoint: String::new(),
                        };
                        if let Ok(hs_socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
                            let _ = hs_socket.set_broadcast(true);
                            if let Ok(msg) = serde_json::to_vec(&hs_payload) {
                                let mut packet = PROTOCOL_MAGIC.to_vec();
                                packet.extend(msg);
                                let _ =
                                    hs_socket.send_to(&packet, format!("255.255.255.255:{}", port));
                            }
                        }
                    }

                    if let Ok((amt, src)) = socket.recv_from(&mut buf) {
                        if amt >= 4 && &buf[0..4] == PROTOCOL_MAGIC {
                            let payload = &buf[4..amt];

                            if let Ok(hs) = serde_json::from_slice::<PeerHandshakePayload>(payload)
                            {
                                if !is_node_banned(&hs.source_node_id) {
                                    if let Ok(mut pb) = GLOBAL_PHONEBOOK.lock() {
                                        let ep = if hs.listen_endpoint.is_empty()
                                            || hs.listen_endpoint.starts_with("0.0.0.0")
                                        {
                                            let mut resolved_src = src;
                                            resolved_src.set_port(port);
                                            resolved_src.to_string()
                                        } else {
                                            hs.listen_endpoint.clone()
                                        };
                                        pb.register_peer(hs.source_node_id, hs.public_key, ep);
                                    }
                                }
                                continue;
                            }

                            let active_peers = if let Ok(pb) = GLOBAL_PHONEBOOK.lock() {
                                pb.get_active_peers()
                            } else {
                                Vec::new()
                            };

                            for peer in active_peers {
                                if is_node_banned(&peer.node_id) {
                                    continue;
                                }
                                let session_key = derive_shared_session_key(
                                    &local_key.secret_key,
                                    &peer.public_key,
                                );
                                if let Some(gossip) =
                                    decrypt_query_with_session_key(payload, &session_key)
                                {
                                    if !is_node_banned(&gossip.source_node_id)
                                        && !is_prohibited_query(
                                            &gossip.query,
                                            &config.prohibited_keywords_regex,
                                        )
                                    {
                                        if let Ok(mut pool) = GLOBAL_NOISE_POOL.lock() {
                                            pool.add_term(
                                                gossip.query,
                                                gossip.source_node_id,
                                                config.search_terms_ttl_days,
                                            );
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn broadcast_search(&self, config: &crate::util::config::AppConfig, query: &str) {
        if !config.allow_dht_search_sharing {
            return;
        }
        if is_prohibited_query(query, &config.prohibited_keywords_regex) {
            return;
        }

        let local_key = match super::crypto::get_unlocked_node_key() {
            Some(k) => k,
            None => return, // Secret key is locked in RAM
        };
        let node_id = get_readable_node_id(config);
        let payload = GossipQueryPayload {
            query: query.to_string(),
            source_node_id: node_id,
            timestamp_epoch: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let active_peers = if let Ok(pb) = GLOBAL_PHONEBOOK.lock() {
            pb.get_active_peers()
        } else {
            Vec::new()
        };

        std::thread::spawn(move || {
            if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
                for peer in active_peers {
                    let session_key =
                        derive_shared_session_key(&local_key.secret_key, &peer.public_key);
                    let encrypted = encrypt_query_with_session_key(&payload, &session_key);

                    let mut packet = PROTOCOL_MAGIC.to_vec();
                    packet.extend(encrypted);

                    let _ = socket.send_to(&packet, &peer.endpoint);
                }
            }
        });
    }
}
