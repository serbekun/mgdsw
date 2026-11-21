use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Metadata of a single Minecraft map stored on disk.
///
/// The structure mirrors the YAML file layout (`camelCase` keys), so it is used
/// both as the deserialization target for `data/content/minecraft_maps/*.yaml`
/// and as the internal domain object. The public HTTP representation lives in
/// [`crate::domain::http::api_v0_shared_minecraft_maps_yaml_res`] and
/// deliberately omits the `published` flag.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftMap {
    /// Value used to build the download URL. Must be unique.
    slug: String,
    /// Map name shown on the page.
    name: String,
    /// Map authors.
    authors: Vec<String>,
    /// Map description.
    description: String,
    /// Map license.
    license: String,
    /// Minecraft map version.
    map_version: String,
    /// Date when the map was released.
    release_date: NaiveDate,
    /// Minecraft version the map was created for.
    minecraft_version: String,
    /// If `true`, the map is visible on the site and the zip can be downloaded.
    published: bool,
    /// If `true`, the map is visible on the site but the zip cannot be
    /// downloaded.
    preview: bool,
    /// SHA-256 checksum of the map zip (64 hex characters).
    zip_sha256: String,
}

impl MinecraftMap {
    /// Creates a map from its stored metadata.
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        slug: String,
        name: String,
        authors: Vec<String>,
        description: String,
        license: String,
        map_version: String,
        release_date: NaiveDate,
        minecraft_version: String,
        published: bool,
        preview: bool,
        zip_sha256: String,
    ) -> Self {
        Self {
            slug,
            name,
            authors,
            description,
            license,
            map_version,
            release_date,
            minecraft_version,
            published,
            preview,
            zip_sha256,
        }
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn authors(&self) -> &[String] {
        &self.authors
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn license(&self) -> &str {
        &self.license
    }

    pub fn map_version(&self) -> &str {
        &self.map_version
    }

    pub fn release_date(&self) -> NaiveDate {
        self.release_date
    }

    pub fn minecraft_version(&self) -> &str {
        &self.minecraft_version
    }

    pub fn published(&self) -> bool {
        self.published
    }

    pub fn preview(&self) -> bool {
        self.preview
    }

    pub fn zip_sha256(&self) -> &str {
        &self.zip_sha256
    }
}
