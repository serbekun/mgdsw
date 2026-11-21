use std::path::{Component, Path, PathBuf};

use rust_embed::RustEmbed;

use super::lookup_order::LookupOrder;

/// Resources embedded into the binary at build time.
///
/// A single unit struct covers the whole packaged tree; individual directories
/// are addressed by their relative path (see
/// [`ResourcesBasePath`](super::resources_base_path::ResourcesBasePath)).
#[derive(RustEmbed)]
#[folder = "src/resources_data/"]
pub struct EmbeddedResources;

/// Directory used for on-disk overrides when none is configured explicitly.
pub const DEFAULT_OVERRIDE_ROOT: &str = "data/static";

/// Loads resources from the on-disk override directory first, then from the
/// embedded resources.
///
/// The disk side has an explicit root: every path is resolved against it and
/// normalized, and anything that escapes the root is rejected. This holds even
/// when a caller passes a raw, unvalidated path, so the loader never depends on
/// [`ResourcesBasePath::resolve`](super::resources_base_path::ResourcesBasePath::resolve)
/// having been called upstream.
#[derive(Debug)]
pub struct ResourcesLoader {
    override_root: PathBuf,
    order: LookupOrder,
}

impl Default for ResourcesLoader {
    fn default() -> Self {
        Self::new(DEFAULT_OVERRIDE_ROOT)
    }
}

impl ResourcesLoader {
    /// Creates a disk-first loader using [`DEFAULT_OVERRIDE_ROOT`] as the disk root.
    pub fn with_default_root() -> Self {
        Self::default()
    }

    /// Creates a disk-first loader with an explicit on-disk override root.
    pub fn new(override_root: impl AsRef<Path>) -> Self {
        Self::with_order(override_root, LookupOrder::DiskFirst)
    }

    /// Creates a loader with an explicit on-disk override root and lookup order.
    pub fn with_order(override_root: impl AsRef<Path>, order: LookupOrder) -> Self {
        Self {
            override_root: absolute_normalize(override_root.as_ref()),
            order,
        }
    }

    /// The absolute, normalized directory disk lookups are confined to.
    pub fn override_root(&self) -> &Path {
        &self.override_root
    }

    /// The order this loader resolves disk and embedded resources in.
    pub fn order(&self) -> LookupOrder {
        self.order
    }

    /// Loads a resource as binary data, honouring the lookup order.
    pub fn load_binary(&self, path: &str) -> Option<Vec<u8>> {
        if !is_safe_resource_path(path) {
            return None;
        }

        match self.order {
            LookupOrder::DiskFirst => self
                .load_from_disk(path)
                .or_else(|| self.load_from_classpath(path)),
            LookupOrder::ClasspathFirst => self
                .load_from_classpath(path)
                .or_else(|| self.load_from_disk(path)),
            LookupOrder::DiskOnly => self.load_from_disk(path),
        }
    }

    /// Checks whether a resource exists under the configured lookup order.
    pub fn exists(&self, path: &str) -> bool {
        if !is_safe_resource_path(path) {
            return false;
        }

        match self.order {
            LookupOrder::DiskOnly => self.resolve_on_disk(path).is_some(),
            LookupOrder::DiskFirst => {
                self.resolve_on_disk(path).is_some() || self.exists_on_classpath(path)
            }
            LookupOrder::ClasspathFirst => {
                self.exists_on_classpath(path) || self.resolve_on_disk(path).is_some()
            }
        }
    }

    /// Reads a resource from the embedded resources.
    ///
    /// Public so the first-run unpacker can copy the packaged templates out
    /// without going through the cache or the disk root.
    pub fn load_from_classpath(&self, path: &str) -> Option<Vec<u8>> {
        if !is_safe_resource_path(path) {
            return None;
        }
        EmbeddedResources::get(path).map(|file| file.data.into_owned())
    }

    /// Returns the resources available under the given directory, merging disk
    /// and embedded listings according to the lookup order.
    ///
    /// Unlike [`Self::load_binary`], a single overriding file on disk does not
    /// hide the rest of the packaged directory.
    pub fn list_resources(&self, base_path: &str) -> Vec<String> {
        let base_path = with_trailing_slash(base_path);
        if !is_safe_resource_path(&base_path) {
            return Vec::new();
        }

        match self.order {
            // A disk-only deployment lists disk-only: the embedded tree is a
            // template source the unpacker consumed at startup, not a live
            // second tree.
            LookupOrder::DiskOnly => self.list_from_disk(&base_path),
            LookupOrder::ClasspathFirst => merge(
                self.list_classpath(&base_path),
                self.list_from_disk(&base_path),
            ),
            LookupOrder::DiskFirst => merge(
                self.list_from_disk(&base_path),
                self.list_classpath(&base_path),
            ),
        }
    }

    /// Lists the resources embedded in the binary only.
    ///
    /// Public so the first-run unpacker can enumerate the templates to copy
    /// without merging in the destination directory it is about to write to.
    pub fn list_classpath(&self, base_path: &str) -> Vec<String> {
        let base_path = with_trailing_slash(base_path);
        if !is_safe_resource_path(&base_path) {
            return Vec::new();
        }

        let mut result: Vec<String> = EmbeddedResources::iter()
            .map(|path| path.into_owned())
            .filter(|path| path.starts_with(&base_path))
            .collect();
        result.sort();
        result.dedup();
        result
    }

    /// Reads a resource from disk.
    fn load_from_disk(&self, path: &str) -> Option<Vec<u8>> {
        let file = self.resolve_on_disk(path)?;
        std::fs::read(file).ok()
    }

    /// Checks whether a resource exists in the embedded resources without
    /// reading its contents.
    fn exists_on_classpath(&self, path: &str) -> bool {
        EmbeddedResources::get(path).is_some()
    }

    /// Resolves a resource path to a readable file inside the override root.
    ///
    /// The path is normalized before the containment check, so `..` segments
    /// cannot escape, and the resolved file is compared through
    /// [`std::fs::canonicalize`] so a symlink pointing outside the root is
    /// rejected as well.
    fn resolve_on_disk(&self, path: &str) -> Option<PathBuf> {
        let candidate = lexical_normalize(&self.override_root.join(path));
        if !candidate.starts_with(&self.override_root) {
            return None;
        }
        if !candidate.is_file() {
            return None;
        }

        let real = std::fs::canonicalize(&candidate).ok()?;
        let root_real = std::fs::canonicalize(&self.override_root).ok()?;
        if !real.starts_with(&root_real) {
            return None;
        }
        Some(real)
    }

    /// Lists all files of a directory inside the on-disk override root.
    fn list_from_disk(&self, base_path: &str) -> Vec<String> {
        let dir = lexical_normalize(&self.override_root.join(base_path));
        if !dir.starts_with(&self.override_root) || !dir.is_dir() {
            return Vec::new();
        }
        list_from_directory(&dir, base_path)
    }
}

/// Returns `true` when a path is relative and free of traversal segments.
///
/// Applied to embedded lookups as well: the embedded tree does not confine
/// lookups on its own.
pub fn is_safe_resource_path(path: &str) -> bool {
    if path.trim().is_empty() {
        return false;
    }
    if path.starts_with('/') || path.contains('\\') {
        return false;
    }
    !path.split('/').any(|segment| segment == "..")
}

/// Lexically normalizes a path without touching the filesystem.
pub(crate) fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Returns an absolute, lexically normalized path.
pub(crate) fn absolute_normalize(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        match std::env::current_dir() {
            Ok(current_dir) => current_dir.join(path),
            Err(_) => path.to_path_buf(),
        }
    };
    lexical_normalize(&absolute)
}

fn with_trailing_slash(base_path: &str) -> String {
    if base_path.ends_with('/') {
        base_path.to_owned()
    } else {
        return format!("{base_path}/");
    }
}

fn merge(first: Vec<String>, second: Vec<String>) -> Vec<String> {
    let mut merged = first;
    merged.extend(second);
    merged.sort();
    merged.dedup();
    merged
}

/// Lists files under a directory on the filesystem, recursively.
///
/// Subdirectories are kept in the result (e.g. `html/partials/head.html`), so
/// a directory listing matches what the embedded tree reports.
fn list_from_directory(dir: &Path, base_path: &str) -> Vec<String> {
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            match entry.file_type() {
                Ok(file_type) if file_type.is_dir() => walk(&path, files),
                Ok(file_type) if file_type.is_file() => files.push(path),
                _ => {}
            }
        }
    }

    let mut files = Vec::new();
    walk(dir, &mut files);
    files.sort();

    files
        .into_iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(dir).ok()?;
            Some(format!(
                "{base_path}{}",
                relative.to_string_lossy().replace('\\', "/")
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{is_safe_resource_path, lexical_normalize};
    use std::path::Path;

    #[test]
    fn rejects_unsafe_paths() {
        for path in ["", " ", "/etc/passwd", "a/../b", "a\\b"] {
            assert!(!is_safe_resource_path(path), "{path:?} should be rejected");
        }
        for path in ["html/index.html", "html/", "a/b/c.txt"] {
            assert!(is_safe_resource_path(path), "{path:?} should be accepted");
        }
    }

    #[test]
    fn normalizes_lexically() {
        assert_eq!(
            lexical_normalize(Path::new("/root/sub/../file.txt")),
            Path::new("/root/file.txt")
        );
        assert_eq!(
            lexical_normalize(Path::new("/root/./sub/../sub/file.txt")),
            Path::new("/root/sub/file.txt")
        );
    }
}
