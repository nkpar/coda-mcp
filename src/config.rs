use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("CODA_API_TOKEN environment variable is required")]
    MissingToken,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub api_token: String,
    pub base_url: String,
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
    fn test_config_debug() {
        let config = Config {
            api_token: "secret".to_string(),
            base_url: "https://api.example.com".to_string(),
        };

        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("api_token"));
        assert!(debug_str.contains("base_url"));
    }
}
