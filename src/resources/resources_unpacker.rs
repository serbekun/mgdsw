use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::error::ResourcesError;
use super::resource_type::ResourceType;
use super::resources_base_path::ResourcesBasePath;
use super::resources_loader::{ResourcesLoader, absolute_normalize, lexical_normalize};

/// Marker file, relative to the static root, recording the unpacked version.
pub const MARKER_FILE: &str = ".unpacked";

/// The four independent roots resources are split into, one per
/// [`ResourceType`].
#[derive(Clone, Debug)]
pub struct ResourceRoots {
    /// Templates and static assets served by nginx (HTML, CSS, JS, images,
    /// partials).
    pub static_root: PathBuf,
    /// Operator-managed metadata that can change without rebuilding the server
    /// (for example, map YAML).
    pub content_root: PathBuf,
    /// Large shared files such as Minecraft map ZIP archives.
    pub shared_root: PathBuf,
    /// Example files copied out for the operator, never served over HTTP.
    pub examples_root: PathBuf,
}

impl ResourceRoots {
    /// Creates a set of roots, normalizing each one to an absolute path.
    pub fn new(
        static_root: impl AsRef<Path>,
        content_root: impl AsRef<Path>,
        shared_root: impl AsRef<Path>,
        examples_root: impl AsRef<Path>,
    ) -> Self {
        Self {
            static_root: absolute_normalize(static_root.as_ref()),
            content_root: absolute_normalize(content_root.as_ref()),
            shared_root: absolute_normalize(shared_root.as_ref()),
            examples_root: absolute_normalize(examples_root.as_ref()),
        }
    }

    fn root_for(&self, resource_type: ResourceType) -> &Path {
        match resource_type {
            ResourceType::Static => &self.static_root,
            ResourceType::Content => &self.content_root,
            ResourceType::Shared => &self.shared_root,
            ResourceType::Examples => &self.examples_root,
        }
    }

    /// Rejects equal or nested roots. The static root is served publicly, so
    /// content, shared data and examples must never live inside it (or inside
    /// each other).
    fn require_separate(&self) -> Result<(), ResourcesError> {
        let roots = [
            &self.static_root,
            &self.content_root,
            &self.shared_root,
            &self.examples_root,
        ];
        for (index, first) in roots.iter().enumerate() {
            for second in roots.iter().skip(index + 1) {
                if first == second || first.starts_with(second) || second.starts_with(first) {
                    return Err(ResourcesError::NestedRoots);
                }
            }
        }
        Ok(())
    }
}

/// Creates the runtime data directories and writes the packaged template
/// resources to disk on first run.
///
/// Every root and every directory declared by [`ResourcesBasePath`] is created
/// on startup. Only directories with
/// [`UnpackPolicy::Packaged`](super::unpack_policy::UnpackPolicy::Packaged) are
/// copied from the embedded tree, and each one is copied into the root of its
/// own [`ResourceType`]. Disk-only directories are created empty; their
/// contents are supplied by the operator.
///
/// The unpacker is deliberately conservative:
///
/// - a file that already exists on disk is never overwritten, so an edited
///   template or a dropped-in asset survives a re-run;
/// - the marker carries the build version, so upgrading the binary re-runs the
///   copy and fills in templates that are new in that version while still
///   leaving existing files alone;
/// - the marker lives in the static root only, because only packaged templates
///   are versioned with the binary. Content and shared files are owned by the
///   operator.
pub struct ResourcesUnpacker {
    loader: ResourcesLoader,
    version: String,
}

impl ResourcesUnpacker {
    /// Creates an unpacker.
    ///
    /// The loader reads the packaged resources; the version is recorded in the
    /// marker.
    pub fn new(loader: ResourcesLoader, version: impl Into<String>) -> Self {
        Self {
            loader,
            version: version.into(),
        }
    }

    /// Ensures the configured roots and their subdirectories exist, then
    /// unpacks the packaged templates when the static copy is missing or from
    /// another version.
    ///
    /// Returns `true` when the templates were (re)written, `false` when the
    /// static copy was already current.
    pub fn unpack(&self, roots: &ResourceRoots) -> Result<bool, ResourcesError> {
        roots.require_separate()?;

        let marker = roots.static_root.join(MARKER_FILE);
        let unpacked_version = if marker.is_file() {
            fs::read_to_string(&marker)
                .map(|contents| contents.trim().to_owned())
                .unwrap_or_default()
        } else {
            String::new()
        };
        let current = !unpacked_version.is_empty() && self.version == unpacked_version;

        for root in [
            &roots.static_root,
            &roots.content_root,
            &roots.shared_root,
            &roots.examples_root,
        ] {
            fs::create_dir_all(root)?;
        }

        for dir in ResourcesBasePath::directories_to_create() {
            self.create_directory(roots.root_for(dir.resource_type()), dir.base_path())?;
        }

        let mut copied = 0;
        if !current {
            for dir in ResourcesBasePath::packaged_directories() {
                let dir_root = roots.root_for(dir.resource_type());
                copied += self.copy_directory(dir_root, dir.source_path(), dir.base_path())?;
            }
            fs::write(&marker, format!("{}\n", self.version))?;
        }

        // The count is kept for parity with the Java implementation and for
        // callers that will log it once a logger is wired up.
        let _ = copied;

        Ok(!current)
    }

    fn create_directory(&self, root: &Path, dir: &str) -> Result<(), ResourcesError> {
        let target = lexical_normalize(&root.join(dir));
        if !target.starts_with(root) {
            return Err(ResourcesError::PathEscapesRoot(dir.to_owned()));
        }
        fs::create_dir_all(target)?;
        Ok(())
    }

    /// Copies every file of one packaged directory that is not already on disk.
    ///
    /// The embedded source and the destination directory may differ: files are
    /// written below `destination_dir` while the file names are taken relative
    /// to `source_dir`.
    fn copy_directory(
        &self,
        root: &Path,
        source_dir: &str,
        destination_dir: &str,
    ) -> Result<usize, ResourcesError> {
        let mut copied = 0;

        for path in self.loader.list_classpath(source_dir) {
            let relative = &path[source_dir.len()..];
            let destination = format!("{destination_dir}{relative}");
            let target = lexical_normalize(&root.join(&destination));
            if !target.starts_with(root) {
                return Err(ResourcesError::PathEscapesRoot(destination));
            }

            let Some(data) = self.loader.load_from_classpath(&path) else {
                continue;
            };

            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }

            // `create_new` both skips existing files and refuses to follow a
            // symlink planted at the destination.
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
            {
                Ok(mut file) => {
                    file.write_all(&data)?;
                    copied += 1;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(ResourcesError::Io(error.to_string())),
            }
        }

        Ok(copied)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{MARKER_FILE, ResourceRoots, ResourcesUnpacker};
    use crate::resources::error::ResourcesError;
    use crate::resources::resources_loader::ResourcesLoader;

    fn roots(base: PathBuf) -> ResourceRoots {
        ResourceRoots::new(
            base.join("static"),
            base.join("content"),
            base.join("shared"),
            base.join("examples"),
        )
    }

    #[test]
    fn rejects_nested_roots() {
        let base = PathBuf::from("target/unpacker-test");
        let nested = ResourceRoots::new(
            base.join("static"),
            base.join("static/content"),
            base.join("shared"),
            base.join("examples"),
        );
        assert_eq!(
            ResourcesUnpacker::new(ResourcesLoader::new(&base), "v")
                .unpack(&nested)
                .unwrap_err(),
            ResourcesError::NestedRoots
        );
    }

    #[test]
    fn unpacks_packaged_templates_and_writes_marker() {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/unpacker-test-ok");
        let _ = std::fs::remove_dir_all(&base);
        let roots = roots(base.clone());

        let unpacker = ResourcesUnpacker::new(ResourcesLoader::new(&base), "test-version");
        assert!(unpacker.unpack(&roots).unwrap());

        assert!(roots.static_root.join("html/resource_test.html").is_file());
        assert!(roots.static_root.join("css").is_dir());
        assert!(roots.content_root.join("minecraft_maps").is_dir());
        assert_eq!(
            std::fs::read_to_string(roots.static_root.join(MARKER_FILE))
                .unwrap()
                .trim(),
            "test-version"
        );

        // A second run with the same version is a no-op.
        assert!(!unpacker.unpack(&roots).unwrap());
    }
}
