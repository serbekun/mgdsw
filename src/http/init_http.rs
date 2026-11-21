use axum::{Router, routing::get};

use crate::http::handles::api_v0_health;

pub fn init_http() -> Router {
    Router::<()>::new().route("/api/v0/health", get(api_v0_health::api_v0_health))
}
