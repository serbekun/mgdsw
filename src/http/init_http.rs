use std::sync::Arc;

use axum::{Router, routing::get};

use crate::http::handles::{
    api_v0_health, api_v0_shared_minecraft_maps_list, api_v0_shared_minecraft_maps_yaml,
    api_v0_version,
};
use crate::services::shared_content::minecraft_maps_service::MinecraftMapsService;

/// Builds the HTTP router with the shared application state.
pub fn init_http(minecraft_maps_service: Arc<MinecraftMapsService>) -> Router {
    Router::new()
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
        .with_state(minecraft_maps_service)
}
