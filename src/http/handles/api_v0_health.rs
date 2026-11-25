use axum::{Json, http::StatusCode};

use crate::{domain::http::api_v0_health_res::ApiV0HealthRes};

pub async fn api_v0_health() -> (StatusCode, Json<ApiV0HealthRes>) {
    let api_v0_health_res = ApiV0HealthRes {
        health: "ok".to_string(),
    };
    (StatusCode::OK, Json(api_v0_health_res))
}
