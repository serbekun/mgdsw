mod build_info;
mod config;
mod domain;
mod http;
#[allow(dead_code)]
mod resources;
#[allow(dead_code)]
mod services;

use std::process::ExitCode;
use std::sync::Arc;

use tracing::{error, info};

use resources::{ResourceRoots, ResourcesLoader, ResourcesUnpacker};

use crate::config::ConfigLoader;
use crate::domain::shared_content::minecraft_maps::MinecraftMaps;
use crate::domain::shared_content::minecraft_maps_zips::MinecraftMapsZips;
use crate::http::init_http::init_http;
use crate::services::shared_content::minecraft_maps_service::MinecraftMapsService;
use crate::services::shared_content::minecraft_maps_zips_service::MinecraftMapsZipsService;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt::init();

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error!("mgdsc: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    info!("mgdsc: {}", build_info::version());

    let config = ConfigLoader::new().load_from_env()?;

    let roots = ResourceRoots::new(
        &config.resources.static_base_path,
        &config.resources.content_base_path,
        &config.resources.shared_base_path,
        &config.resources.examples_base_path,
    );

    unpack_resources(&roots)?;

    let minecraft_maps = Arc::new(MinecraftMaps::new(&roots.content_root));
    let minecraft_maps_service = Arc::new(MinecraftMapsService::new(minecraft_maps));
    let minecraft_maps_zips = Arc::new(MinecraftMapsZips::new(&roots.shared_root));
    let minecraft_maps_zips_service = Arc::new(MinecraftMapsZipsService::new(minecraft_maps_zips));

    let svr = init_http(minecraft_maps_service, minecraft_maps_zips_service);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", config.http.port)).await?;
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
