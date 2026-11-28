use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use tracing::{error, info, warn};

use crate::domain::shared_content::minecraft_map::MinecraftMap;
use crate::resources::resources_base_path::ResourcesBasePath;

const YAML_EXTENSION: &str = ".yaml";

/// Access to the Minecraft map metadata stored on disk.
///
/// Owns the content-side concerns: it reads map YAML files from the
/// operator-managed content root, keeps the list of available slugs and parses
/// raw YAML bytes into [`MinecraftMap`] values. It deliberately contains no
/// public API or business rules; those belong to
/// [`crate::services::shared_content::minecraft_maps_service::MinecraftMapsService`].
pub struct MinecraftMaps {
    /// Resolved `content/minecraft_maps/` directory.
    root: PathBuf,
    /// Slugs currently known to the store, sorted alphabetically.
    slugs: RwLock<Vec<String>>,
}

impl MinecraftMaps {
    /// Creates a map store that reads map YAML from
    /// `content_root/minecraft_maps/` and immediately scans it for slugs.
    ///
    /// The scan cost grows with the number of files, but parsing happens only
    /// when a specific map is requested.
    pub fn new(content_root: impl AsRef<Path>) -> Self {
        let root = content_root
            .as_ref()
            .join(ResourcesBasePath::MinecraftMapsYaml.base_path());

        let slugs = Self::scan_slugs(&root);

        Self {
            root,
            slugs: RwLock::new(slugs),
        }
    }

    /// Rescans the content root for `*.yaml` files and atomically replaces the
    /// cached slug list.
    pub fn rebuild_slug_list(&self) {
        let slugs = Self::scan_slugs(&self.root);
        *self.slugs.write().unwrap_or_else(|e| e.into_inner()) = slugs;
    }

    /// All map slugs available in the content root, sorted alphabetically.
    ///
    /// The returned slugs are file names without the `.yaml` extension.
    pub fn slugs(&self) -> Vec<String> {
        self.slugs.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Reads and parses the map metadata for a slug.
    ///
    /// Returns `None` if the slug is empty or invalid, or if the file does not
    /// exist or cannot be parsed.
    pub fn get_by_slug(&self, slug: &str) -> Option<MinecraftMap> {
        if slug.is_empty() {
            return None;
        }

        // Validate the slug before touching the filesystem: rejects traversal
        // sequences (`..`, `/`, `\`, `%`).
        if ResourcesBasePath::MinecraftMapsYaml
            .resolve(&format!("{slug}{YAML_EXTENSION}"))
            .is_err()
        {
            info!("Minecraft map: {slug} has an invalid slug");
            return None;
        }

        let yaml_path = self.root.join(format!("{slug}{YAML_EXTENSION}"));
        if !yaml_path.starts_with(&self.root) || !yaml_path.is_file() {
            info!("Minecraft map: {slug} not found");
            return None;
        }

        match fs::read(&yaml_path) {
            Ok(bytes) => match Self::parse(&bytes) {
                Ok(map) => Some(map),
                Err(error) => {
                    error!("Error parse slug: {slug}. error message: {error}");
                    None
                }
            },
            Err(error) => {
                error!("Error reading slug: {slug}. error message: {error}");
                None
            }
        }
    }

    /// Parses raw YAML bytes into a [`MinecraftMap`].
    pub fn parse(yaml_bytes: &[u8]) -> Result<MinecraftMap, serde_yaml::Error> {
        serde_yaml::from_slice(yaml_bytes)
    }

    /// Lists `*.yaml` file names under `root` and strips the extension.
    ///
    /// A missing or unreadable directory yields an empty list: a map store
    /// without content is valid.
    fn scan_slugs(root: &Path) -> Vec<String> {
        if !root.is_dir() {
            warn!(
                "Minecraft maps directory does not exist: {}",
                root.display()
            );
            return Vec::new();
        }

        let entries = match fs::read_dir(root) {
            Ok(entries) => entries,
            Err(error) => {
                error!(
                    "Failed to list minecraft maps in {}: {error}",
                    root.display()
                );
                return Vec::new();
            }
        };

        let mut slugs: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|t| t.is_file()).unwrap_or(false))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter_map(|name| name.strip_suffix(YAML_EXTENSION).map(str::to_owned))
            .collect();

        slugs.sort_unstable();
        slugs
    }
}

#[cfg(test)]
mod tests {
    use super::MinecraftMaps;
    use std::fs;
    use std::path::{Path, PathBuf};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let unique = format!(
                "mgdsc-minecraft-maps-{}-{}",
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
name: "Test map"
authors:
  - "Author"
description: "Description"
license: "CC0"
mapVersion: "1.0.0"
releaseDate: 2026-01-01
minecraftVersion: "1.21.4"
published: {published}
preview: false
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

    #[test]
    fn lists_yaml_files_as_sorted_slugs() {
        let dir = TempDir::new();
        write_map(dir.path(), "zeta", true);
        write_map(dir.path(), "alpha", true);
        fs::write(dir.path().join("minecraft_maps").join("notes.txt"), "x").unwrap();

        let maps = MinecraftMaps::new(dir.path());
        assert_eq!(maps.slugs(), vec!["alpha".to_owned(), "zeta".to_owned()]);
    }

    #[test]
    fn missing_directory_yields_empty_store() {
        let dir = TempDir::new();
        let maps = MinecraftMaps::new(dir.path().join("does-not-exist"));
        assert!(maps.slugs().is_empty());
        assert!(maps.get_by_slug("anything").is_none());
    }

    #[test]
    fn parses_map_yaml() {
        let dir = TempDir::new();
        write_map(dir.path(), "example-map", true);

        let maps = MinecraftMaps::new(dir.path());
        let map = maps.get_by_slug("example-map").expect("map should parse");

        assert_eq!(map.slug(), "example-map");
        assert_eq!(map.name(), "Test map");
        assert_eq!(map.authors(), ["Author".to_owned()]);
        assert_eq!(map.release_date().to_string(), "2026-01-01");
        assert!(map.published());
        assert!(!map.preview());
    }

    #[test]
    fn rebuild_picks_up_new_files() {
        let dir = TempDir::new();
        let maps = MinecraftMaps::new(dir.path());
        assert!(maps.slugs().is_empty());

        write_map(dir.path(), "late", true);
        maps.rebuild_slug_list();

        assert_eq!(maps.slugs(), vec!["late".to_owned()]);
    }

    #[test]
    fn rejects_unknown_and_invalid_slugs() {
        let dir = TempDir::new();
        write_map(dir.path(), "known", true);
        let maps = MinecraftMaps::new(dir.path());

        assert!(maps.get_by_slug("").is_none());
        assert!(maps.get_by_slug("unknown").is_none());
        assert!(maps.get_by_slug("../known").is_none());
        assert!(maps.get_by_slug("known/../known").is_none());
    }
}
