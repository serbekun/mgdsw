use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use crate::domain::shared_content::minecraft_maps_zips::MinecraftMapsZips;

/// Business logic for downloadable Minecraft map archives.
///
/// Filesystem access is delegated to [`MinecraftMapsZips`]. The service keeps
/// an atomically replaceable snapshot of the available archive slugs and
/// serializes rescans, matching the behaviour of [`MinecraftMapsService`].
///
/// [`MinecraftMapsService`]: crate::services::shared_content::minecraft_maps_service::MinecraftMapsService
pub struct MinecraftMapsZipsService {
    minecraft_maps_zips: Arc<MinecraftMapsZips>,
    slugs: RwLock<Arc<Vec<String>>>,
    rebuild_lock: Mutex<()>,
}

impl MinecraftMapsZipsService {
    /// Creates a service and builds the initial archive slug snapshot.
    pub fn new(minecraft_maps_zips: Arc<MinecraftMapsZips>) -> Self {
        let service = Self {
            minecraft_maps_zips,
            slugs: RwLock::new(Arc::new(Vec::new())),
            rebuild_lock: Mutex::new(()),
        };
        service.build_slug_list();
        service
    }

    fn build_slug_list(&self) {
        *self
            .slugs
            .write()
            .unwrap_or_else(|error| error.into_inner()) =
            Arc::new(self.minecraft_maps_zips.slugs());
    }

    /// Rescans the archive directory and refreshes the cached slug snapshot.
    pub fn rebuild_minecraft_maps_zips_slug_list(&self) {
        let _guard = self
            .rebuild_lock
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        self.minecraft_maps_zips.rebuild_slug_list();
        self.build_slug_list();
    }

    /// Returns the path to a map archive if it exists.
    pub fn get_minecraft_map_zip(&self, slug: &str) -> Option<PathBuf> {
        self.minecraft_maps_zips.get_by_slug(slug)
    }

    /// Returns the cached list of all available archive slugs.
    pub fn get_minecraft_maps_zips_list(&self) -> Arc<Vec<String>> {
        Arc::clone(&self.slugs.read().unwrap_or_else(|error| error.into_inner()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use super::MinecraftMapsZipsService;
    use crate::domain::shared_content::minecraft_maps_zips::MinecraftMapsZips;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "mgdsw-minecraft-maps-zips-service-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(path.join("minecraft_maps_zips")).unwrap();
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

    fn service(shared_root: &Path) -> MinecraftMapsZipsService {
        MinecraftMapsZipsService::new(Arc::new(MinecraftMapsZips::new(shared_root)))
    }

    #[test]
    fn returns_archive_path_and_cached_list() {
        let dir = TempDir::new();
        fs::write(dir.path().join("minecraft_maps_zips/alpha.zip"), []).unwrap();
        fs::write(dir.path().join("minecraft_maps_zips/beta.zip"), []).unwrap();

        let service = service(dir.path());

        assert_eq!(*service.get_minecraft_maps_zips_list(), ["alpha", "beta"]);
        assert_eq!(
            service.get_minecraft_map_zip("alpha"),
            Some(dir.path().join("minecraft_maps_zips/alpha.zip"))
        );
        assert!(service.get_minecraft_map_zip("missing").is_none());
    }

    #[test]
    fn rebuild_refreshes_cached_list() {
        let dir = TempDir::new();
        let service = service(dir.path());
        assert!(service.get_minecraft_maps_zips_list().is_empty());

        fs::write(dir.path().join("minecraft_maps_zips/late.zip"), []).unwrap();
        service.rebuild_minecraft_maps_zips_slug_list();

        assert_eq!(*service.get_minecraft_maps_zips_list(), ["late"]);
    }
}
