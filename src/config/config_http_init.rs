use serde::Deserialize;

use super::error::ConfigError;

/// Configuration for the embedded HTTP server.
///
/// Maps the `http` section of the YAML config. The `port` is mandatory: a
/// missing value fails during deserialization, and `0` is rejected by
/// [`ConfigHttpInit::validate`] because it is not a usable TCP port.
///
/// Ports above `65535` are already rejected by the `u16` deserialization.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfigHttpInit {
    /// TCP port the HTTP server binds to.
    pub port: u16,
}

impl ConfigHttpInit {
    /// Rejects values that deserialize successfully but cannot be listened on.
    pub fn validate(&self, source: &str) -> Result<(), ConfigError> {
        if self.port == 0 {
            return Err(ConfigError::PortOutOfRange {
                port: self.port,
                source: source.to_owned(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigHttpInit;

    #[test]
    fn accepts_a_usable_port() {
        assert!(ConfigHttpInit { port: 8080 }.validate("test").is_ok());
        assert!(ConfigHttpInit { port: 1 }.validate("test").is_ok());
        assert!(ConfigHttpInit { port: 65535 }.validate("test").is_ok());
    }

    #[test]
    fn rejects_port_zero() {
        assert_eq!(
            ConfigHttpInit { port: 0 }
                .validate("test")
                .unwrap_err()
                .to_string(),
            "port out of range: 0 in test"
        );
    }
}
