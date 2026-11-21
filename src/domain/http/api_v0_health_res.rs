use serde::Serialize;

#[derive(Serialize)]
pub struct ApiV0HealthRes {
    pub health: String,
}
