pub mod daemon;

pub use daemon::{init_i2p, is_i2p_running, shutdown_i2p, I2P_SOCKS_PORT};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i2p_ports() {
        assert_eq!(I2P_SOCKS_PORT, 4447);
        assert_eq!(daemon::I2P_HTTP_PORT, 4444);
    }
}
