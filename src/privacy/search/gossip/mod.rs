pub mod crypto;
pub mod network;
pub mod phonebook;
pub mod pool;

pub use crypto::*;
pub use network::*;
pub use phonebook::*;
pub use pool::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_keypair_generation_and_dh_session_key() {
        let node_a = NodeKeyPair::random();
        let node_b = NodeKeyPair::random();

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
