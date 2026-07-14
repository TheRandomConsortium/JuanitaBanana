use std::process::Command;

fn main() {
    // Only run if the build script itself or the binary doesn't exist
    println!("cargo:rerun-if-changed=scripts/build_hnsd.sh");

    let status = Command::new("./scripts/build_hnsd.sh")
        .status()
        .expect("Failed to execute scripts/build_hnsd.sh");

    if !status.success() {
        panic!("build_hnsd.sh failed with exit code: {:?}", status.code());
    }
}
