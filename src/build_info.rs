pub const VERSION: &str = match option_env!("MGDSW_BUILD_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

/// Returns the version of the build.
pub fn version() -> &'static str {
    VERSION
}