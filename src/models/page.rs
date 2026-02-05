use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageParent {
    pub id: String,
    #[serde(rename = "type")]
    pub parent_type: Option<String>,
    pub href: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    #[serde(rename = "type")]
    pub page_type: Option<String>,
    pub href: Option<String>,
    pub name: String,
    pub parent: Option<PageParent>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageList {
    pub items: Vec<Page>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContent {
    pub id: String,
    pub name: String,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListPagesParams {
    /// The document ID
    pub doc_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPageParams {
    /// The document ID
    pub doc_id: String,
    /// The page ID or name
    pub page_id: String,
}

// Export workflow types for canvas pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    #[serde(rename = "outputFormat")]
    pub output_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    pub id: String,
    pub status: String,
    /// URL to poll for export status
    pub href: Option<String>,
    #[serde(rename = "downloadLink")]
    pub download_link: Option<String>,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_request_serialize() {
        let req = ExportRequest {
            output_format: "html".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"outputFormat\":\"html\""));
    }

    #[test]
    fn test_export_response_deserialize() {
        let json = r#"{
            "id": "export123",
            "status": "complete",
            "downloadLink": "https://example.com/download"
        }"#;

        let resp: ExportResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "export123");
        assert_eq!(resp.status, "complete");
        assert_eq!(
            resp.download_link,
            Some("https://example.com/download".to_string())
        );
    }

    #[test]
    fn test_export_response_with_error() {
        let json = r#"{
            "id": "export123",
            "status": "failed",
            "error": "Export failed"
        }"#;

        let resp: ExportResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "failed");
        assert_eq!(resp.error, Some("Export failed".to_string()));
    }

    #[test]
    fn test_page_deserialize() {
        let json = r#"{
            "id": "page123",
            "type": "page",
            "href": "https://coda.io/apis/v1/docs/doc1/pages/page123",
            "name": "Home Page",
            "contentType": "canvas"
        }"#;

        let page: Page = serde_json::from_str(json).unwrap();
        assert_eq!(page.id, "page123");
        assert_eq!(page.name, "Home Page");
        assert_eq!(page.content_type, Some("canvas".to_string()));
    }

    #[test]
    fn test_page_with_parent() {
        let json = r#"{
            "id": "page123",
            "name": "Child Page",
            "parent": {
                "id": "page000",
                "type": "page",
                "name": "Parent Page"
            }
        }"#;

        let page: Page = serde_json::from_str(json).unwrap();
        let parent = page.parent.unwrap();
        assert_eq!(parent.id, "page000");
        assert_eq!(parent.name, Some("Parent Page".to_string()));
    }

    #[test]
    fn test_page_list_deserialize() {
        let json = r#"{
            "items": [
                {"id": "p1", "name": "Page 1"},
                {"id": "p2", "name": "Page 2"}
            ],
            "nextPageToken": "next123"
        }"#;

        let list: PageList = serde_json::from_str(json).unwrap();
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.next_page_token, Some("next123".to_string()));
    }

    #[test]
    fn test_page_content_deserialize() {
        let json = r#"{
            "id": "page123",
            "name": "Content Page",
            "contentType": "canvas",
            "content": "<h1>Hello World</h1>"
        }"#;

        let content: PageContent = serde_json::from_str(json).unwrap();
        assert_eq!(content.id, "page123");
        assert_eq!(content.content, Some("<h1>Hello World</h1>".to_string()));
    }
}
