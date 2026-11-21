use std::error::Error;
use std::fmt;

use super::resource_type::ResourceType;

/// Errors produced by the resources module.
///
/// "Not found" is not an error: lookups return [`Option`] for that. This type
/// is reserved for misuse and for failures while preparing or reading
/// resources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourcesError {
    /// A caller-supplied resource name contains path traversal characters.
    InvalidResourceName,

    /// The resource type is not served over HTTP (`Content` or `Examples`).
    NotServable(ResourceType),

    /// No root was configured for the requested resource type.
    NoRootForType(ResourceType),

    /// The cache was created without a `Static` root.
    MissingStaticRoot,

    /// Two configured roots are equal or nested inside each other.
    NestedRoots,

    /// A resolved path escaped its resource root.
    PathEscapesRoot(String),

    /// An I/O operation failed.
    Io(String),
}

impl fmt::Display for ResourcesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidResourceName => {
                write!(f, "invalid resource name: path traversal is not allowed")
            }
            Self::NotServable(resource_type) => {
                write!(f, "{resource_type:?} resources are not served by this API")
            }
            Self::NoRootForType(resource_type) => {
                write!(f, "no root configured for resource type {resource_type:?}")
            }
            Self::MissingStaticRoot => write!(f, "a Static root is required"),
            Self::NestedRoots => {
                write!(
                    f,
                    "configured resource roots must be separate, non-nested directories"
                )
            }
            Self::PathEscapesRoot(path) => {
                write!(f, "resource path escapes its root: {path}")
            }
            Self::Io(message) => write!(f, "resource I/O failed: {message}"),
        }
    }
}

impl Error for ResourcesError {}

impl From<std::io::Error> for ResourcesError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
