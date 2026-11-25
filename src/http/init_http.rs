use std::sync::Arc;

use axum::{Router, routing::get};

use crate::http::handles::api_v0_shared_minecraft_maps_zip::ApiV0SharedMinecraftMapsZipHandlerState;
use crate::http::handles::{
    api_v0_health, api_v0_shared_minecraft_maps_list, api_v0_shared_minecraft_maps_yaml,
    api_v0_shared_minecraft_maps_zip, api_v0_version,
};
use crate::services::shared_content::minecraft_maps_service::MinecraftMapsService;
use crate::services::shared_content::minecraft_maps_zips_service::MinecraftMapsZipsService;

/// Builds the HTTP router with the shared application state.
pub fn init_http(
    minecraft_maps_service: Arc<MinecraftMapsService>,
    minecraft_maps_zips_service: Arc<MinecraftMapsZipsService>,
) -> Router {
    let maps_router = Router::new()
        .route("/api/v0/health", get(api_v0_health::api_v0_health))
        .route("/api/v0/version", get(api_v0_version::api_v0_version))
        .route(
            "/api/v0/shared/minecraft/maps/list",
            get(api_v0_shared_minecraft_maps_list::api_v0_shared_minecraft_maps_list),
        )
        .route(
            "/api/v0/shared/minecraft/maps/{slug}/yaml",
            get(api_v0_shared_minecraft_maps_yaml::api_v0_shared_minecraft_maps_yaml),
        )
        .with_state(Arc::clone(&minecraft_maps_service));

    let zip_router = Router::new()
        .route(
            "/api/v0/shared/minecraft/maps/{slug}/zip",
            get(api_v0_shared_minecraft_maps_zip::api_v0_shared_minecraft_maps_zips),
        )
        .with_state(ApiV0SharedMinecraftMapsZipHandlerState {
            minecraft_maps: minecraft_maps_service,
            minecraft_zips: minecraft_maps_zips_service,
        });

    maps_router.merge(zip_router)
}
