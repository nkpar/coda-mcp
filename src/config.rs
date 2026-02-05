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

    /// Helper to save, run test, and restore env vars.
    /// Always sets a sentinel value before the test so restore branches are exercised.
    fn with_env_vars<F: FnOnce()>(f: F) {
        let saved_token = env::var("CODA_API_TOKEN").ok();
        let saved_url = env::var("CODA_BASE_URL").ok();

        // Pre-set sentinel values so restore branches always execute
        env::set_var("CODA_API_TOKEN", "__sentinel__");
        env::set_var("CODA_BASE_URL", "__sentinel__");

        f();

        // Restore original values
        match saved_token {
            Some(val) => env::set_var("CODA_API_TOKEN", val),
            None => env::remove_var("CODA_API_TOKEN"),
        }
        match saved_url {
            Some(val) => env::set_var("CODA_BASE_URL", val),
            None => env::remove_var("CODA_BASE_URL"),
        }
    }

    #[test]
    fn test_from_env_missing_token() {
        with_env_vars(|| {
            env::remove_var("CODA_API_TOKEN");

            let result = Config::from_env();
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), ConfigError::MissingToken));
        });
    }

    #[test]
    fn test_from_env_with_token_default_url() {
        with_env_vars(|| {
            env::set_var("CODA_API_TOKEN", "test_token_123");
            env::remove_var("CODA_BASE_URL");

            let config = Config::from_env().unwrap();
            assert_eq!(config.api_token, "test_token_123");
            assert_eq!(config.base_url, "https://coda.io/apis/v1");
        });
    }

    #[test]
    fn test_from_env_with_custom_base_url() {
        with_env_vars(|| {
            env::set_var("CODA_API_TOKEN", "test_token_456");
            env::set_var("CODA_BASE_URL", "https://custom.api.example.com/v2");

            let config = Config::from_env().unwrap();
            assert_eq!(config.api_token, "test_token_456");
            assert_eq!(config.base_url, "https://custom.api.example.com/v2");
        });
    }

    #[test]
    fn test_with_env_vars_restores_existing_values() {
        // Pre-set env vars so that saved_token/saved_url are Some(_)
        // This ensures the Some(val) => env::set_var restore branch is covered
        env::set_var("CODA_API_TOKEN", "pre_existing_token");
        env::set_var("CODA_BASE_URL", "https://pre-existing.example.com");

        with_env_vars(|| {
            // Inside the closure, the sentinel values are set
            // The test just needs to run; restore happens on exit
        });

        // Verify the original values were restored
        assert_eq!(env::var("CODA_API_TOKEN").unwrap(), "pre_existing_token");
        assert_eq!(
            env::var("CODA_BASE_URL").unwrap(),
            "https://pre-existing.example.com"
        );

        // Clean up
        env::remove_var("CODA_API_TOKEN");
        env::remove_var("CODA_BASE_URL");
    }
}
