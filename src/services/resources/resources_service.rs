use std::sync::Arc;

use crate::resources::error::ResourcesError;
use crate::resources::resource_type::ResourceType;
use crate::resources::resources_base_path::ResourcesBasePath;
use crate::resources::resources_cache::ResourcesCache;

/// A resource ready to be written to a response.
#[derive(Clone, Debug)]
pub struct ResourceData {
    /// The resource bytes; must not be modified.
    pub data: Arc<Vec<u8>>,
    /// The ETag of these bytes, or `None` for shared resources, which are
    /// never cached or hashed.
    pub etag: Option<String>,
    /// The content type derived from the file extension.
    pub content_type: &'static str,
}

/// Public API for serving static and shared resources.
///
/// The resource type is part of the contract:
///
/// - [`ResourceType::Static`] resources are served and may be cached;
/// - [`ResourceType::Shared`] resources are served but never cached;
/// - [`ResourceType::Content`] is deliberately not served. Content is
///   configuration data read by its own services, not an HTTP resource, so
///   there are no content methods here and every attempt to use a content base
///   path fails fast;
/// - [`ResourceType::Examples`] is not served either: example files are copied
///   out for the operator at startup.
///
/// Every method that takes a caller-supplied name runs it through
/// [`ResourcesBasePath::resolve`], so a name coming from a URL can never address
/// anything outside its own resource directory. Raw-path access is deliberately
/// not part of this API.
pub struct ResourcesService {
    cache: ResourcesCache,
}

impl ResourcesService {
    /// Creates a new service backed by the given cache.
    pub fn new(cache: ResourcesCache) -> Self {
        Self { cache }
    }

    /// Loads a single file of a resource directory together with its ETag and
    /// content type.
    ///
    /// Returns `Ok(None)` when the name is empty or the file is missing, and an
    /// error when the base path is not servable or the name contains path
    /// traversal characters.
    pub fn get_resource(
        &self,
        base_path: ResourcesBasePath,
        name: &str,
    ) -> Result<Option<ResourceData>, ResourcesError> {
        require_servable(base_path)?;
        if name.is_empty() {
            return Ok(None);
        }

        let path = base_path.resolve(name)?;
        let entry = self
            .cache
            .get(base_path.resource_type(), &path, base_path.cache_policy())?;

        Ok(entry.map(|entry| ResourceData {
            data: entry.data,
            etag: entry.etag,
            content_type: Self::detect_mime_type(name),
        }))
    }

    /// Detects the MIME type from a file extension.
    pub fn detect_mime_type(filename: &str) -> &'static str {
        match file_extension(filename).as_str() {
            "html" | "htm" => "text/html; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "js" => "application/javascript; charset=utf-8",
            "json" => "application/json; charset=utf-8",
            "xml" => "application/xml; charset=utf-8",
            "txt" => "text/plain; charset=utf-8",
            "svg" => "image/svg+xml; charset=utf-8",
            "webmanifest" => "application/manifest+json; charset=utf-8",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "webp" => "image/webp",
            "avif" => "image/avif",
            "ico" => "image/x-icon",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "pdf" => "application/pdf",
            "zip" => "application/zip",
            _ => "application/octet-stream",
        }
    }

    /// Clears all cached resources.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Returns the files available in a resource directory.
    pub fn list_resources(
        &self,
        base_path: ResourcesBasePath,
    ) -> Result<Vec<String>, ResourcesError> {
        require_servable(base_path)?;
        self.cache
            .list_resources(base_path.resource_type(), base_path.base_path())
    }

    /// Returns a JSON array of the file names under `base_path`, with the base
    /// path stripped from each entry.
    ///
    /// Returns `Ok(None)` if serialization fails.
    pub fn list_as_json(
        &self,
        base_path: ResourcesBasePath,
    ) -> Result<Option<String>, ResourcesError> {
        let prefix = base_path.base_path();
        let resources = self.list_resources(base_path)?;
        let files: Vec<&str> = resources
            .iter()
            .map(|file| file.strip_prefix(prefix).unwrap_or(file))
            .collect();
        Ok(serde_json::to_string(&files).ok())
    }

    // region static resource helpers

    /// Returns the binary data for an image resource.
    pub fn get_image(&self, name: &str) -> Result<Option<Vec<u8>>, ResourcesError> {
        self.get_bytes(ResourcesBasePath::Images, name)
    }

    /// Returns the text of a template partial.
    ///
    /// Partials are build-time template fragments. They are never served over
    /// HTTP and never appear in a listing.
    pub fn get_partial(&self, name: &str) -> Result<Option<String>, ResourcesError> {
        self.get_text(ResourcesBasePath::Partials, name)
    }

    /// Returns the HTML content for a given resource name.
    pub fn get_html(&self, name: &str) -> Result<Option<String>, ResourcesError> {
        self.get_text(ResourcesBasePath::Html, name)
    }

    /// Returns the CSS content for a given resource name.
    pub fn get_css(&self, name: &str) -> Result<Option<String>, ResourcesError> {
        self.get_text(ResourcesBasePath::Css, name)
    }

    /// Returns the JavaScript content for a given resource name.
    pub fn get_js(&self, name: &str) -> Result<Option<String>, ResourcesError> {
        self.get_text(ResourcesBasePath::Js, name)
    }

    // endregion

    /// Reads a named file of a resource directory as UTF-8 text.
    fn get_text(
        &self,
        base_path: ResourcesBasePath,
        name: &str,
    ) -> Result<Option<String>, ResourcesError> {
        require_servable(base_path)?;
        if name.is_empty() {
            return Ok(None);
        }
        self.cache.get_text(
            base_path.resource_type(),
            &base_path.resolve(name)?,
            base_path.cache_policy(),
        )
    }

    /// Reads a named file of a resource directory as bytes.
    fn get_bytes(
        &self,
        base_path: ResourcesBasePath,
        name: &str,
    ) -> Result<Option<Vec<u8>>, ResourcesError> {
        require_servable(base_path)?;
        if name.is_empty() {
            return Ok(None);
        }
        self.cache.get_binary(
            base_path.resource_type(),
            &base_path.resolve(name)?,
            base_path.cache_policy(),
        )
    }
}

/// Rejects content and examples before any lookup happens: content is not an
/// HTTP resource and must be read by its own service, while examples are copied
/// out by the unpacker and never served.
fn require_servable(base_path: ResourcesBasePath) -> Result<(), ResourcesError> {
    match base_path.resource_type() {
        ResourceType::Content | ResourceType::Examples => {
            Err(ResourcesError::NotServable(base_path.resource_type()))
        }
        ResourceType::Static | ResourceType::Shared => Ok(()),
    }
}

fn file_extension(filename: &str) -> String {
    match filename.rsplit_once('.') {
        Some((_, extension)) if !extension.is_empty() => extension.to_ascii_lowercase(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::ResourcesService;

    #[test]
    fn detects_common_mime_types() {
        assert_eq!(
            ResourcesService::detect_mime_type("index.html"),
            "text/html; charset=utf-8"
        );
        assert_eq!(ResourcesService::detect_mime_type("logo.PNG"), "image/png");
        assert_eq!(
            ResourcesService::detect_mime_type("archive.zip"),
            "application/zip"
        );
        assert_eq!(
            ResourcesService::detect_mime_type("unknown.bin"),
            "application/octet-stream"
        );
        assert_eq!(
            ResourcesService::detect_mime_type("no-extension"),
            "application/octet-stream"
        );
    }
}
