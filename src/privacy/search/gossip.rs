use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeKeyPair {
    pub secret_key: [u8; 32],
    pub public_key: [u8; 32],
}

impl NodeKeyPair {
    fn key_file_path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
            });
        base.join("juanita-banana").join("node_key.bin")
    }

    pub fn load_or_generate(_config: &crate::util::config::AppConfig) -> Self {
        let path = Self::key_file_path();
        if path.exists() {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(kp) = bincode::deserialize::<NodeKeyPair>(&bytes) {
                    return kp;
                }
            }
        }

        // Generar un par de claves seguras en la Curva 25519
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);

        let kp = NodeKeyPair {
            secret_key: secret.to_bytes(),
            public_key: public.to_bytes(),
        };

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bin) = bincode::serialize(&kp) {
            let _ = fs::write(path, bin);
        }
        kp
    }
}

pub fn derive_shared_session_key(local_secret: &[u8; 32], peer_public: &[u8; 32]) -> [u8; 32] {
    let secret = StaticSecret::from(*local_secret);
    let public = PublicKey::from(*peer_public);
    let shared_secret = secret.diffie_hellman(&public);
    *shared_secret.as_bytes() // Clave robusta acordada mutuamente por ECDH
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerHandshakePayload {
    pub source_node_id: String,
    pub public_key: [u8; 32],
    pub listen_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GossipQueryPayload {
    pub query: String,
    pub source_node_id: String,
    pub timestamp_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerIdentity {
    pub node_id: String,
    pub public_key: [u8; 32],
    pub endpoint: String,
    pub last_seen_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwarmPhonebook {
    pub peers: Vec<PeerIdentity>,
}

impl SwarmPhonebook {
    fn phonebook_path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
            });
        base.join("juanita-banana").join("phonebook.bin")
    }

    pub fn empty() -> Self {
        Self { peers: Vec::new() }
    }

    pub fn load() -> Self {
        let path = Self::phonebook_path();
        if path.exists() {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(pb) = bincode::deserialize::<SwarmPhonebook>(&bytes) {
                    return pb;
                }
            }
        }
        SwarmPhonebook::default()
    }

    pub fn save(&self) {
        let path = Self::phonebook_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bin) = bincode::serialize(self) {
            let _ = fs::write(path, bin);
        }
    }

    pub fn register_peer(&mut self, node_id: String, public_key: [u8; 32], endpoint: String) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.peers.retain(|p| p.node_id != node_id);
        self.peers.push(PeerIdentity {
            node_id,
            public_key,
            endpoint,
            last_seen_epoch: now,
        });
        self.save();
    }

    pub fn remove_peer(&mut self, node_id: &str) {
        self.peers.retain(|p| p.node_id != node_id);
        self.save();
    }

    pub fn get_active_peers(&self) -> Vec<PeerIdentity> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.peers
            .iter()
            .filter(|p| now.saturating_sub(p.last_seen_epoch) < 86400)
            .cloned()
            .collect()
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_PHONEBOOK: std::sync::Arc<std::sync::Mutex<SwarmPhonebook>> =
        std::sync::Arc::new(std::sync::Mutex::new(SwarmPhonebook::load()));
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchPoolEntry {
    pub term: String,
    pub origin: String,
    pub ingested_epoch: u64,
    pub expires_epoch: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SearchNoisePool {
    pub entries: Vec<SearchPoolEntry>,
}

impl SearchNoisePool {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn clean_expired(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Cierra la fuga de memoria purgando los caducados
        self.entries
            .retain(|e| e.expires_epoch == 0 || e.expires_epoch > now);
    }

    pub fn add_term(&mut self, term: String, origin: String, ttl_days: u32) {
        self.clean_expired();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let expires = now + (ttl_days as u64 * 86400);
        self.entries.push(SearchPoolEntry {
            term,
            origin,
            ingested_epoch: now,
            expires_epoch: expires,
        });
    }

    pub fn get_all_terms(&mut self) -> Vec<SearchPoolEntry> {
        self.clean_expired();
        self.entries.clone()
    }

    pub fn remove_term(&mut self, term: &str) {
        self.entries.retain(|e| e.term != term);
    }

    pub fn remove_by_node(&mut self, node_id: &str) {
        let node_id_clean = node_id.trim().to_lowercase();
        self.entries.retain(|e| {
            let origin_clean = e.origin.trim().to_lowercase();
            !origin_clean.contains(&node_id_clean)
        });
    }
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

pub fn is_node_banned(node_id: &str, banned_peers: &[String]) -> bool {
    let cleaned = node_id.trim().to_lowercase();
    banned_peers
        .iter()
        .any(|banned| banned.trim().to_lowercase() == cleaned)
}

pub fn encrypt_query_with_session_key(
    payload: &GossipQueryPayload,
    session_key: &[u8; 32],
) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new(session_key.into());
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let data = serde_json::to_vec(payload).unwrap_or_default();

    if let Ok(ciphertext) = cipher.encrypt(&nonce, data.as_ref()) {
        let mut packed = nonce.to_vec();
        packed.extend(ciphertext);
        packed
    } else {
        Vec::new() // Fallo fatal de encriptación AEAD
    }
}

pub fn decrypt_query_with_session_key(
    encrypted: &[u8],
    session_key: &[u8; 32],
) -> Option<GossipQueryPayload> {
    if encrypted.len() < 12 {
        return None;
    }
    let cipher = ChaCha20Poly1305::new(session_key.into());
    let nonce = Nonce::from_slice(&encrypted[..12]);
    let ciphertext = &encrypted[12..];

    if let Ok(decrypted) = cipher.decrypt(nonce, ciphertext) {
        serde_json::from_slice(&decrypted).ok()
    } else {
        None
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_NOISE_POOL: std::sync::Arc<std::sync::Mutex<SearchNoisePool>> =
        std::sync::Arc::new(std::sync::Mutex::new(SearchNoisePool::new()));
}

pub struct P2pGossipNetwork {
    pub socket_port: u16,
}

impl P2pGossipNetwork {
    pub fn new(port: u16) -> Self {
        Self { socket_port: port }
    }

    pub fn start_daemon(&self, config: crate::util::config::AppConfig, banned_peers: Vec<String>) {
        if !config.allow_dht_search_sharing {
            return;
        }

        let port = self.socket_port;
        let local_key = NodeKeyPair::load_or_generate(&config);

        // Fase 1: Enviar Handshake de Bootstrap en Broadcast a la subred local
        let hs_payload = PeerHandshakePayload {
            source_node_id: get_readable_node_id(&config),
            public_key: local_key.public_key,
            listen_endpoint: String::new(),
        };
        std::thread::spawn(move || {
            if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
                let _ = socket.set_broadcast(true);
                if let Ok(msg) = serde_json::to_vec(&hs_payload) {
                    let _ = socket.send_to(&msg, format!("255.255.255.255:{}", port));
                }
            }
        });

        // Fase 2: Escuchar el tráfico entrante
        std::thread::spawn(move || {
            let addr = format!("0.0.0.0:{}", port);
            if let Ok(socket) = std::net::UdpSocket::bind(&addr) {
                let _ = socket.set_read_timeout(Some(std::time::Duration::from_secs(2)));
                let mut buf = [0u8; 4096];

                loop {
                    if let Ok((amt, src)) = socket.recv_from(&mut buf) {
                        if amt > 0 {
                            // Intentar procesar como Handshake
                            if let Ok(hs) =
                                serde_json::from_slice::<PeerHandshakePayload>(&buf[..amt])
                            {
                                if !is_node_banned(&hs.source_node_id, &banned_peers) {
                                    if let Ok(mut pb) = GLOBAL_PHONEBOOK.lock() {
                                        let ep = if hs.listen_endpoint.is_empty()
                                            || hs.listen_endpoint.starts_with("0.0.0.0")
                                        {
                                            let mut resolved_src = src;
                                            resolved_src.set_port(port); // Forzamos el puerto estándar P2P
                                            resolved_src.to_string()
                                        } else {
                                            hs.listen_endpoint.clone()
                                        };
                                        pb.register_peer(hs.source_node_id, hs.public_key, ep);
                                    }
                                }
                                continue;
                            }

                            // Intentar descifrar como Búsqueda Entrante (E2EE)
                            let active_peers = if let Ok(pb) = GLOBAL_PHONEBOOK.lock() {
                                pb.get_active_peers()
                            } else {
                                Vec::new()
                            };

                            for peer in active_peers {
                                if is_node_banned(&peer.node_id, &banned_peers) {
                                    continue;
                                }
                                let session_key = derive_shared_session_key(
                                    &local_key.secret_key,
                                    &peer.public_key,
                                );
                                if let Some(payload) =
                                    decrypt_query_with_session_key(&buf[..amt], &session_key)
                                {
                                    if !is_node_banned(&payload.source_node_id, &banned_peers)
                                        && !is_prohibited_query(
                                            &payload.query,
                                            &config.prohibited_keywords_regex,
                                        )
                                    {
                                        if let Ok(mut pool) = GLOBAL_NOISE_POOL.lock() {
                                            pool.add_term(
                                                payload.query,
                                                payload.source_node_id,
                                                config.search_terms_ttl_days,
                                            );
                                        }
                                    }
                                    break; // Match de descifrado exitoso, parar de iterar peers
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

        let local_key = NodeKeyPair::load_or_generate(config);
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
                // Broadcast false (es tráfico unicast directo y cifrado E2EE a cada peer)
                for peer in active_peers {
                    let session_key =
                        derive_shared_session_key(&local_key.secret_key, &peer.public_key);
                    let encrypted = encrypt_query_with_session_key(&payload, &session_key);
                    let _ = socket.send_to(&encrypted, &peer.endpoint);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_keypair_generation_and_dh_session_key() {
        let config = crate::util::config::AppConfig::default();

        let node_a = NodeKeyPair::load_or_generate(&config);
        let node_b = NodeKeyPair::load_or_generate(&config);

        assert_ne!(node_a.secret_key, [0u8; 32]);
        assert_ne!(node_a.public_key, [0u8; 32]);

        let session_key_a = derive_shared_session_key(&node_a.secret_key, &node_b.public_key);
        let session_key_b = derive_shared_session_key(&node_b.secret_key, &node_a.public_key);

        assert_eq!(session_key_a, session_key_b);
        assert_ne!(session_key_a, [0u8; 32]);

        let payload = GossipQueryPayload {
            query: "handshake test".to_string(),
            source_node_id: "node-test-1".to_string(),
            timestamp_epoch: 1700000000,
        };

        let encrypted = encrypt_query_with_session_key(&payload, &session_key_a);
        let decrypted = decrypt_query_with_session_key(&encrypted, &session_key_b).unwrap();

        assert_eq!(payload, decrypted);
    }

    #[test]
    fn test_swarm_phonebook_operations() {
        let mut pb = SwarmPhonebook::empty();
        let pk = [0x55u8; 32];
        pb.register_peer("node-peer-1".to_string(), pk, "127.0.0.1:7744".to_string());

        let active = pb.get_active_peers();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].node_id, "node-peer-1");
        assert_eq!(active[0].public_key, pk);

        pb.remove_peer("node-peer-1");
        assert!(pb.get_active_peers().is_empty());
    }
}
