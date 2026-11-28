//! Application configuration.
//!
//! The module mirrors the Java config package of `pre-alpha-2026-11-09`:
//!
//! - [`ConfigLoader`] reads a YAML file from [`CONFIG_ENV`] or falls back to
//!   the config embedded into the binary;
//! - [`ConfigContext`] is the single object handed to the rest of the
//!   application;
//! - [`ConfigHttpInit`] and [`ConfigResources`] describe the `http` and
//!   `resources` sections.
//!
//! Parsing is strict: missing and unknown fields are errors.

#![allow(unused_imports)]

pub mod config_context;
pub mod config_http_init;
pub mod config_loader;
pub mod config_resources;
pub mod error;

pub use config_context::ConfigContext;
pub use config_http_init::ConfigHttpInit;
pub use config_loader::{CONFIG_ENV, ConfigLoader, DEFAULT_CONFIG, DEFAULT_CONFIG_SOURCE};
pub use config_resources::ConfigResources;
pub use error::ConfigError;
