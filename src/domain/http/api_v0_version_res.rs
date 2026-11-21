use serde::Serialize;

#[derive(Serialize)]
pub struct ApiV0VersionRes {
    pub version: String,
}
