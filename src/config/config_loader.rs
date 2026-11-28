use std::fs;
use std::path::Path;

use tracing::info;

use super::config_context::ConfigContext;
use super::error::ConfigError;

/// Environment variable holding the optional path to a config file on disk.
pub const CONFIG_ENV: &str = "MGDSC_CONFIG";

/// Source label used for the config embedded at build time.
pub const DEFAULT_CONFIG_SOURCE: &str = "embedded:config/config.default.yaml";

/// Fallback config embedded into the binary at build time.
///
/// Mirrors the classpath resource `config/config.default.yaml` of the Java
/// implementation.
pub const DEFAULT_CONFIG: &str = include_str!("config.default.yaml");

/// Reads, validates and assembles the application configuration.
///
/// The loader resolves a YAML file in two steps:
///
/// 1. If a path is given (normally from [`CONFIG_ENV`]) and the file exists, it
///    is read from disk.
/// 2. Otherwise the embedded [`DEFAULT_CONFIG`] is used.
///
/// Deserialization is strict: missing sections, missing fields and unknown
/// fields are errors, so a typo cannot silently fall back to a default port or
/// root. A file that exists but cannot be read is an error, not a fallback.
///
/// # Example
///
/// ```ignore
/// let config = ConfigLoader::new().load_from_env()?;
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct ConfigLoader;

impl ConfigLoader {
    /// Creates a loader.
    pub fn new() -> Self {
        Self
    }

    /// Loads the config from the path in [`CONFIG_ENV`], falling back to the
    /// embedded default when the variable is unset or empty.
    pub fn load_from_env(&self) -> Result<ConfigContext, ConfigError> {
        match std::env::var(CONFIG_ENV)
            .ok()
            .filter(|value| !value.is_empty())
        {
            Some(path) => {
                info!("Loading config from {CONFIG_ENV}={path}");
                self.load(Some(Path::new(&path)))
            }
            None => {
                info!("{CONFIG_ENV} is not set, using the embedded default config");
                self.load_default()
            }
        }
    }

    /// Loads the config from `path`, falling back to the embedded default when
    /// no path is given or the file does not exist.
    ///
    /// A path that exists but cannot be read or parsed is an error.
    pub fn load(&self, path: Option<&Path>) -> Result<ConfigContext, ConfigError> {
        match path {
            Some(path) if path.exists() => self.load_file(path),
            Some(path) => {
                info!(
                    "Config file {} does not exist, using the embedded default config",
                    path.display()
                );
                self.load_default()
            }
            None => self.load_default(),
        }
    }

    /// Loads the fallback config embedded into the binary.
    pub fn load_default(&self) -> Result<ConfigContext, ConfigError> {
        self.parse(DEFAULT_CONFIG, DEFAULT_CONFIG_SOURCE)
    }

    fn load_file(&self, path: &Path) -> Result<ConfigContext, ConfigError> {
        let source = path.display().to_string();
        let yaml = fs::read_to_string(path).map_err(|error| ConfigError::Io {
            path: source.clone(),
            message: error.to_string(),
        })?;
        self.parse(&yaml, &source)
    }

    fn parse(&self, yaml: &str, source: &str) -> Result<ConfigContext, ConfigError> {
        let config: ConfigContext =
            serde_yaml::from_str(yaml).map_err(|error| ConfigError::Parse {
                source: source.to_owned(),
                message: error.to_string(),
            })?;
        config.http.validate(source)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ConfigLoader, DEFAULT_CONFIG, DEFAULT_CONFIG_SOURCE};
    use crate::config::error::ConfigError;

    fn parse(yaml: &str) -> Result<super::ConfigContext, ConfigError> {
        ConfigLoader::new().parse(yaml, "test")
    }

    #[test]
    fn default_config_has_the_java_values() {
        let config = ConfigLoader::new().load_default().unwrap();

        assert_eq!(config.http.port, 8080);
        assert_eq!(config.resources.max_cacheable_size, 104_857_600);
        assert_eq!(config.resources.static_base_path, "data/static/");
        assert_eq!(config.resources.content_base_path, "data/content/");
        assert_eq!(config.resources.shared_base_path, "data/shared/");
        assert_eq!(config.resources.examples_base_path, "data/examples/");
    }

    #[test]
    fn embedded_default_is_not_empty() {
        assert!(!DEFAULT_CONFIG.trim().is_empty());
        assert!(
            ConfigLoader::new()
                .parse(DEFAULT_CONFIG, DEFAULT_CONFIG_SOURCE)
                .is_ok()
        );
    }

    #[test]
    fn missing_http_section_is_an_error() {
        let error = parse(
            r#"
resources:
  maxCacheableSize: 1
  staticBasePath: "s/"
  contentBasePath: "c/"
  sharedBasePath: "h/"
  examplesBasePath: "e/"
"#,
        )
        .unwrap_err();

        assert!(matches!(error, ConfigError::Parse { .. }));
        assert!(error.to_string().contains("missing field `http`"));
    }

    #[test]
    fn missing_resources_section_is_an_error() {
        let error = parse("http:\n  port: 8080\n").unwrap_err();
        assert!(matches!(error, ConfigError::Parse { .. }));
        assert!(error.to_string().contains("missing field `resources`"));
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let error = parse(
            r#"
http:
  port: 8080
  host: "0.0.0.0"
resources:
  maxCacheableSize: 1
  staticBasePath: "s/"
  contentBasePath: "c/"
  sharedBasePath: "h/"
  examplesBasePath: "e/"
"#,
        )
        .unwrap_err();

        assert!(matches!(error, ConfigError::Parse { .. }));
        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn port_zero_is_out_of_range() {
        let error = parse(
            r#"
http:
  port: 0
resources:
  maxCacheableSize: 1
  staticBasePath: "s/"
  contentBasePath: "c/"
  sharedBasePath: "h/"
  examplesBasePath: "e/"
"#,
        )
        .unwrap_err();

        assert_eq!(
            error,
            ConfigError::PortOutOfRange {
                port: 0,
                source: "test".to_owned()
            }
        );
    }

    #[test]
    fn port_above_u16_is_rejected() {
        let error = parse(
            r#"
http:
  port: 70000
resources:
  maxCacheableSize: 1
  staticBasePath: "s/"
  contentBasePath: "c/"
  sharedBasePath: "h/"
  examplesBasePath: "e/"
"#,
        )
        .unwrap_err();

        assert!(matches!(error, ConfigError::Parse { .. }));
    }

    #[test]
    fn negative_cache_size_is_rejected() {
        let error = parse(
            r#"
http:
  port: 8080
resources:
  maxCacheableSize: -1
  staticBasePath: "s/"
  contentBasePath: "c/"
  sharedBasePath: "h/"
  examplesBasePath: "e/"
"#,
        )
        .unwrap_err();

        assert!(matches!(error, ConfigError::Parse { .. }));
    }

    #[test]
    fn missing_file_falls_back_to_default() {
        let path = PathBuf::from("target/config-loader-does-not-exist.yaml");
        let config = ConfigLoader::new().load(Some(&path)).unwrap();
        assert_eq!(config.http.port, 8080);
    }

    #[test]
    fn a_file_on_disk_overrides_the_default() {
        let path = PathBuf::from("target/config-loader-override.yaml");
        std::fs::write(
            &path,
            r#"
http:
  port: 9090
resources:
  maxCacheableSize: 42
  staticBasePath: "/srv/static/"
  contentBasePath: "/srv/content/"
  sharedBasePath: "/srv/shared/"
  examplesBasePath: "/srv/examples/"
"#,
        )
        .unwrap();

        let config = ConfigLoader::new().load(Some(&path)).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(config.http.port, 9090);
        assert_eq!(config.resources.max_cacheable_size, 42);
        assert_eq!(config.resources.static_base_path, "/srv/static/");
    }
}
