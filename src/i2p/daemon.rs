use crate::log;
use crate::util::config::AppConfig;
use lazy_static::lazy_static;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

pub const I2P_SOCKS_PORT: u16 = 4447;
pub const I2P_HTTP_PORT: u16 = 4444;

lazy_static! {
    static ref I2P_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
}

fn find_i2p_binary() -> Option<PathBuf> {
    let names = ["i2prouter", "i2pd", "i2p-rs"];
    for name in &names {
        let local = PathBuf::from(format!("bin/{}", name));
        if local.exists() {
            return Some(local);
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            for name in &names {
                let candidate = parent.join(name);
                if candidate.exists() {
                    return Some(candidate);
                }
                let candidate_bin = parent.join("bin").join(name);
                if candidate_bin.exists() {
                    return Some(candidate_bin);
                }
            }
        }
    }

    let sys_candidates = [
        PathBuf::from("/usr/bin/i2prouter"),
        PathBuf::from("/usr/bin/i2pd"),
        PathBuf::from("/usr/bin/i2p-rs"),
        PathBuf::from("/usr/share/i2p/i2prouter"),
    ];
    for c in &sys_candidates {
        if c.exists() {
            return Some(c.clone());
        }
    }

    for name in &names {
        if let Ok(output) = Command::new("which").arg(name).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let path = PathBuf::from(&path_str);
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    None
}

pub fn init_i2p() {
    let config = AppConfig::load();
    if !config.i2p_enabled {
        log!(Info, I2P, "I2P transport is disabled in configuration");
        return;
    }

    if is_i2p_running() {
        log!(
            Info,
            I2P,
            "User's active I2P router daemon detected alive on 127.0.0.1:{}",
            I2P_SOCKS_PORT
        );
        return;
    }

    let i2p_bin = match find_i2p_binary() {
        Some(b) => b,
        None => {
            log!(
                Warn,
                I2P,
                "I2P transport is enabled in config, but no active router daemon found on 127.0.0.1:{} or :{}. Please start i2prouter or i2pd service.",
                I2P_SOCKS_PORT,
                I2P_HTTP_PORT
            );
            return;
        }
    };

    let mut lock = I2P_PROCESS.lock().unwrap();
    if lock.is_some() {
        return;
    }

    log!(Info, I2P, "Starting I2P daemon from {}", i2p_bin.display());
    let mut cmd = Command::new(&i2p_bin);
    if i2p_bin.file_name().and_then(|s| s.to_str()) == Some("i2prouter") {
        cmd.arg("start");
    } else if i2p_bin.file_name().and_then(|s| s.to_str()) == Some("i2pd") {
        cmd.arg("--daemon");
    }

    cmd.stdout(Stdio::null()).stderr(Stdio::null());

    match cmd.spawn() {
        Ok(child) => {
            *lock = Some(child);
            log!(
                Info,
                I2P,
                "I2P router process spawned, awaiting readiness..."
            );
            for _ in 0..15 {
                std::thread::sleep(Duration::from_millis(200));
                if is_i2p_running() {
                    log!(
                        Info,
                        I2P,
                        "I2P router daemon is now ready on 127.0.0.1:{}",
                        I2P_SOCKS_PORT
                    );
                    return;
                }
            }
            log!(
                Warn,
                I2P,
                "I2P router spawned but not yet responding on SOCKS port {}",
                I2P_SOCKS_PORT
            );
        }
        Err(e) => {
            log!(Error, I2P, "Failed to spawn I2P router: {}", e);
        }
    }
}

pub fn is_i2p_running() -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], I2P_SOCKS_PORT));
    if TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok() {
        return true;
    }
    let http_addr = SocketAddr::from(([127, 0, 0, 1], I2P_HTTP_PORT));
    TcpStream::connect_timeout(&http_addr, Duration::from_millis(200)).is_ok()
}

pub fn shutdown_i2p() {
    let mut lock = I2P_PROCESS.lock().unwrap();
    if let Some(mut child) = lock.take() {
        log!(Info, I2P, "Terminating managed I2P process...");
        let _ = child.kill();
        let _ = child.wait();
        log!(Info, I2P, "Managed I2P process terminated");
    }
}
