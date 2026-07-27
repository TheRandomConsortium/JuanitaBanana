use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // ── Tidiness & Cleanliness Check ───────────────────────────────────────
    println!("cargo:rerun-if-changed=scripts/sh/check_cleanliness.sh");
    let check_status = Command::new("bash")
        .arg("./scripts/sh/check_cleanliness.sh")
        .status()
        .expect("Failed to execute scripts/sh/check_cleanliness.sh");

    if !check_status.success() {
        panic!("Project tidiness check failed! Fix clutter violations before building.");
    }

    // ── Root Path Resolution Macro Generator (.scannable) ───────────────────
    generate_root_path_macros();

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
    let build_tor = std::env::var("CARGO_FEATURE_TOR").is_ok();
    if build_tor {
        println!("cargo:rerun-if-changed=scripts/sh/build_arti.sh");
        if !std::path::Path::new("bin/arti").exists() {
            let status = Command::new("bash")
                .arg("./scripts/sh/build_arti.sh")
                .status()
                .expect("Failed to execute scripts/sh/build_arti.sh");

            if !status.success() {
                eprintln!(
                    "WARNING: build_arti.sh failed (exit {:?}). \
                     Tor transport will not function until `arti` is available in bin/ or PATH.",
                    status.code()
                );
            }
        }
    }

    // ── I2P (Garlic transport) ──────────────────────────────────────────────
    let build_i2p = std::env::var("CARGO_FEATURE_I2P").is_ok();
    if build_i2p {
        println!("cargo:rerun-if-changed=scripts/sh/build_i2p.sh");
        let has_bin = std::path::Path::new("bin/i2prouter").exists()
            || std::path::Path::new("bin/i2p.jar").exists()
            || std::path::Path::new("bin/i2p-rs").exists();
        if !has_bin {
            let status = Command::new("bash")
                .arg("./scripts/sh/build_i2p.sh")
                .status()
                .expect("Failed to execute scripts/sh/build_i2p.sh");

            if !status.success() {
                eprintln!(
                    "WARNING: build_i2p.sh failed (exit {:?}). \
                     I2P transport will not function until `i2prouter` or `i2p.jar` is available in bin/ or PATH.",
                    status.code()
                );
            }
        }
    }
}

fn generate_root_path_macros() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let root_path = Path::new(&manifest_dir);

    let mut str_rules = String::new();
    let mut bytes_rules = String::new();

    if let Ok(entries) = fs::read_dir(root_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let scannable_file = path.join(".scannable");
                if scannable_file.exists() {
                    println!("cargo:rerun-if-changed={}", scannable_file.display());
                    let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let (inclusion_type, search_mode) = parse_scannable_config(&scannable_file);

                    let mut files = Vec::new();
                    collect_files(&path, search_mode == "recursive", &mut files);

                    for file in files {
                        println!("cargo:rerun-if-changed={}", file.display());
                        let rel_to_manifest = file
                            .strip_prefix(root_path)
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        let rel_to_root = file
                            .strip_prefix(&path)
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        let file_name = file.file_name().unwrap().to_string_lossy().to_string();

                        let generate_str = inclusion_type == "string" || inclusion_type == "both";
                        let generate_bytes = inclusion_type == "bytes" || inclusion_type == "both";

                        if generate_str {
                            str_rules.push_str(&format!(
                                "    (@{}, {:?}) => {{ include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\")) }};\n",
                                dir_name, rel_to_root, rel_to_manifest
                            ));
                            if file_name != rel_to_root {
                                str_rules.push_str(&format!(
                                    "    (@{}, {:?}) => {{ include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\")) }};\n",
                                    dir_name, file_name, rel_to_manifest
                                ));
                            }
                        }

                        if generate_bytes {
                            bytes_rules.push_str(&format!(
                                "    (@{}, {:?}) => {{ include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\")) }};\n",
                                dir_name, rel_to_root, rel_to_manifest
                            ));
                            if file_name != rel_to_root {
                                bytes_rules.push_str(&format!(
                                    "    (@{}, {:?}) => {{ include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\")) }};\n",
                                    dir_name, file_name, rel_to_manifest
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    let generated_code = format!(
        "#[macro_export]\nmacro_rules! include_root_str {{\n{}}}\n\n#[macro_export]\nmacro_rules! include_root_bytes {{\n{}}}\n",
        str_rules, bytes_rules
    );

    let dest_path = Path::new(&out_dir).join("generated_root_macros.rs");
    fs::write(dest_path, generated_code).expect("Failed to write generated_root_macros.rs");
}

fn parse_scannable_config(file: &Path) -> (String, String) {
    let mut inclusion_type = "both".to_string();
    let mut search_mode = "recursive".to_string();

    if let Ok(content) = fs::read_to_string(file) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("inclusion_type=") {
                inclusion_type = line
                    .trim_start_matches("inclusion_type=")
                    .trim()
                    .to_string();
            } else if line.starts_with("search_mode=") {
                search_mode = line.trim_start_matches("search_mode=").trim().to_string();
            }
        }
    }

    (inclusion_type, search_mode)
}

fn collect_files(dir: &Path, recursive: bool, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if path.file_name().and_then(|s| s.to_str()) != Some(".scannable") {
                    files.push(path);
                }
            } else if recursive && path.is_dir() {
                collect_files(&path, recursive, files);
            }
        }
    }
}
