fn main() {
    println!("cargo:rerun-if-changed=VERSION");
    println!("cargo:rerun-if-env-changed=MGDSW_VERSION");

    let version = std::env::var("MGDSW_VERSION")
        .ok()
        .or_else(|| std::fs::read_to_string("VERSION").ok())
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").unwrap());

    println!("cargo:rustc-env=MGDSW_BUILD_VERSION={version}");
}
