use crate::util::config::AppConfig;
use lazy_static::lazy_static;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

lazy_static! {
    static ref HNSD_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
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
    if !config.handshake_enabled || !config.resolver_order.contains(&"Handshake".to_string()) {
        crate::log!(
            Info,
            RESOLVER,
            "Handshake resolver not enabled or not in resolver_order, stopping hnsd daemon if active"
        );
        shutdown_resolver();
        return;
    }

    {
        let mut lock = HNSD_PROCESS.lock().unwrap();
        if let Some(ref mut child) = *lock {
            if let Ok(None) = child.try_wait() {
                // Process is still running, do not relaunch
                return;
            } else {
                // Process has exited/failed, clean up handle so we can restart
                *lock = None;
            }
        }
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

    // When Tor is enabled, attempt to route hnsd's P2P UDP traffic through Tor
    // by wrapping the subprocess with torsocks (HNS-over-Tor, Option 1).
    // If torsocks is not available, fall back to direct hnsd with a clear warning.
    let config_for_tor = AppConfig::load();
    let use_torsocks = config_for_tor.tor_enabled
        && std::process::Command::new("which")
            .arg("torsocks")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    if use_torsocks {
        crate::log!(
            Info,
            RESOLVER,
            "torsocks detected — wrapping hnsd with torsocks for HNS-over-Tor"
        );
    } else if config_for_tor.tor_enabled {
        crate::log!(
            Info,
            RESOLVER,
            "torsocks not found in PATH — hnsd will query the HNS P2P network directly \
             (not through Tor). Install torsocks for full HNS-over-Tor support."
        );
    }

    let spawn_result = if use_torsocks {
        Command::new("torsocks")
            .arg(&hnsd_bin)
            .arg("-n")
            .arg("127.0.0.1:5349")
            .arg("-r")
            .arg("127.0.0.1:5350")
            .arg("-p")
            .arg("8")
            .arg("-x")
            .arg(&state_dir)
            .arg("-t")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else {
        Command::new(&hnsd_bin)
            .arg("-n")
            .arg("127.0.0.1:5349")
            .arg("-r")
            .arg("127.0.0.1:5350")
            .arg("-p")
            .arg("8")
            .arg("-x")
            .arg(&state_dir)
            .arg("-t")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    };

    match spawn_result {
        Ok(mut child) => {
            if let Some(stdout) = child.stdout.take() {
                std::thread::spawn(move || {
                    let reader = std::io::BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            crate::log!(Trace, RESOLVER, "hnsd: {}", line);
                        } else {
                            break;
                        }
                    }
                });
            }
            if let Some(stderr) = child.stderr.take() {
                std::thread::spawn(move || {
                    let reader = std::io::BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            crate::log!(Error, RESOLVER, "hnsd error: {}", line);
                        } else {
                            break;
                        }
                    }
                });
            }
            let mut lock = HNSD_PROCESS.lock().unwrap();
            *lock = Some(child);
            crate::log!(
                Info,
                RESOLVER,
                "hnsd daemon started, recursive nameserver listening on 127.0.0.1:5350"
            );
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
        #[cfg(unix)]
        {
            let pid = child.id();
            let mut killed = false;
            if std::process::Command::new("kill")
                .arg("-15")
                .arg(pid.to_string())
                .status()
                .is_ok()
            {
                for _ in 0..15 {
                    if let Ok(Some(_)) = child.try_wait() {
                        killed = true;
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(200));
                }
            }
            if !killed {
                let _ = child.kill();
            }
        }
        #[cfg(not(unix))]
        {
            let _ = child.kill();
        }
        let _ = child.wait();
        crate::log!(Info, RESOLVER, "hnsd daemon terminated");
    }
}
