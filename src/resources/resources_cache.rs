use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::cache_policy::CachePolicy;
use super::error::ResourcesError;
use super::etags;
use super::lookup_order::LookupOrder;
use super::resource_type::ResourceType;
use super::resources_loader::ResourcesLoader;

/// Default upper bound (100 MiB) for a resource to be eligible for caching.
pub const DEFAULT_MAX_CACHEABLE_SIZE: u64 = 100 * 1024 * 1024;

/// A loaded resource: its bytes and, for cacheable types, the ETag computed
/// once at load time. The bytes are shared with every caller and must not be
/// modified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub data: Arc<Vec<u8>>,
    /// The quoted ETag value, or `None` for [`ResourceType::Shared`] entries,
    /// which are never cached and never hashed.
    pub etag: Option<String>,
}

/// Cache key that keeps equal relative paths in different roots apart.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key {
    resource_type: ResourceType,
    path: String,
}

/// Thread-safe cache layer for resources, keyed by [`ResourceType`] and path.
///
/// The resource type is part of the contract, not just metadata:
///
/// - [`ResourceType::Static`] is read from the static root with the usual
///   embedded fallback and may be cached;
/// - [`ResourceType::Content`] is never served by this class. Content is
///   configuration data owned by its own services, not an HTTP resource;
/// - [`ResourceType::Examples`] is never served either: example files are
///   copied out for the operator by the unpacker;
/// - [`ResourceType::Shared`] is read from the shared root and is **never
///   cached**, even when the caller asks for [`CachePolicy::Cache`]. Shared
///   files are large and are served by nginx, so retaining their bytes would be
///   a bug.
///
/// Only bytes are cached — callers answer with bytes, so keeping a decoded copy
/// would both duplicate memory and re-encode on every response. Text is decoded
/// on demand, always as UTF-8.
///
/// Misses are cached too (for cacheable types): a bot scanning for files that
/// do not exist must not reach disk and the embedded tree on every request.
#[derive(Debug)]
pub struct ResourcesCache {
    loaders: HashMap<ResourceType, ResourcesLoader>,
    max_cacheable_size: u64,
    cache: RwLock<HashMap<Key, Option<Entry>>>,
}

impl ResourcesCache {
    /// Creates a cache with an explicit root per served resource type.
    ///
    /// A [`ResourceType::Static`] root is required. A [`ResourceType::Shared`]
    /// root is optional: when it is absent, shared lookups fail with a clear
    /// error instead of silently reading from static. `Content` and `Examples`
    /// roots are deliberately not used because those types are not served.
    pub fn new(
        roots: HashMap<ResourceType, PathBuf>,
        max_cacheable_size: u64,
    ) -> Result<Self, ResourcesError> {
        let Some(static_root) = roots.get(&ResourceType::Static) else {
            return Err(ResourcesError::MissingStaticRoot);
        };

        if let Some(shared_root) = roots.get(&ResourceType::Shared)
            && (shared_root == static_root
                || shared_root.starts_with(static_root)
                || static_root.starts_with(shared_root))
        {
            return Err(ResourcesError::NestedRoots);
        }

        let mut loaders = HashMap::new();
        loaders.insert(
            ResourceType::Static,
            ResourcesLoader::with_order(static_root, LookupOrder::DiskFirst),
        );
        if let Some(shared_root) = roots.get(&ResourceType::Shared) {
            loaders.insert(
                ResourceType::Shared,
                ResourcesLoader::with_order(shared_root, LookupOrder::DiskOnly),
            );
        }

        Ok(Self {
            loaders,
            max_cacheable_size,
            cache: RwLock::new(HashMap::new()),
        })
    }

    /// Gets a resource entry, honouring the type's cache rules and the given
    /// policy.
    ///
    /// Returns `Ok(None)` when the resource does not exist or the path is
    /// empty, and an error for `Content`/`Examples` and unconfigured roots.
    pub fn get(
        &self,
        resource_type: ResourceType,
        path: &str,
        policy: CachePolicy,
    ) -> Result<Option<Entry>, ResourcesError> {
        if matches!(
            resource_type,
            ResourceType::Content | ResourceType::Examples
        ) {
            return Err(ResourcesError::NotServable(resource_type));
        }
        if path.is_empty() {
            return Ok(None);
        }

        let cacheable = resource_type != ResourceType::Shared && policy == CachePolicy::Cache;
        let key = Key {
            resource_type,
            path: path.to_owned(),
        };

        if cacheable && let Some(entry) = self.read_cache().get(&key) {
            return Ok(entry.clone());
        }

        // Loading happens outside the write lock so blocking I/O never runs
        // while the cache is locked.
        let Some(data) = self.loader(resource_type)?.load_binary(path) else {
            if cacheable {
                self.write_cache().insert(key, None);
            }
            return Ok(None);
        };

        let data_len = data.len() as u64;
        let entry = Entry {
            // Shared bytes are never retained and never hashed: hashing a large
            // archive on every lookup would defeat the point of not caching it.
            etag: if resource_type == ResourceType::Shared {
                None
            } else {
                Some(etags::of(&data))
            },
            data: Arc::new(data),
        };

        if cacheable && data_len <= self.max_cacheable_size {
            self.write_cache().insert(key, Some(entry.clone()));
        }

        Ok(Some(entry))
    }

    /// Gets binary data, honouring the type's cache rules and the given policy.
    pub fn get_binary(
        &self,
        resource_type: ResourceType,
        path: &str,
        policy: CachePolicy,
    ) -> Result<Option<Vec<u8>>, ResourcesError> {
        Ok(self
            .get(resource_type, path, policy)?
            .map(|entry| entry.data.as_ref().clone()))
    }

    /// Gets the resource decoded as UTF-8 text.
    pub fn get_text(
        &self,
        resource_type: ResourceType,
        path: &str,
        policy: CachePolicy,
    ) -> Result<Option<String>, ResourcesError> {
        Ok(self
            .get_binary(resource_type, path, policy)?
            .map(|data| String::from_utf8_lossy(&data).into_owned()))
    }

    /// Checks whether a resource exists without retaining its bytes for
    /// [`ResourceType::Shared`].
    pub fn exists(&self, resource_type: ResourceType, path: &str) -> Result<bool, ResourcesError> {
        if matches!(
            resource_type,
            ResourceType::Content | ResourceType::Examples
        ) {
            return Err(ResourcesError::NotServable(resource_type));
        }
        if path.is_empty() {
            return Ok(false);
        }
        if resource_type == ResourceType::Shared {
            return Ok(self.loader(resource_type)?.exists(path));
        }
        Ok(self.get(resource_type, path, CachePolicy::Cache)?.is_some())
    }

    /// Returns the resources available under the given directory of a type.
    pub fn list_resources(
        &self,
        resource_type: ResourceType,
        base_path: &str,
    ) -> Result<Vec<String>, ResourcesError> {
        if matches!(
            resource_type,
            ResourceType::Content | ResourceType::Examples
        ) {
            return Err(ResourcesError::NotServable(resource_type));
        }
        Ok(self.loader(resource_type)?.list_resources(base_path))
    }

    /// Checks if a resource lookup is cached, including a cached miss.
    pub fn is_cached(&self, resource_type: ResourceType, path: &str) -> bool {
        if matches!(
            resource_type,
            ResourceType::Content | ResourceType::Examples
        ) || path.is_empty()
        {
            return false;
        }
        self.read_cache().contains_key(&Key {
            resource_type,
            path: path.to_owned(),
        })
    }

    // region static shortcuts

    /// Gets a [`ResourceType::Static`] entry, cached subject to the default policy.
    pub fn get_static(&self, path: &str) -> Result<Option<Entry>, ResourcesError> {
        self.get(ResourceType::Static, path, CachePolicy::Cache)
    }

    /// Gets a [`ResourceType::Static`] entry, honouring the given policy.
    pub fn get_static_with(
        &self,
        path: &str,
        policy: CachePolicy,
    ) -> Result<Option<Entry>, ResourcesError> {
        self.get(ResourceType::Static, path, policy)
    }

    /// Gets [`ResourceType::Static`] binary data with the default policy.
    pub fn get_static_binary(&self, path: &str) -> Result<Option<Vec<u8>>, ResourcesError> {
        self.get_binary(ResourceType::Static, path, CachePolicy::Cache)
    }

    /// Gets a [`ResourceType::Static`] resource as UTF-8 text.
    pub fn get_static_text(&self, path: &str) -> Result<Option<String>, ResourcesError> {
        self.get_text(ResourceType::Static, path, CachePolicy::Cache)
    }

    /// Checks whether a [`ResourceType::Static`] resource exists.
    pub fn exists_static(&self, path: &str) -> Result<bool, ResourcesError> {
        self.exists(ResourceType::Static, path)
    }

    /// Checks whether a [`ResourceType::Static`] lookup is cached.
    pub fn is_static_cached(&self, path: &str) -> bool {
        self.is_cached(ResourceType::Static, path)
    }

    /// Lists [`ResourceType::Static`] resources under the given directory.
    pub fn list_static_resources(&self, base_path: &str) -> Result<Vec<String>, ResourcesError> {
        self.list_resources(ResourceType::Static, base_path)
    }

    // endregion

    /// The maximum resource size in bytes that may be cached.
    pub fn max_cacheable_size(&self) -> u64 {
        self.max_cacheable_size
    }

    /// Clears all cached resources.
    pub fn clear(&self) {
        self.write_cache().clear();
    }

    fn loader(&self, resource_type: ResourceType) -> Result<&ResourcesLoader, ResourcesError> {
        self.loaders
            .get(&resource_type)
            .ok_or(ResourcesError::NoRootForType(resource_type))
    }

    fn read_cache(&self) -> RwLockReadGuard<'_, HashMap<Key, Option<Entry>>> {
        self.cache.read().unwrap_or_else(|error| error.into_inner())
    }

    fn write_cache(&self) -> RwLockWriteGuard<'_, HashMap<Key, Option<Entry>>> {
        self.cache
            .write()
            .unwrap_or_else(|error| error.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::{DEFAULT_MAX_CACHEABLE_SIZE, ResourcesCache};
    use crate::resources::cache_policy::CachePolicy;
    use crate::resources::error::ResourcesError;
    use crate::resources::resource_type::ResourceType;

    fn cache() -> ResourcesCache {
        let mut roots = HashMap::new();
        roots.insert(
            ResourceType::Static,
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test-static"),
        );
        roots.insert(
            ResourceType::Shared,
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test-shared"),
        );
        ResourcesCache::new(roots, DEFAULT_MAX_CACHEABLE_SIZE).unwrap()
    }

    #[test]
    fn requires_a_static_root() {
        let error = ResourcesCache::new(HashMap::new(), 0).unwrap_err();
        assert_eq!(error, ResourcesError::MissingStaticRoot);
    }

    #[test]
    fn rejects_nested_shared_root() {
        let root = PathBuf::from("/tmp/root");
        let mut roots = HashMap::new();
        roots.insert(ResourceType::Static, root.clone());
        roots.insert(ResourceType::Shared, root.join("shared"));
        let error = ResourcesCache::new(roots, 0).unwrap_err();
        assert_eq!(error, ResourcesError::NestedRoots);
    }

    #[test]
    fn content_is_not_servable() {
        let cache = cache();
        assert_eq!(
            cache.get(ResourceType::Content, "x", CachePolicy::Cache),
            Err(ResourcesError::NotServable(ResourceType::Content))
        );
    }

    #[test]
    fn missing_static_file_returns_none_and_is_cached_as_miss() {
        let cache = cache();
        assert!(cache.get_static("does/not/exist").unwrap().is_none());
        assert!(cache.is_static_cached("does/not/exist"));
    }
}
