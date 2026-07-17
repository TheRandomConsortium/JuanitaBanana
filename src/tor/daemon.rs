use crate::util::config::AppConfig;
use lazy_static::lazy_static;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

/// The SOCKS5 port on which the local arti daemon will listen.
pub const ARTI_SOCKS_PORT: u16 = 9150;

lazy_static! {
    static ref ARTI_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
}

/// Locates the `arti` binary in standard locations.
fn find_arti_path() -> Option<PathBuf> {
    // 1. Relative to CWD: bin/arti
    let local = PathBuf::from("bin/arti");
    if local.exists() {
        return Some(local);
    }
    // 2. Next to the running executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let candidate = parent.join("arti");
            if candidate.exists() {
                return Some(candidate);
            }
            let candidate_bin = parent.join("bin").join("arti");
            if candidate_bin.exists() {
                return Some(candidate_bin);
            }
        }
    }
    // 3. /usr/bin/arti
    let usr_bin = PathBuf::from("/usr/bin/arti");
    if usr_bin.exists() {
        return Some(usr_bin);
    }
    // 4. PATH lookup via `which`
    if let Ok(output) = Command::new("which").arg("arti").output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let path = PathBuf::from(&path_str);
            if path.exists() {
                return Some(path);
            }
        }
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

/// Initialises the Tor transport. Starts the `arti` daemon if Tor is enabled in config.
/// This is idempotent — calling it again while arti is already running is a no-op.
pub fn init_tor() {
    let config = AppConfig::load();
    if !config.tor_enabled {
        crate::log!(
            Info,
            TOR,
            "Tor transport is disabled in config — not starting arti"
        );
        shutdown_tor();
        return;
    }

    {
        let mut lock = ARTI_PROCESS.lock().unwrap();
        if let Some(ref mut child) = *lock {
            if let Ok(None) = child.try_wait() {
                // Already running
                return;
            } else {
                *lock = None;
            }
        }
    }

    let arti_bin = match find_arti_path() {
        Some(p) => p,
        None => {
            crate::log!(
                Info,
                TOR,
                "arti binary not found — Tor transport will not function. \
                 Install arti (https://gitlab.torproject.org/tpo/core/arti) \
                 or place the binary at bin/arti next to the executable."
            );
            return;
        }
    };

    let state_dir = base_data_dir().join("arti_state");
    std::fs::create_dir_all(&state_dir).ok();

    let socks_addr = format!("127.0.0.1:{}", ARTI_SOCKS_PORT);

    crate::log!(
        Info,
        TOR,
        "Starting arti daemon from {:?}",
        arti_bin.to_string_lossy()
    );

    // arti proxy --socks-port 9150 --state-dir <dir>
    match Command::new(&arti_bin)
        .arg("proxy")
        .arg("--socks-port")
        .arg(ARTI_SOCKS_PORT.to_string())
        .arg("--state-dir")
        .arg(&state_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(stdout) = child.stdout.take() {
                std::thread::spawn(move || {
                    let reader = std::io::BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        crate::log!(Trace, TOR, "arti: {}", line);
                    }
                });
            }
            if let Some(stderr) = child.stderr.take() {
                std::thread::spawn(move || {
                    let reader = std::io::BufReader::new(stderr);
                    for line in reader.lines().map_while(Result::ok) {
                        crate::log!(Debug, TOR, "arti: {}", line);
                    }
                });
            }
            {
                let mut lock = ARTI_PROCESS.lock().unwrap();
                *lock = Some(child);
            }
            crate::log!(
                Info,
                TOR,
                "arti daemon started — SOCKS5 proxy listening on {}",
                socks_addr
            );
        }
        Err(e) => {
            crate::log!(Info, TOR, "Failed to start arti daemon: {}", e);
        }
    }
}

/// Returns `true` if the arti daemon subprocess is running.
pub fn is_tor_running() -> bool {
    let mut lock = ARTI_PROCESS.lock().unwrap();
    if let Some(ref mut child) = *lock {
        matches!(child.try_wait(), Ok(None))
    } else {
        false
    }
}

/// Shuts down the arti daemon if it was started by us.
pub fn shutdown_tor() {
    let mut lock = ARTI_PROCESS.lock().unwrap();
    if let Some(mut child) = lock.take() {
        crate::log!(Info, TOR, "Terminating arti daemon...");
        #[cfg(unix)]
        {
            let pid = child.id();
            let mut killed = false;
            if Command::new("kill")
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
        crate::log!(Info, TOR, "arti daemon terminated");
    }
}
