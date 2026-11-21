/// Whether a resource may be retained in the [`ResourcesCache`].
///
/// [`ResourcesCache`]: super::resources_cache::ResourcesCache
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CachePolicy {
    /// The resource may be cached.
    Cache,

    /// The resource is loaded on every request and never retained.
    DisableCache,
}
