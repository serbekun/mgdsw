use axum::{Json, http::StatusCode};

use crate::{build_info, domain::http::api_v0_version_res::ApiV0VersionRes};

pub async fn api_v0_version() -> (StatusCode, Json<ApiV0VersionRes>) {
    let api_v0_version_res = ApiV0VersionRes {
        version: build_info::version().to_string(),
    };
    (StatusCode::OK, Json(api_v0_version_res))
}
