use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header::HeaderName},
    response::{IntoResponse, Response},
};

use crate::domain::http::api_v0_shared_minecraft_maps_zip_res_not_found::ApiV0SharedMinecraftMapsZipResNotFound;
use crate::services::shared_content::minecraft_maps_service::MinecraftMapsService;
use crate::services::shared_content::minecraft_maps_zips_service::MinecraftMapsZipsService;

#[derive(Clone)]
pub struct ApiV0SharedMinecraftMapsZipHandlerState {
    pub minecraft_maps: Arc<MinecraftMapsService>,
    pub minecraft_zips: Arc<MinecraftMapsZipsService>,
}

/// `GET /api/v0/shared/minecraft/maps/{slug}/zip`
///
/// Returns an `X-Accel-Redirect` response for a published map archive. Nginx
/// can use that header to serve the file without copying it through Rust.
pub async fn api_v0_shared_minecraft_maps_zips(
    State(state): State<ApiV0SharedMinecraftMapsZipHandlerState>,
    Path(slug): Path<String>,
) -> Response {
    match state.minecraft_maps.get_minecraft_map_public(&slug) {
        Some(_) => match state.minecraft_zips.get_minecraft_map_zip(&slug) {
            Some(path_to_zip) => {
                let path = path_to_zip.to_string_lossy();
                match HeaderValue::from_str(&path) {
                    Ok(path) => (
                        StatusCode::OK,
                        [(HeaderName::from_static("x-accel-redirect"), path)],
                    )
                        .into_response(),
                    Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                }
            }
            None => not_found(&slug),
        },
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiV0SharedMinecraftMapsZipResNotFound::new(&slug)),
        )
            .into_response(),
    }
}

fn not_found(slug: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ApiV0SharedMinecraftMapsZipResNotFound::new(slug)),
    )
        .into_response()
}
