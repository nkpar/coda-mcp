//! End-to-end tests for Coda MCP server
//!
//! These tests require a valid `CODA_API_TOKEN` environment variable.
//! Run with: `cargo test --test e2e_tests -- --ignored --test-threads=1`
//!
//! To run: `export $(cat .env | xargs) && cargo test --test e2e_tests -- --ignored --test-threads=1`

#![allow(clippy::manual_let_else)]
#![allow(clippy::single_match_else)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unused_async)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::redundant_closure_for_method_calls)]

use std::env;

fn get_token() -> Option<String> {
    env::var("CODA_API_TOKEN").ok()
}

fn skip_if_no_token() -> bool {
    if get_token().is_none() {
        eprintln!("Skipping E2E test: CODA_API_TOKEN not set");
        true
    } else {
        false
    }
}

mod e2e {
    use super::*;
    use reqwest::Client;

    const BASE_URL: &str = "https://coda.io/apis/v1";

    async fn get_client() -> (Client, String) {
        let token = get_token().expect("CODA_API_TOKEN required");
        (Client::new(), token)
    }

    #[tokio::test]
    #[ignore = "requires CODA_API_TOKEN"]
    async fn test_list_docs() {
        if skip_if_no_token() {
            return;
        }

        let (client, token) = get_client().await;
        let resp = client
            .get(format!("{BASE_URL}/docs?limit=5"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        assert!(
            resp.status().is_success(),
            "list_docs failed: {}",
            resp.status()
        );
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["items"].is_array(), "Expected items array");
        println!("Found {} docs", body["items"].as_array().unwrap().len());
    }

    #[tokio::test]
    #[ignore = "requires CODA_API_TOKEN"]
    async fn test_get_doc() {
        if skip_if_no_token() {
            return;
        }

        let (client, token) = get_client().await;

        // First get a doc ID
        let list_resp = client
            .get(format!("{BASE_URL}/docs?limit=1"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        let list_body: serde_json::Value = list_resp.json().await.unwrap();
        let doc_id = list_body["items"][0]["id"].as_str().unwrap();

        // Then get the specific doc
        let resp = client
            .get(format!("{BASE_URL}/docs/{doc_id}"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        assert!(
            resp.status().is_success(),
            "get_doc failed: {}",
            resp.status()
        );
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["id"].is_string(), "Expected doc id");
        assert!(body["name"].is_string(), "Expected doc name");
        println!("Got doc: {} ({})", body["name"], body["id"]);
    }

    #[tokio::test]
    #[ignore = "requires CODA_API_TOKEN"]
    async fn test_list_tables() {
        if skip_if_no_token() {
            return;
        }

        let (client, token) = get_client().await;

        // First get a doc ID
        let list_resp = client
            .get(format!("{BASE_URL}/docs?limit=1"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        if !list_resp.status().is_success() {
            println!("Rate limited or error, skipping: {}", list_resp.status());
            return;
        }

        let list_body: serde_json::Value = list_resp.json().await.unwrap();
        let doc_id = match list_body["items"].get(0).and_then(|d| d["id"].as_str()) {
            Some(id) => id,
            None => {
                println!("No docs available, skipping test");
                return;
            }
        };

        // Then list tables
        let resp = client
            .get(format!("{BASE_URL}/docs/{doc_id}/tables?limit=5"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        assert!(
            resp.status().is_success(),
            "list_tables failed: {}",
            resp.status()
        );
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["items"].is_array(), "Expected items array");
        println!("Found {} tables", body["items"].as_array().unwrap().len());
    }

    #[tokio::test]
    #[ignore = "requires CODA_API_TOKEN"]
    async fn test_list_pages() {
        if skip_if_no_token() {
            return;
        }

        let (client, token) = get_client().await;

        // First get a doc ID
        let list_resp = client
            .get(format!("{BASE_URL}/docs?limit=1"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        if !list_resp.status().is_success() {
            println!("Rate limited or error, skipping: {}", list_resp.status());
            return;
        }

        let list_body: serde_json::Value = list_resp.json().await.unwrap();
        let doc_id = match list_body["items"].get(0).and_then(|d| d["id"].as_str()) {
            Some(id) => id,
            None => {
                println!("No docs available, skipping test");
                return;
            }
        };

        // Then list pages
        let resp = client
            .get(format!("{BASE_URL}/docs/{doc_id}/pages?limit=5"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        assert!(
            resp.status().is_success(),
            "list_pages failed: {}",
            resp.status()
        );
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["items"].is_array(), "Expected items array");
        println!("Found {} pages", body["items"].as_array().unwrap().len());
    }

    #[tokio::test]
    #[ignore = "requires CODA_API_TOKEN"]
    async fn test_get_rows() {
        if skip_if_no_token() {
            return;
        }

        let (client, token) = get_client().await;

        // First get a doc ID
        let list_resp = client
            .get(format!("{BASE_URL}/docs?limit=1"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        let list_body: serde_json::Value = list_resp.json().await.unwrap();
        let doc_id = list_body["items"][0]["id"].as_str().unwrap();

        // Get tables
        let tables_resp = client
            .get(format!("{BASE_URL}/docs/{doc_id}/tables?limit=1"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        let tables_body: serde_json::Value = tables_resp.json().await.unwrap();
        if tables_body["items"]
            .as_array()
            .map(|a| a.is_empty())
            .unwrap_or(true)
        {
            println!("No tables found, skipping get_rows test");
            return;
        }

        let table_id = tables_body["items"][0]["id"].as_str().unwrap();

        // Get rows
        let resp = client
            .get(format!(
                "{BASE_URL}/docs/{doc_id}/tables/{table_id}/rows?limit=3&useColumnNames=true"
            ))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        assert!(
            resp.status().is_success(),
            "get_rows failed: {}",
            resp.status()
        );
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["items"].is_array(), "Expected items array");
        println!("Found {} rows", body["items"].as_array().unwrap().len());
    }

    #[tokio::test]
    #[ignore = "requires CODA_API_TOKEN"]
    async fn test_list_columns() {
        if skip_if_no_token() {
            return;
        }

        let (client, token) = get_client().await;

        // First get a doc ID
        let list_resp = client
            .get(format!("{BASE_URL}/docs?limit=1"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        if !list_resp.status().is_success() {
            println!("Rate limited or error, skipping: {}", list_resp.status());
            return;
        }

        let list_body: serde_json::Value = list_resp.json().await.unwrap();
        let doc_id = match list_body["items"].get(0).and_then(|d| d["id"].as_str()) {
            Some(id) => id,
            None => {
                println!("No docs available, skipping test");
                return;
            }
        };

        // Get tables
        let tables_resp = client
            .get(format!("{BASE_URL}/docs/{doc_id}/tables?limit=1"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        if !tables_resp.status().is_success() {
            println!("Rate limited or error, skipping: {}", tables_resp.status());
            return;
        }

        let tables_body: serde_json::Value = tables_resp.json().await.unwrap();
        let table_id = match tables_body["items"].get(0).and_then(|t| t["id"].as_str()) {
            Some(id) => id,
            None => {
                println!("No tables found, skipping list_columns test");
                return;
            }
        };

        // Get columns
        let resp = client
            .get(format!(
                "{BASE_URL}/docs/{doc_id}/tables/{table_id}/columns"
            ))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        assert!(
            resp.status().is_success(),
            "list_columns failed: {}",
            resp.status()
        );
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["items"].is_array(), "Expected items array");
        println!("Found {} columns", body["items"].as_array().unwrap().len());
    }

    #[tokio::test]
    #[ignore = "requires CODA_API_TOKEN"]
    async fn test_search_docs() {
        if skip_if_no_token() {
            return;
        }

        let (client, token) = get_client().await;
        let resp = client
            .get(format!("{BASE_URL}/docs?query=test"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        if resp.status() == 429 {
            println!("Rate limited, skipping search_docs test");
            return;
        }

        assert!(
            resp.status().is_success(),
            "search_docs failed: {}",
            resp.status()
        );
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["items"].is_array(), "Expected items array");
        println!(
            "Found {} docs matching 'test'",
            body["items"].as_array().unwrap().len()
        );
    }

    #[tokio::test]
    #[ignore = "requires CODA_API_TOKEN"]
    async fn test_invalid_token() {
        let client = Client::new();
        let resp = client
            .get(format!("{BASE_URL}/docs"))
            .header("Authorization", "Bearer invalid_token_12345")
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 401, "Expected 401 for invalid token");
    }

    #[tokio::test]
    #[ignore = "requires CODA_API_TOKEN"]
    async fn test_nonexistent_doc() {
        if skip_if_no_token() {
            return;
        }

        let (client, token) = get_client().await;
        let resp = client
            .get(format!("{BASE_URL}/docs/nonexistent_doc_id_12345"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 404, "Expected 404 for nonexistent doc");
    }

    #[tokio::test]
    #[ignore = "requires CODA_API_TOKEN - creates and deletes a real document"]
    async fn test_create_and_delete_doc() {
        if skip_if_no_token() {
            return;
        }

        let (client, token) = get_client().await;

        // Create a new document
        let create_resp = client
            .post(format!("{BASE_URL}/docs"))
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "title": "E2E Test Document - Safe to Delete"
            }))
            .send()
            .await
            .unwrap();

        if create_resp.status() == 429 {
            println!("Rate limited on create, skipping test");
            return;
        }

        assert!(
            create_resp.status().is_success(),
            "create_doc failed: {}",
            create_resp.status()
        );

        let create_body: serde_json::Value = create_resp.json().await.unwrap();
        let doc_id = create_body["id"].as_str().expect("Expected doc id");
        println!("Created document: {} ({})", create_body["name"], doc_id);

        // Verify we can get the document
        let get_resp = client
            .get(format!("{BASE_URL}/docs/{doc_id}"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        assert!(
            get_resp.status().is_success(),
            "get_doc failed: {}",
            get_resp.status()
        );

        // Delete the document
        let delete_resp = client
            .delete(format!("{BASE_URL}/docs/{doc_id}"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        assert!(
            delete_resp.status().is_success() || delete_resp.status() == 202,
            "delete_doc failed: {}",
            delete_resp.status()
        );
        println!("Deleted document: {}", doc_id);

        // Verify the document is gone (should return 404)
        // Note: There may be a delay before the API reflects the deletion
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let verify_resp = client
            .get(format!("{BASE_URL}/docs/{doc_id}"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        assert_eq!(
            verify_resp.status(),
            404,
            "Document should be deleted (404), got: {}",
            verify_resp.status()
        );
        println!("Verified document is deleted");
    }
}
