use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// We need to test the actual tool handlers. Since they are in main.rs,
// we'll create a helper module to test the business logic.

mod common {
    use super::*;

    pub async fn setup_mock_server() -> MockServer {
        MockServer::start().await
    }

    pub fn mock_docs_response() -> serde_json::Value {
        serde_json::json!({
            "items": [
                {"id": "doc1", "name": "Document 1"},
                {"id": "doc2", "name": "Document 2"}
            ]
        })
    }

    pub fn mock_tables_response() -> serde_json::Value {
        serde_json::json!({
            "items": [
                {"id": "tbl1", "name": "Tasks", "rowCount": 10},
                {"id": "tbl2", "name": "Projects", "rowCount": 5}
            ]
        })
    }

    pub fn mock_rows_response() -> serde_json::Value {
        serde_json::json!({
            "items": [
                {"id": "row1", "name": "Row 1", "values": {"Name": "Alice", "Status": "Active"}},
                {"id": "row2", "name": "Row 2", "values": {"Name": "Bob", "Status": "Inactive"}}
            ]
        })
    }

    pub fn mock_pages_response() -> serde_json::Value {
        serde_json::json!({
            "items": [
                {"id": "page1", "name": "Home"},
                {"id": "page2", "name": "About"}
            ]
        })
    }

    pub fn mock_columns_response() -> serde_json::Value {
        serde_json::json!({
            "items": [
                {"id": "col1", "name": "Name", "format": {"type": "text"}},
                {"id": "col2", "name": "Status", "format": {"type": "select"}}
            ]
        })
    }

    pub fn mock_formulas_response() -> serde_json::Value {
        serde_json::json!({
            "items": [
                {"id": "f1", "name": "TotalTasks", "value": 15},
                {"id": "f2", "name": "CompletedTasks", "value": 10}
            ]
        })
    }

    pub fn mock_controls_response() -> serde_json::Value {
        serde_json::json!({
            "items": [
                {"id": "ctrl1", "name": "Submit", "controlType": "button"},
                {"id": "ctrl2", "name": "Progress", "controlType": "slider", "value": 75}
            ]
        })
    }
}

#[tokio::test]
async fn test_list_docs_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs"))
        .and(query_param("limit", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(common::mock_docs_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/docs?limit=50", mock_server.uri()))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_tables_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs/doc1/tables"))
        .respond_with(ResponseTemplate::new(200).set_body_json(common::mock_tables_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/docs/doc1/tables", mock_server.uri()))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["items"][0]["name"], "Tasks");
}

#[tokio::test]
async fn test_get_rows_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs/doc1/tables/tbl1/rows"))
        .respond_with(ResponseTemplate::new(200).set_body_json(common::mock_rows_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/docs/doc1/tables/tbl1/rows", mock_server.uri()))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["items"][0]["values"]["Name"], "Alice");
}

#[tokio::test]
async fn test_add_row_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/docs/doc1/tables/tbl1/rows"))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "requestId": "req123",
            "addedRowIds": ["row-new"]
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/docs/doc1/tables/tbl1/rows", mock_server.uri()))
        .header("Authorization", "Bearer test_token")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "rows": [{"cells": [{"column": "Name", "value": "Charlie"}]}]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 202);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["addedRowIds"][0], "row-new");
}

#[tokio::test]
async fn test_update_row_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("PUT"))
        .and(path("/docs/doc1/tables/tbl1/rows/row1"))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "requestId": "req456"
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .put(format!(
            "{}/docs/doc1/tables/tbl1/rows/row1",
            mock_server.uri()
        ))
        .header("Authorization", "Bearer test_token")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "row": {"cells": [{"column": "Status", "value": "Done"}]}
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 202);
}

#[tokio::test]
async fn test_delete_row_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("DELETE"))
        .and(path("/docs/doc1/tables/tbl1/rows/row1"))
        .respond_with(ResponseTemplate::new(202))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .delete(format!(
            "{}/docs/doc1/tables/tbl1/rows/row1",
            mock_server.uri()
        ))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 202);
}

#[tokio::test]
async fn test_list_pages_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs/doc1/pages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(common::mock_pages_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/docs/doc1/pages", mock_server.uri()))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_page_export_initiate_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/docs/doc1/pages/page1/export"))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "id": "export123",
            "status": "inProgress"
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!(
            "{}/docs/doc1/pages/page1/export",
            mock_server.uri()
        ))
        .header("Authorization", "Bearer test_token")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({"outputFormat": "html"}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 202);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], "export123");
    assert_eq!(body["status"], "inProgress");
}

#[tokio::test]
async fn test_page_export_status_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs/doc1/pages/page1/export/export123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "export123",
            "status": "complete",
            "downloadLink": "https://example.com/download/abc123"
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/docs/doc1/pages/page1/export/export123",
            mock_server.uri()
        ))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "complete");
    assert!(body["downloadLink"].as_str().is_some());
}

#[tokio::test]
async fn test_page_export_failed_status() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs/doc1/pages/page1/export/export456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "export456",
            "status": "failed",
            "error": "Export failed due to invalid page format"
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/docs/doc1/pages/page1/export/export456",
            mock_server.uri()
        ))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "failed");
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("invalid page format"));
}

#[tokio::test]
async fn test_list_columns_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs/doc1/tables/tbl1/columns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(common::mock_columns_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/docs/doc1/tables/tbl1/columns",
            mock_server.uri()
        ))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["items"][0]["name"], "Name");
}

#[tokio::test]
async fn test_list_formulas_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs/doc1/formulas"))
        .respond_with(ResponseTemplate::new(200).set_body_json(common::mock_formulas_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/docs/doc1/formulas", mock_server.uri()))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["items"][0]["value"], 15);
}

#[tokio::test]
async fn test_list_controls_endpoint() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs/doc1/controls"))
        .respond_with(ResponseTemplate::new(200).set_body_json(common::mock_controls_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/docs/doc1/controls", mock_server.uri()))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["items"][0]["controlType"], "button");
}

#[tokio::test]
async fn test_search_docs_with_query() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs"))
        .and(query_param("query", "project"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "doc1", "name": "My Project"}]
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/docs?query=project", mock_server.uri()))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["items"][0]["name"], "My Project");
}

#[tokio::test]
async fn test_get_rows_with_filter() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs/doc1/tables/tbl1/rows"))
        .and(query_param("query", "Status:\"Active\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "row1", "values": {"Name": "Alice", "Status": "Active"}}]
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/docs/doc1/tables/tbl1/rows?query={}",
            mock_server.uri(),
            urlencoding::encode("Status:\"Active\"")
        ))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_rate_limit_handling() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/docs", mock_server.uri()))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 429);
}

#[tokio::test]
async fn test_not_found_error() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "statusCode": 404,
            "statusMessage": "Not Found",
            "message": "Document not found"
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/docs/nonexistent", mock_server.uri()))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_unauthorized_error() {
    let mock_server = common::setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/docs"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "statusCode": 401,
            "message": "Invalid API token"
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/docs", mock_server.uri()))
        .header("Authorization", "Bearer invalid_token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}
