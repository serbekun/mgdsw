use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use tracing::{error, warn};

use crate::resources::resources_base_path::ResourcesBasePath;

const ZIP_EXTENSION: &str = ".zip";

/// Access to downloadable Minecraft map archives stored on disk.
///
/// Archives are expected at `shared_root/minecraft_maps_zips/<slug>.zip`.
/// The slug is always validated before it is used to construct a path, so a
/// caller cannot use this type to access a file outside the archive directory.
pub struct MinecraftMapsZips {
    root: PathBuf,
    slugs: RwLock<Vec<String>>,
}

impl MinecraftMapsZips {
    /// Creates an archive store and scans the shared root immediately.
    pub fn new(shared_root: impl AsRef<Path>) -> Self {
        let shared_root = shared_root.as_ref();
        let shared_root = if shared_root.is_absolute() {
            shared_root.to_owned()
        } else {
            std::env::current_dir()
                .expect("current working directory must be available")
                .join(shared_root)
        };
        let root = shared_root.join(ResourcesBasePath::MinecraftMapsZips.base_path());
        let slugs = Self::scan_slugs(&root);

        Self {
            root,
            slugs: RwLock::new(slugs),
        }
    }

    /// Rescans the archive directory and replaces the cached slug list.
    pub fn rebuild_slug_list(&self) {
        let slugs = Self::scan_slugs(&self.root);
        *self
            .slugs
            .write()
            .unwrap_or_else(|error| error.into_inner()) = slugs;
    }

    /// Returns all archive slugs in alphabetical order.
    pub fn slugs(&self) -> Vec<String> {
        self.slugs
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    /// Returns the path to `<slug>.zip`, if that archive exists.
    pub fn get_by_slug(&self, slug: &str) -> Option<PathBuf> {
        if slug.is_empty()
            || ResourcesBasePath::MinecraftMapsZips
                .resolve(&format!("{slug}{ZIP_EXTENSION}"))
                .is_err()
        {
            return None;
        }

        let path = self.root.join(format!("{slug}{ZIP_EXTENSION}"));
        (path.starts_with(&self.root) && path.is_file()).then_some(path)
    }

    /// Alias describing explicitly that the result is a filesystem path.
    #[allow(dead_code)]
    pub fn path_by_slug(&self, slug: &str) -> Option<PathBuf> {
        self.get_by_slug(slug)
    }

    fn scan_slugs(root: &Path) -> Vec<String> {
        if !root.is_dir() {
            warn!(
                "Minecraft maps zips directory does not exist: {}",
                root.display()
            );
            return Vec::new();
        }

        let entries = match fs::read_dir(root) {
            Ok(entries) => entries,
            Err(error) => {
                error!(
                    "Failed to list Minecraft maps zips in {}: {error}",
                    root.display()
                );
                return Vec::new();
            }
        };

        let mut slugs: Vec<String> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                (path.is_file() && path.extension().is_some_and(|ext| ext == "zip"))
                    .then(|| path.file_stem()?.to_str().map(str::to_owned))
                    .flatten()
            })
            .collect();
        slugs.sort_unstable();
        slugs.dedup();
        slugs
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::MinecraftMapsZips;

    #[test]
    fn lists_zip_slugs_and_resolves_paths() {
        let dir = tempfile_dir();
        let zips = dir.path().join("minecraft_maps_zips");
        fs::create_dir_all(&zips).unwrap();
        fs::write(zips.join("zeta.zip"), []).unwrap();
        fs::write(zips.join("alpha.zip"), []).unwrap();
        fs::write(zips.join("notes.txt"), []).unwrap();

        let maps = MinecraftMapsZips::new(dir.path());

        assert_eq!(maps.slugs(), ["alpha", "zeta"]);
        assert_eq!(maps.get_by_slug("alpha"), Some(zips.join("alpha.zip")));
        assert!(maps.get_by_slug("../alpha").is_none());
    }

    #[test]
    fn rebuilds_after_an_archive_is_added() {
        let dir = tempfile_dir();
        let zips = dir.path().join("minecraft_maps_zips");
        fs::create_dir_all(&zips).unwrap();
        let maps = MinecraftMapsZips::new(dir.path());

        fs::write(zips.join("late.zip"), []).unwrap();
        assert!(maps.slugs().is_empty());
        maps.rebuild_slug_list();
        assert_eq!(maps.slugs(), ["late"]);
    }

    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn tempfile_dir() -> TempDir {
        let path = std::env::temp_dir().join(format!(
            "mgdsc-minecraft-maps-zips-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }
}
