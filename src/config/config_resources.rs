use serde::Deserialize;

/// Resource roots and caching limits.
///
/// Maps the `resources` section of the YAML config. Every field is mandatory,
/// and `max_cacheable_size` uses `u64`, so a negative value fails during
/// deserialization.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigResources {
    /// Upper bound in bytes for a resource to be eligible for caching.
    pub max_cacheable_size: u64,

    /// Templates and static assets served by nginx (HTML, CSS, JS, images,
    /// partials).
    pub static_base_path: String,

    /// Operator-managed metadata that can change without rebuilding the server
    /// (for example, map YAML).
    pub content_base_path: String,

    /// Large shared files such as Minecraft map ZIP archives.
    pub shared_base_path: String,

    /// Example files copied out for the operator, never served over HTTP.
    pub examples_base_path: String,
}
