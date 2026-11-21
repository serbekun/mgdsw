/// The root a resource belongs to.
///
/// Each type maps to its own base directory, so equal relative paths in
/// different roots never collide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// Static files like HTML, CSS, JS and images.
    Static,

    /// Content files that can change without rebuilding the server.
    Content,

    /// Shared files such as Minecraft map ZIP archives, which can change
    /// without rebuilding the server.
    Shared,

    /// Example files copied out for operators; never served over HTTP.
    Examples,
}
