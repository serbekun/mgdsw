use std::sync::Arc;

use axum::{Json, extract::State};

use crate::services::shared_content::minecraft_maps_service::MinecraftMapsService;

/// `GET /api/v0/shared/minecraft/maps/list`
///
/// Returns a JSON array with the slugs of all **published** Minecraft maps.
/// The list is the cached snapshot built by [`MinecraftMapsService`], so
/// unpublished maps are never exposed to outside callers.
pub async fn api_v0_shared_minecraft_maps_list(
    State(service): State<Arc<MinecraftMapsService>>,
) -> Json<Vec<String>> {
    Json(service.get_minecraft_maps_public_list().as_ref().clone())
}
