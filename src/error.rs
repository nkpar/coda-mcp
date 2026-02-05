use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodaError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Rate limited by Coda API. Please wait and try again.")]
    RateLimited,

    #[error("Permission denied. Your API token does not have write access. Generate a new token at https://coda.io/account with write permissions enabled.")]
    Forbidden,

    #[error("Not found. The document, table, or resource does not exist or you don't have access to it.")]
    NotFound,

    #[error("Unauthorized. Your API token is invalid or expired. Check your token at https://coda.io/account")]
    Unauthorized,

    #[error("API error {status}: {body}")]
    Api { status: u16, body: String },

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Export timed out after {seconds} seconds")]
    ExportTimeout { seconds: u64 },

    #[error("Export failed: {message}")]
    ExportFailed { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limited_error_display() {
        let err = CodaError::RateLimited;
        assert!(err.to_string().contains("Rate limited"));
    }

    #[test]
    fn test_forbidden_error_display() {
        let err = CodaError::Forbidden;
        assert!(err.to_string().contains("Permission denied"));
        assert!(err.to_string().contains("write access"));
        assert!(err.to_string().contains("coda.io/account"));
    }

    #[test]
    fn test_not_found_error_display() {
        let err = CodaError::NotFound;
        assert!(err.to_string().contains("Not found"));
    }

    #[test]
    fn test_unauthorized_error_display() {
        let err = CodaError::Unauthorized;
        assert!(err.to_string().contains("Unauthorized"));
        assert!(err.to_string().contains("invalid or expired"));
    }

    #[test]
    fn test_api_error_display() {
        let err = CodaError::Api {
            status: 500,
            body: "Internal error".to_string(),
        };
        assert_eq!(err.to_string(), "API error 500: Internal error");
    }

    #[test]
    fn test_export_timeout_error_display() {
        let err = CodaError::ExportTimeout { seconds: 30 };
        assert_eq!(err.to_string(), "Export timed out after 30 seconds");
    }

    #[test]
    fn test_export_failed_error_display() {
        let err = CodaError::ExportFailed {
            message: "Invalid format".to_string(),
        };
        assert_eq!(err.to_string(), "Export failed: Invalid format");
    }

    #[test]
    fn test_json_error_from() {
        let json_err: Result<serde_json::Value, _> = serde_json::from_str("invalid json");
        let err: CodaError = json_err.unwrap_err().into();
        assert!(err.to_string().contains("JSON parse error"));
    }
}
