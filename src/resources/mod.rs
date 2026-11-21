//! Resource loading, caching and first-run unpacking.
//!
//! The module is a Rust port of the Java `com.serbekun.mgdsw.resources` package
//! and keeps the same contract:
//!
//! - [`ResourcesBasePath`] is the single source of truth for resource directory
//!   names and their unpack/cache policies;
//! - [`ResourcesLoader`] reads disk overrides first, then the resources embedded
//!   in the binary;
//! - [`ResourcesCache`] keeps bytes (and ETags) for cacheable resource types;
//! - [`ResourcesUnpacker`] writes the packaged templates to disk on first run or
//!   after a version upgrade;
//! - [`ResourcesService`] is the public, name-validating API used by HTTP
//!   handlers.

#![allow(unused_imports)]

pub mod cache_policy;
pub mod error;
pub mod etags;
pub mod lookup_order;
pub mod resource_type;
pub mod resources_base_path;
pub mod resources_cache;
pub mod resources_loader;
pub mod resources_unpacker;
pub mod unpack_policy;

pub use cache_policy::CachePolicy;
pub use error::ResourcesError;
pub use lookup_order::LookupOrder;
pub use resource_type::ResourceType;
pub use resources_base_path::ResourcesBasePath;
pub use resources_cache::{DEFAULT_MAX_CACHEABLE_SIZE, Entry, ResourcesCache};
pub use resources_loader::{DEFAULT_OVERRIDE_ROOT, EmbeddedResources, ResourcesLoader};
pub use resources_unpacker::{MARKER_FILE, ResourceRoots, ResourcesUnpacker};
pub use unpack_policy::UnpackPolicy;
