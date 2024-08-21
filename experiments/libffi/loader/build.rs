use std::env;
use std::process::Command;

fn main() {
    let library_dir = env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("library");

    println!("cargo:rerun-if-changed={}", library_dir.display());

    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(library_dir.join("Cargo.toml"))
        .status()
        .unwrap();

    if !status.success() {
        panic!("Failed to build the crate in the parent directory");
    }
}
