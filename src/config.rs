use std::env;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("CODA_API_TOKEN environment variable is required")]
    MissingToken,
}

#[derive(Clone)]
pub struct Config {
    pub api_token: String,
    pub base_url: String,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("api_token", &"[REDACTED]")
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let api_token = env::var("CODA_API_TOKEN").map_err(|_| ConfigError::MissingToken)?;

        let base_url =
            env::var("CODA_BASE_URL").unwrap_or_else(|_| "https://coda.io/apis/v1".to_string());

        tracing::info!("Config loaded: base_url={}", base_url);

        Ok(Self {
            api_token,
            base_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::MissingToken;
        assert_eq!(
            err.to_string(),
            "CODA_API_TOKEN environment variable is required"
        );
    }

    #[test]
    fn test_config_clone() {
        let config = Config {
            api_token: "token123".to_string(),
            base_url: "https://api.example.com".to_string(),
        };

        let cloned = config.clone();
        assert_eq!(cloned.api_token, "token123");
        assert_eq!(cloned.base_url, "https://api.example.com");
    }

    #[test]
    fn test_config_debug_redacts_token() {
        let config = Config {
            api_token: "super_secret_token_12345".to_string(),
            base_url: "https://api.example.com".to_string(),
        };

        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("[REDACTED]"));
        assert!(debug_str.contains("base_url"));
        // Ensure the actual token is NOT in the debug output
        assert!(!debug_str.contains("super_secret_token_12345"));
    }
}
