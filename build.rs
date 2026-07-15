use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=scripts/build_hnsd.sh");

    if std::path::Path::new("bin/hnsd").exists() {
        return;
    }

    let status = Command::new("bash")
        .arg("./scripts/build_hnsd.sh")
        .status()
        .expect("Failed to execute scripts/build_hnsd.sh");

    if !status.success() {
        panic!("build_hnsd.sh failed with exit code: {:?}", status.code());
    }
}
