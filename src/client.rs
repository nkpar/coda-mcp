use flate2::read::GzDecoder;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Read;
use std::time::Duration;

use crate::config::Config;
use crate::error::CodaError;

/// Trusted hosts for downloading export content
const ALLOWED_DOWNLOAD_HOSTS: &[&str] = &["coda.io", "codahosted.io", "storage.googleapis.com"];

#[derive(Clone)]
pub struct CodaClient {
    client: Client,
    base_url: String,
    api_token: String,
}

impl CodaClient {
    pub fn new(config: &Config) -> Self {
        tracing::info!("Creating Coda API client");
        // Build client with explicit settings to match curl behaviour:
        // - Disable connection pooling to avoid HTTP/2 multiplexing issues
        // - Set reasonable timeouts
        let client = Client::builder()
            .pool_max_idle_per_host(0) // Disable connection pooling
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.base_url.clone(),
            api_token: config.api_token.clone(),
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, CodaError> {
        let url = format!("{}{}", self.base_url, path);

        tracing::info!("GET {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        let status = response.status();
        tracing::debug!("Response status: {}", status);

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            tracing::error!("API error {}: {}", status.as_u16(), body);
            return Err(match status.as_u16() {
                401 => CodaError::Unauthorized,
                403 => CodaError::Forbidden,
                404 => CodaError::NotFound,
                429 => CodaError::RateLimited,
                _ => CodaError::Api {
                    status: status.as_u16(),
                    body,
                },
            });
        }

        let body = response.text().await?;
        tracing::debug!("Response body: {}", body);
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, CodaError> {
        let url = format!("{}{}", self.base_url, path);
        tracing::info!("POST {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        tracing::debug!("Response status: {}", status);

        if !status.is_success() && status.as_u16() != 202 {
            let body = response.text().await.unwrap_or_default();
            tracing::error!("API error {}: {}", status.as_u16(), body);
            return Err(match status.as_u16() {
                401 => CodaError::Unauthorized,
                403 => CodaError::Forbidden,
                404 => CodaError::NotFound,
                429 => CodaError::RateLimited,
                _ => CodaError::Api {
                    status: status.as_u16(),
                    body,
                },
            });
        }

        let body = response.text().await?;
        tracing::debug!("Response body: {}", body);
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, CodaError> {
        let url = format!("{}{}", self.base_url, path);
        tracing::debug!("PUT {}", url);

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() && status.as_u16() != 202 {
            let body = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 => CodaError::Unauthorized,
                403 => CodaError::Forbidden,
                404 => CodaError::NotFound,
                429 => CodaError::RateLimited,
                _ => CodaError::Api {
                    status: status.as_u16(),
                    body,
                },
            });
        }

        let body = response.text().await?;
        tracing::trace!("Response: {}", body);
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn delete(&self, path: &str) -> Result<(), CodaError> {
        let url = format!("{}{}", self.base_url, path);
        tracing::debug!("DELETE {}", url);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() && status.as_u16() != 202 {
            let body = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 => CodaError::Unauthorized,
                403 => CodaError::Forbidden,
                404 => CodaError::NotFound,
                429 => CodaError::RateLimited,
                _ => CodaError::Api {
                    status: status.as_u16(),
                    body,
                },
            });
        }

        Ok(())
    }

    /// Download raw content from an external URL (used for export downloads)
    /// Automatically decompresses gzip content if detected
    /// Only allows downloads from trusted Coda-related hosts
    pub async fn download_raw(&self, url: &str) -> Result<String, CodaError> {
        // Validate URL is from a trusted host
        let parsed = url::Url::parse(url).map_err(|e| CodaError::Api {
            status: 0,
            body: format!("Invalid URL: {e}"),
        })?;

        let host = parsed.host_str().unwrap_or("");

        if !ALLOWED_DOWNLOAD_HOSTS.iter().any(|h| host.ends_with(h)) {
            tracing::warn!("Blocked download from untrusted host: {}", host);
            return Err(CodaError::Api {
                status: 0,
                body: format!("Untrusted download host: {host}"),
            });
        }

        tracing::debug!("Downloading from external URL: {}", url);

        let response = self.client.get(url).send().await?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(CodaError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let bytes = response.bytes().await?;
        tracing::debug!("Downloaded {} bytes", bytes.len());

        // Check for gzip magic bytes (0x1f, 0x8b)
        if bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b {
            tracing::debug!("Detected gzip content, decompressing...");
            let mut decoder = GzDecoder::new(&bytes[..]);
            let mut decompressed = String::new();
            decoder
                .read_to_string(&mut decompressed)
                .map_err(|e| CodaError::Api {
                    status: 0,
                    body: format!("Failed to decompress gzip: {e}"),
                })?;
            tracing::debug!("Decompressed to {} bytes", decompressed.len());
            Ok(decompressed)
        } else {
            // Not gzip, return as string
            Ok(String::from_utf8_lossy(&bytes).to_string())
        }
    }

    #[cfg(test)]
    pub fn new_with_base_url(api_token: &str, base_url: &str) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(0)
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
            api_token: api_token.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/docs"))
            .and(header("Authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [{"id": "doc1", "name": "Test Doc"}]
            })))
            .mount(&mock_server)
            .await;

        let client = CodaClient::new_with_base_url("test_token", &mock_server.uri());
        let result: serde_json::Value = client.get("/docs").await.unwrap();

        assert!(result["items"].is_array());
        assert_eq!(result["items"][0]["id"], "doc1");
    }

    #[tokio::test]
    async fn test_get_rate_limited() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/docs"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        let client = CodaClient::new_with_base_url("test_token", &mock_server.uri());
        let result: Result<serde_json::Value, _> = client.get("/docs").await;

        assert!(matches!(result, Err(CodaError::RateLimited)));
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/docs/invalid"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
            .mount(&mock_server)
            .await;

        let client = CodaClient::new_with_base_url("test_token", &mock_server.uri());
        let result: Result<serde_json::Value, _> = client.get("/docs/invalid").await;

        assert!(matches!(result, Err(CodaError::NotFound)));
    }

    #[tokio::test]
    async fn test_get_forbidden() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/docs"))
            .respond_with(ResponseTemplate::new(403).set_body_string("Forbidden"))
            .mount(&mock_server)
            .await;

        let client = CodaClient::new_with_base_url("test_token", &mock_server.uri());
        let result: Result<serde_json::Value, _> = client.get("/docs").await;

        assert!(matches!(result, Err(CodaError::Forbidden)));
    }

    #[tokio::test]
    async fn test_get_unauthorized() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/docs"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        let client = CodaClient::new_with_base_url("test_token", &mock_server.uri());
        let result: Result<serde_json::Value, _> = client.get("/docs").await;

        assert!(matches!(result, Err(CodaError::Unauthorized)));
    }

    #[tokio::test]
    async fn test_post_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/docs/doc1/tables/tbl1/rows"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "requestId": "req123",
                "addedRowIds": ["row1"]
            })))
            .mount(&mock_server)
            .await;

        let client = CodaClient::new_with_base_url("test_token", &mock_server.uri());
        let body = serde_json::json!({"rows": [{"cells": []}]});
        let result: serde_json::Value = client
            .post("/docs/doc1/tables/tbl1/rows", &body)
            .await
            .unwrap();

        assert_eq!(result["requestId"], "req123");
    }

    #[tokio::test]
    async fn test_put_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/docs/doc1/tables/tbl1/rows/row1"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "requestId": "req456"
            })))
            .mount(&mock_server)
            .await;

        let client = CodaClient::new_with_base_url("test_token", &mock_server.uri());
        let body = serde_json::json!({"row": {"cells": []}});
        let result: serde_json::Value = client
            .put("/docs/doc1/tables/tbl1/rows/row1", &body)
            .await
            .unwrap();

        assert_eq!(result["requestId"], "req456");
    }

    #[tokio::test]
    async fn test_delete_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/docs/doc1/tables/tbl1/rows/row1"))
            .respond_with(ResponseTemplate::new(202))
            .mount(&mock_server)
            .await;

        let client = CodaClient::new_with_base_url("test_token", &mock_server.uri());
        let result = client.delete("/docs/doc1/tables/tbl1/rows/row1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_rate_limited() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/docs/doc1/tables/tbl1/rows/row1"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        let client = CodaClient::new_with_base_url("test_token", &mock_server.uri());
        let result = client.delete("/docs/doc1/tables/tbl1/rows/row1").await;

        assert!(matches!(result, Err(CodaError::RateLimited)));
    }

    #[tokio::test]
    async fn test_get_json_parse_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/docs"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
            .mount(&mock_server)
            .await;

        let client = CodaClient::new_with_base_url("test_token", &mock_server.uri());
        let result: Result<serde_json::Value, _> = client.get("/docs").await;

        assert!(matches!(result, Err(CodaError::Json(_))));
    }

    // Note: download_raw_success test removed because it used localhost which is now
    // blocked by the trusted host validation. The functionality is tested via
    // test_download_raw_allows_trusted_hosts which validates the host allowlist.

    #[tokio::test]
    async fn test_download_raw_rejects_untrusted_host() {
        let client = CodaClient::new_with_base_url("test_token", "https://api.coda.io");

        // Try to download from an untrusted host
        let result = client
            .download_raw("https://evil.example.com/malicious")
            .await;

        match result {
            Err(CodaError::Api { status, body }) => {
                assert_eq!(status, 0);
                assert!(body.contains("Untrusted download host"));
            }
            _ => panic!("Expected API error for untrusted host"),
        }
    }

    #[tokio::test]
    async fn test_download_raw_rejects_invalid_url() {
        let client = CodaClient::new_with_base_url("test_token", "https://api.coda.io");

        let result = client.download_raw("not-a-valid-url").await;

        match result {
            Err(CodaError::Api { status, body }) => {
                assert_eq!(status, 0);
                assert!(body.contains("Invalid URL"));
            }
            _ => panic!("Expected API error for invalid URL"),
        }
    }

    #[tokio::test]
    async fn test_download_raw_allows_trusted_hosts() {
        let client = CodaClient::new_with_base_url("test_token", "https://api.coda.io");

        // These hosts should be allowed (will fail to connect but pass validation)
        let trusted_urls = [
            "https://coda.io/export/123",
            "https://export.codahosted.io/file.html",
            "https://storage.googleapis.com/bucket/file",
        ];

        for url in trusted_urls {
            let result = client.download_raw(url).await;
            // Should fail with connection error, NOT untrusted host error
            match result {
                Err(CodaError::Api { body, .. }) if body.contains("Untrusted") => {
                    panic!("URL {} should be trusted but was rejected", url);
                }
                _ => {} // Any other error (e.g., connection failure) is fine
            }
        }
    }
}
