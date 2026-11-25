use serde::Serialize;

/// Error body returned when a requested Minecraft map is unknown or not
/// published.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ApiV0SharedMinecraftMapsZipResNotFound {
    pub error: String,
}

impl ApiV0SharedMinecraftMapsZipResNotFound {
    /// Builds the standard `404` body for a slug.
    pub fn new(slug: &str) -> Self {
        Self {
            error: format!("slug: {slug}. not found"),
        }
    }
}
