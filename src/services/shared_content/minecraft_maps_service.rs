use std::sync::{Arc, Mutex, RwLock};

use tracing::info;

use crate::domain::http::api_v0_shared_minecraft_maps_yaml_res::ApiV0SharedMinecraftMapsYamlRes;
use crate::domain::shared_content::minecraft_maps::MinecraftMaps;

/// Public API and business rules for Minecraft map metadata.
///
/// Disk access, slug listing and YAML parsing are delegated to
/// [`MinecraftMaps`]. This service decides which maps are visible to the
/// outside world and converts stored objects into public response objects.
pub struct MinecraftMapsService {
    minecraft_maps: Arc<MinecraftMaps>,
    /// Cached snapshot of the slugs of published maps.
    ///
    /// The snapshot is always replaced as a whole, never modified in place, so
    /// readers see either the previous or the new list and never a partially
    /// built one.
    public_slugs: RwLock<Arc<Vec<String>>>,
    /// Serializes rebuilds so two concurrent scans cannot publish their
    /// results out of order.
    rebuild_lock: Mutex<()>,
}

impl MinecraftMapsService {
    /// Creates a service backed by an existing map store and immediately builds
    /// the cached list of published slugs, so
    /// [`Self::get_minecraft_maps_public_list`] is usable right after
    /// construction.
    ///
    /// Building the list reads and parses the YAML of every map, so the
    /// construction cost grows with the number of maps.
    pub fn new(minecraft_maps: Arc<MinecraftMaps>) -> Self {
        let service = Self {
            minecraft_maps,
            public_slugs: RwLock::new(Arc::new(Vec::new())),
            rebuild_lock: Mutex::new(()),
        };
        service.build_public_slug_list();
        service
    }

    /// Rebuilds the cached list of published slugs from the store.
    ///
    /// Walks every slug returned by [`MinecraftMaps::slugs`], keeps only the
    /// maps that exist and are published, and atomically replaces the cached
    /// snapshot with the result.
    fn build_public_slug_list(&self) {
        let public_slugs: Vec<String> = self
            .minecraft_maps
            .slugs()
            .into_iter()
            .filter(|slug| {
                self.minecraft_maps
                    .get_by_slug(slug)
                    .is_some_and(|map| map.published())
            })
            .collect();

        *self.public_slugs.write().unwrap_or_else(|e| e.into_inner()) = Arc::new(public_slugs);
    }

    // region public api

    /// Rescans the content root and refreshes the cached list of published
    /// slugs.
    ///
    /// First asks the underlying [`MinecraftMaps`] to rebuild its own slug list,
    /// then rebuilds the public snapshot from it. Rebuilds are serialized so
    /// concurrent callers cannot publish their results out of order; readers of
    /// [`Self::get_minecraft_maps_public_list`] are not blocked and keep seeing
    /// the previous snapshot until the new one is ready.
    pub fn rebuild_minecraft_maps_slug_list(&self) {
        let _guard = self.rebuild_lock.lock().unwrap_or_else(|e| e.into_inner());
        self.minecraft_maps.rebuild_slug_list();
        self.build_public_slug_list();
    }

    /// Returns the public representation of a map if it exists and is
    /// published.
    ///
    /// Returns `None` when the map does not exist or is not published, so the
    /// HTTP layer can answer with `404`.
    pub fn get_minecraft_map_public(&self, slug: &str) -> Option<ApiV0SharedMinecraftMapsYamlRes> {
        let minecraft_map = self.minecraft_maps.get_by_slug(slug)?;


        // If the map is not published, it must not be visible from outside.
        if !minecraft_map.published() {
            info!("Minecraft map: '{slug}' not published");
            return None;
        }

        Some(ApiV0SharedMinecraftMapsYamlRes::from(&minecraft_map))
    }

    /// All map slugs found in the content root, **including unpublished maps**.
    ///
    /// This is the raw list as returned by the store and is not filtered by
    /// visibility, so it must not be exposed to outside callers. Use
    /// [`Self::get_minecraft_maps_public_list`] for the list that is safe to
    /// publish.
    pub fn get_minecraft_maps_list(&self) -> Vec<String> {
        self.minecraft_maps.slugs()
    }

    /// Slug list exposed to the public API, i.e. the maps that are published
    /// and therefore visible from outside.
    ///
    /// The returned vector is a cached snapshot built at construction time and
    /// refreshed by [`Self::rebuild_minecraft_maps_slug_list`].
    pub fn get_minecraft_maps_public_list(&self) -> Arc<Vec<String>> {
        Arc::clone(&self.public_slugs.read().unwrap_or_else(|e| e.into_inner()))
    }

    // endregion
}

#[cfg(test)]
mod tests {
    use super::MinecraftMapsService;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use crate::domain::shared_content::minecraft_maps::MinecraftMaps;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let unique = format!(
                "mgdsw-minecraft-maps-service-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            let path = std::env::temp_dir().join(unique);
            fs::create_dir_all(path.join("minecraft_maps")).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_map(content_root: &Path, slug: &str, published: bool) {
        let yaml = format!(
            r#"slug: "{slug}"
name: "Map {slug}"
authors:
  - "Author"
description: "Description"
license: "CC0"
mapVersion: "1.0.0"
releaseDate: 2026-01-01
minecraftVersion: "1.21.4"
published: {published}
preview: true
zipSha256: "0000000000000000000000000000000000000000000000000000000000000000"
"#
        );
        fs::write(
            content_root
                .join("minecraft_maps")
                .join(format!("{slug}.yaml")),
            yaml,
        )
        .unwrap();
    }

    fn service(content_root: &Path) -> MinecraftMapsService {
        MinecraftMapsService::new(Arc::new(MinecraftMaps::new(content_root)))
    }

    #[test]
    fn public_list_contains_only_published_maps() {
        let dir = TempDir::new();
        write_map(dir.path(), "alpha", true);
        write_map(dir.path(), "beta", false);

        let service = service(dir.path());

        assert_eq!(service.get_minecraft_maps_list(), ["alpha", "beta"]);
        assert_eq!(*service.get_minecraft_maps_public_list(), ["alpha"]);
    }

    #[test]
    fn public_map_is_returned_without_published_flag() {
        let dir = TempDir::new();
        write_map(dir.path(), "alpha", true);

        let map = service(dir.path())
            .get_minecraft_map_public("alpha")
            .expect("published map should be public");

        assert_eq!(map.slug, "alpha");
        assert_eq!(map.name, "Map alpha");
        assert_eq!(map.map_version, "1.0.0");
        assert_eq!(map.release_date.to_string(), "2026-01-01");
        assert!(map.preview);
    }

    #[test]
    fn unpublished_and_unknown_maps_are_not_public() {
        let dir = TempDir::new();
        write_map(dir.path(), "hidden", false);
        let service = service(dir.path());

        assert!(service.get_minecraft_map_public("hidden").is_none());
        assert!(service.get_minecraft_map_public("unknown").is_none());
    }

    #[test]
    fn rebuild_refreshes_the_public_snapshot() {
        let dir = TempDir::new();
        write_map(dir.path(), "alpha", true);
        let service = service(dir.path());
        assert_eq!(*service.get_minecraft_maps_public_list(), ["alpha"]);

        write_map(dir.path(), "beta", true);
        service.rebuild_minecraft_maps_slug_list();

        assert_eq!(*service.get_minecraft_maps_public_list(), ["alpha", "beta"]);
    }
}
