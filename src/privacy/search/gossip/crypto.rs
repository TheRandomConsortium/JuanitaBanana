use serde::{Deserialize, Serialize};

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
    pub fn random() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        NodeKeyPair {
            secret_key: secret.to_bytes(),
            public_key: public.to_bytes(),
        }
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_NODE_KEY: std::sync::Arc<std::sync::Mutex<Option<NodeKeyPair>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
}

pub fn is_node_key_unlocked() -> bool {
    if let Ok(guard) = GLOBAL_NODE_KEY.lock() {
        guard.is_some()
    } else {
        false
    }
}

pub fn unlock_node_key(password: &str) -> Result<(), String> {
    use crate::unsubscribe::db::SecureDbManager;
    let mut db = SecureDbManager::new_responsive(password)?;
    let kp = match db.get_p2p_node_key()? {
        Some((sec, pubk)) => NodeKeyPair {
            secret_key: sec,
            public_key: pubk,
        },
        None => {
            let new_kp = NodeKeyPair::random();
            db.save_p2p_node_key(&new_kp.secret_key, &new_kp.public_key)?;
            new_kp
        }
    };
    if let Ok(mut guard) = GLOBAL_NODE_KEY.lock() {
        *guard = Some(kp);
    }
    Ok(())
}

pub fn get_unlocked_node_key() -> Option<NodeKeyPair> {
    if let Ok(guard) = GLOBAL_NODE_KEY.lock() {
        guard.clone()
    } else {
        None
    }
}

pub fn derive_shared_session_key(local_secret: &[u8; 32], peer_public: &[u8; 32]) -> [u8; 32] {
    let secret = StaticSecret::from(*local_secret);
    let public = PublicKey::from(*peer_public);
    let shared_secret = secret.diffie_hellman(&public);
    *shared_secret.as_bytes()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GossipQueryPayload {
    pub query: String,
    pub source_node_id: String,
    pub timestamp_epoch: u64,
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
        Vec::new()
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
