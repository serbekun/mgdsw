/// Whether the resources of a [`ResourcesBasePath`] directory are shipped
/// inside the binary and unpacked on first run, or exist only on disk.
///
/// This is not [`LookupOrder`]: the policy describes what ships in the
/// artifact, while the lookup order decides which copy wins at request time.
///
/// [`ResourcesBasePath`]: super::resources_base_path::ResourcesBasePath
/// [`LookupOrder`]: super::lookup_order::LookupOrder
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnpackPolicy {
    /// Templates bundled in the binary and copied to disk on first run (and
    /// after a version upgrade). Files already on disk are never overwritten.
    Packaged,

    /// Nothing is shipped for this directory; it is created empty on disk and
    /// populated by the operator. The unpacker never copies files into it.
    DiskOnly,
}
