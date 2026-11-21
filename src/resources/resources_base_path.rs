use super::cache_policy::CachePolicy;
use super::error::ResourcesError;
use super::resource_type::ResourceType;
use super::unpack_policy::UnpackPolicy;

/// Single source of truth for resource directory paths.
///
/// Every variant represents a resource directory together with its base path
/// inside the embedded resources. It is the single point where all resource
/// directory names are defined, so the loader, the cache, the service and the
/// unpacker all agree on where each kind of resource lives.
///
/// The destination path ([`ResourcesBasePath::base_path`]) normally also
/// identifies the embedded directory the files are unpacked from. Entries like
/// [`ResourcesBasePath::MinecraftMapsYamlExamples`] use a separate
/// [`ResourcesBasePath::source_path`] when the packaged layout differs from the
/// runtime layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourcesBasePath {
    /// Frontend static HTML templates.
    Html,

    /// Stylesheets packaged with the application.
    Css,

    /// JavaScript files packaged with the application.
    Js,

    /// Image assets packaged with the application.
    Images,

    /// HTML template partials packaged with the application.
    Partials,

    /// Minecraft map ZIP archives shared from the Discord server.
    MinecraftMapsZips,

    /// Minecraft map YAML holding map metadata.
    MinecraftMapsYaml,

    /// Example Minecraft map YAML, unpacked into the examples root.
    MinecraftMapsYamlExamples,
}

impl ResourcesBasePath {
    /// Every directory that must exist on disk (for empty forks).
    pub const ALL: [ResourcesBasePath; 8] = [
        ResourcesBasePath::Html,
        ResourcesBasePath::Css,
        ResourcesBasePath::Js,
        ResourcesBasePath::Images,
        ResourcesBasePath::Partials,
        ResourcesBasePath::MinecraftMapsZips,
        ResourcesBasePath::MinecraftMapsYaml,
        ResourcesBasePath::MinecraftMapsYamlExamples,
    ];

    /// The destination directory, always ending with `'/'`.
    pub fn base_path(self) -> &'static str {
        match self {
            Self::Html => "html/",
            Self::Css => "css/",
            Self::Js => "js/",
            Self::Images => "images/",
            Self::Partials => "html/partials/",
            Self::MinecraftMapsZips => "minecraft_maps_zips/",
            Self::MinecraftMapsYaml | Self::MinecraftMapsYamlExamples => "minecraft_maps/",
        }
    }

    /// The embedded source directory, always ending with `'/'`.
    pub fn source_path(self) -> &'static str {
        match self {
            Self::MinecraftMapsYamlExamples => "examples/minecraft_maps/",
            other => other.base_path(),
        }
    }

    /// Whether this directory's resources are shipped in the binary.
    pub fn unpack_policy(self) -> UnpackPolicy {
        use ResourcesBasePath::*;
        match self {
            Html | Css | Js | Images | Partials | MinecraftMapsYamlExamples => {
                UnpackPolicy::Packaged
            }
            MinecraftMapsZips | MinecraftMapsYaml => UnpackPolicy::DiskOnly,
        }
    }

    /// Whether this directory's resources are packaged in the binary.
    pub fn is_packaged(self) -> bool {
        self.unpack_policy() == UnpackPolicy::Packaged
    }

    /// The cache policy for this directory's resources.
    pub fn cache_policy(self) -> CachePolicy {
        use ResourcesBasePath::*;
        match self {
            Html | Css | Js | Images | Partials => CachePolicy::Cache,
            MinecraftMapsZips | MinecraftMapsYaml | MinecraftMapsYamlExamples => {
                CachePolicy::DisableCache
            }
        }
    }

    /// The root this directory belongs to.
    pub fn resource_type(self) -> ResourceType {
        use ResourcesBasePath::*;
        match self {
            Html | Css | Js | Images | Partials => ResourceType::Static,
            MinecraftMapsYaml => ResourceType::Content,
            MinecraftMapsZips => ResourceType::Shared,
            MinecraftMapsYamlExamples => ResourceType::Examples,
        }
    }

    /// Whether this directory's resources may be cached.
    pub fn is_cached(self) -> bool {
        self.cache_policy() == CachePolicy::Cache
    }

    /// All directories that must exist on disk.
    pub fn directories_to_create() -> &'static [ResourcesBasePath] {
        &Self::ALL
    }

    /// Only those directories whose templates must be unpacked from the binary.
    pub fn packaged_directories() -> impl Iterator<Item = ResourcesBasePath> {
        Self::ALL.into_iter().filter(|dir| dir.is_packaged())
    }

    /// Resolves the full path for a resource of this type.
    ///
    /// The supplied name is validated before being appended, so path traversal
    /// sequences (`..`, `/`, `\`, `%`) are rejected and a caller can never
    /// address a file outside its own resource directory.
    pub fn resolve(self, name: &str) -> Result<String, ResourcesError> {
        if name.contains("..") || name.contains('/') || name.contains('\\') || name.contains('%') {
            return Err(ResourcesError::InvalidResourceName);
        }
        Ok(format!("{}{}", self.base_path(), name))
    }
}

#[cfg(test)]
mod tests {
    use super::ResourcesBasePath;
    use crate::resources::error::ResourcesError;
    use crate::resources::resource_type::ResourceType;

    #[test]
    fn resolve_prefixes_the_base_path() {
        assert_eq!(
            ResourcesBasePath::Css.resolve("style.css").unwrap(),
            "css/style.css"
        );
    }

    #[test]
    fn resolve_rejects_traversal() {
        for name in ["..", "../x", "a/b", "a\\b", "a%2e%2e"] {
            assert_eq!(
                ResourcesBasePath::Css.resolve(name),
                Err(ResourcesError::InvalidResourceName),
                "name {name:?} should be rejected"
            );
        }
    }

    #[test]
    fn examples_use_a_separate_source_path() {
        assert_eq!(
            ResourcesBasePath::MinecraftMapsYamlExamples.base_path(),
            "minecraft_maps/"
        );
        assert_eq!(
            ResourcesBasePath::MinecraftMapsYamlExamples.source_path(),
            "examples/minecraft_maps/"
        );
        assert_eq!(
            ResourcesBasePath::MinecraftMapsYamlExamples.resource_type(),
            ResourceType::Examples
        );
    }

    #[test]
    fn packaged_set_matches_policy() {
        let packaged: Vec<_> = ResourcesBasePath::packaged_directories().collect();
        assert!(packaged.iter().all(|dir| dir.is_packaged()));
        assert!(!packaged.contains(&ResourcesBasePath::MinecraftMapsZips));
        assert!(!packaged.contains(&ResourcesBasePath::MinecraftMapsYaml));
    }
}
