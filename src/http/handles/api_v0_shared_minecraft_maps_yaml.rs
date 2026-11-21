use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::info;

use crate::domain::http::api_v0_shared_minecraft_maps_yaml_res_not_found::ApiV0SharedMinecraftMapsYamlResNotFound;
use crate::services::shared_content::minecraft_maps_service::MinecraftMapsService;

/// `GET /api/v0/shared/minecraft/maps/{slug}/yaml`
///
/// Returns the public metadata of a published map, or `404 Not Found` when the
/// map does not exist or is not published.
pub async fn api_v0_shared_minecraft_maps_yaml(
    State(service): State<Arc<MinecraftMapsService>>,
    Path(slug): Path<String>,
) -> Response {
    match service.get_minecraft_map_public(&slug) {
        Some(response) => (StatusCode::OK, Json(response)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiV0SharedMinecraftMapsYamlResNotFound::new(&slug)),
        )
            .into_response(),
    }
}
