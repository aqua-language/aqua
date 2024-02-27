pub fn print() {
    mod build {
        include!(concat!(env!("OUT_DIR"), "/built.rs"));
    }
    println!(" version: {}", build::PKG_VERSION);
    println!("  target: {}", build::TARGET);
    println!(" profile: {}", build::PROFILE);
    println!("compiler: {}", build::RUSTC_VERSION);
    println!("    time: {}", build::BUILT_TIME_UTC);
}
