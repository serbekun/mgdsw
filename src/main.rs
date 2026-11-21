mod build_info;
mod domain;
mod http;
#[allow(dead_code)]
mod resources;

use std::collections::HashMap;
use std::process::ExitCode;

use tracing::{error, info};
use tracing_subscriber;

use resources::{
    DEFAULT_MAX_CACHEABLE_SIZE, ResourceRoots, ResourceType, ResourcesBasePath, ResourcesCache,
    ResourcesLoader, ResourcesService, ResourcesUnpacker,
};

use crate::http::init_http::init_http;

/// Runtime data directory, matching `cleanup.sh` and `config.default.yaml`.
const DATA_DIR: &str = "data";

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt::init();

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error!("mgdsw: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    info!("mgdsw: {}", build_info::version());

    let roots = ResourceRoots::new(
        format!("{DATA_DIR}/static"),
        format!("{DATA_DIR}/content"),
        format!("{DATA_DIR}/shared"),
        format!("{DATA_DIR}/examples"),
    );

    unpack_resources(&roots)?;
    
    let svr = init_http();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    info!("Listening on http://{}", listener.local_addr()?);
    axum::serve(listener, svr).await?;

    Ok(())
}

/// Creates the runtime roots and copies any packaged templates that are missing
/// or new in this build version.
fn unpack_resources(roots: &ResourceRoots) -> Result<(), Box<dyn std::error::Error>> {
    let loader = ResourcesLoader::new(&roots.static_root);
    let unpacker = ResourcesUnpacker::new(loader, build_info::version());

    if unpacker.unpack(roots)? {
        info!(
            "Unpacked resources into {} (version {})",
            roots.static_root.display(),
            build_info::version()
        );
    } else {
        info!(
            "Resources already unpacked at {} (version {})",
            roots.static_root.display(),
            build_info::version()
        );
    }

    Ok(())
}
