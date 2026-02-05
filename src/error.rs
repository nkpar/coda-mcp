use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodaError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Rate limited by Coda API")]
    RateLimited,

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
        assert_eq!(err.to_string(), "Rate limited by Coda API");
    }

    #[test]
    fn test_api_error_display() {
        let err = CodaError::Api {
            status: 404,
            body: "Not found".to_string(),
        };
        assert_eq!(err.to_string(), "API error 404: Not found");
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
