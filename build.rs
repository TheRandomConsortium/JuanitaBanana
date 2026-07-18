use std::process::Command;

fn main() {
    // ── hnsd ────────────────────────────────────────────────────────────────
    let build_handshake = std::env::var("CARGO_FEATURE_HANDSHAKE").is_ok();
    if build_handshake {
        println!("cargo:rerun-if-changed=scripts/sh/build_hnsd.sh");
        if !std::path::Path::new("bin/hnsd").exists() {
            let status = Command::new("bash")
                .arg("./scripts/sh/build_hnsd.sh")
                .status()
                .expect("Failed to execute scripts/sh/build_hnsd.sh");

            if !status.success() {
                panic!("build_hnsd.sh failed with exit code: {:?}", status.code());
            }
        }
    }

    // ── arti (Tor transport) ────────────────────────────────────────────────
    // NOTE: interim subprocess strategy. Phase 4 will embed arti-client in-process.
    let build_tor = std::env::var("CARGO_FEATURE_TOR").is_ok();
    if build_tor {
        println!("cargo:rerun-if-changed=scripts/sh/build_arti.sh");
        if !std::path::Path::new("bin/arti").exists() {
            let status = Command::new("bash")
                .arg("./scripts/sh/build_arti.sh")
                .status()
                .expect("Failed to execute scripts/sh/build_arti.sh");

            if !status.success() {
                // Non-fatal: arti is optional at build time. The browser logs a warning
                // at runtime when Tor is enabled but arti is not found.
                eprintln!(
                    "WARNING: build_arti.sh failed (exit {:?}). \
                     Tor transport will not function until `arti` is available in bin/ or PATH.",
                    status.code()
                );
            }
        }
    }
}
