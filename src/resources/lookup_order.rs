/// Where a resource is looked up, and in what order.
///
/// The mode is chosen per runtime rather than in the config, because it is a
/// property of the deployment:
///
/// - [`LookupOrder::DiskFirst`] — a file on disk shadows the packaged copy.
///   Used by tests and by anything that wants live overrides on top of the
///   bundled templates.
/// - [`LookupOrder::ClasspathFirst`] — the packaged copy wins when both exist.
/// - [`LookupOrder::DiskOnly`] — only the on-disk directory is read. This is
///   the production mode: after the unpacker has written the templates once,
///   the running server never falls back to the embedded copy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LookupOrder {
    /// Disk first, then the embedded resources.
    #[default]
    DiskFirst,

    /// Embedded resources first, then disk.
    ClasspathFirst,

    /// Disk only; embedded resources are never consulted.
    DiskOnly,
}
