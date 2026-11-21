use chrono::NaiveDate;
use serde::Serialize;

use crate::domain::shared_content::minecraft_map::MinecraftMap;

/// Public representation of a Minecraft map.
///
/// A copy of [`MinecraftMap`] without the `published` field: that flag is only
/// needed by the server to decide whether a zip/YAML may be served. Serialized
/// with `snake_case` field names for the public API.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiV0SharedMinecraftMapsYamlRes {
    /// Value used to build the download URL. Must be unique.
    pub slug: String,
    /// Map name shown on the page.
    pub name: String,
    /// Map authors.
    pub authors: Vec<String>,
    /// Map description.
    pub description: String,
    /// Map license.
    pub license: String,
    /// Minecraft map version.
    pub map_version: String,
    /// Date when the map was released.
    pub release_date: NaiveDate,
    /// Minecraft version the map was created for.
    pub minecraft_version: String,
    /// If `true`, the map is visible on the site but the zip cannot be
    /// downloaded.
    pub preview: bool,
    /// SHA-256 checksum of the map zip (64 hex characters).
    pub zip_sha256: String,
}

impl From<&MinecraftMap> for ApiV0SharedMinecraftMapsYamlRes {
    fn from(map: &MinecraftMap) -> Self {
        Self {
            slug: map.slug().to_owned(),
            name: map.name().to_owned(),
            authors: map.authors().to_vec(),
            description: map.description().to_owned(),
            license: map.license().to_owned(),
            map_version: map.map_version().to_owned(),
            release_date: map.release_date(),
            minecraft_version: map.minecraft_version().to_owned(),
            preview: map.preview(),
            zip_sha256: map.zip_sha256().to_owned(),
        }
    }
}
