use std::error::Error;
use std::fmt;

/// Errors produced while loading and validating the application config.
///
/// A missing file is not an error: the loader falls back to the embedded
/// default. This type is reserved for unreadable files, invalid YAML and
/// values that pass deserialization but fail validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// A config file exists but could not be read.
    Io { path: String, message: String },

    /// The YAML is malformed, a required field is missing or an unknown field
    /// is present.
    Parse { source: String, message: String },

    /// A port outside the valid TCP range `1..=65535` was configured.
    PortOutOfRange { port: u16, source: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => {
                write!(f, "cannot read config {path}: {message}")
            }
            Self::Parse { source, message } => {
                write!(f, "cannot parse config {source}: {message}")
            }
            Self::PortOutOfRange { port, source } => {
                write!(f, "port out of range: {port} in {source}")
            }
        }
    }
}

impl Error for ConfigError {}
