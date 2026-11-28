use serde::Deserialize;

use super::config_http_init::ConfigHttpInit;
use super::config_resources::ConfigResources;

/// Aggregate holder for every configuration section of the application.
///
/// [`ConfigLoader`](super::config_loader::ConfigLoader) reads, validates and
/// assembles the config; this type is the single object it hands back. Both
/// sections are mandatory, so a missing `http` or `resources` section fails
/// during deserialization.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfigContext {
    /// Embedded HTTP server settings.
    pub http: ConfigHttpInit,

    /// Resource roots and caching limits.
    pub resources: ConfigResources,
}
