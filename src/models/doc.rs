use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doc {
    pub id: String,
    #[serde(rename = "type")]
    pub doc_type: Option<String>,
    pub href: Option<String>,
    pub name: String,
    pub owner: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "folderId")]
    pub folder_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocList {
    pub items: Vec<Doc>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDocsParams {
    /// Maximum number of docs to return (default: 50)
    pub limit: Option<u32>,
    /// Search query to filter docs by name
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDocParams {
    /// The document ID
    pub doc_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchDocsParams {
    /// Search query
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDocParams {
    /// Title for the new document
    pub title: String,
    /// Folder ID to create document in (optional)
    #[serde(rename = "folderId", skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    /// Source document ID to copy from (optional, for templates)
    #[serde(rename = "sourceDoc", skip_serializing_if = "Option::is_none")]
    pub source_doc: Option<String>,
    /// Timezone for the document (optional, e.g., `America/Los_Angeles`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteDocParams {
    /// The document ID to delete
    pub doc_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_deserialize() {
        let json = r#"{
            "id": "doc123",
            "type": "doc",
            "href": "https://coda.io/apis/v1/docs/doc123",
            "name": "Test Doc",
            "owner": "user@example.com",
            "createdAt": "2024-01-01T00:00:00Z",
            "updatedAt": "2024-01-02T00:00:00Z",
            "folderId": "folder456"
        }"#;

        let doc: Doc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.id, "doc123");
        assert_eq!(doc.name, "Test Doc");
        assert_eq!(doc.owner, Some("user@example.com".to_string()));
        assert_eq!(doc.folder_id, Some("folder456".to_string()));
    }

    #[test]
    fn test_doc_deserialize_minimal() {
        let json = r#"{"id": "doc123", "name": "Test Doc"}"#;
        let doc: Doc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.id, "doc123");
        assert_eq!(doc.name, "Test Doc");
        assert!(doc.owner.is_none());
    }

    #[test]
    fn test_doc_list_deserialize() {
        let json = r#"{
            "items": [
                {"id": "doc1", "name": "Doc 1"},
                {"id": "doc2", "name": "Doc 2"}
            ],
            "nextPageToken": "token123"
        }"#;

        let list: DocList = serde_json::from_str(json).unwrap();
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].id, "doc1");
        assert_eq!(list.next_page_token, Some("token123".to_string()));
    }

    #[test]
    fn test_doc_serialize() {
        let doc = Doc {
            id: "doc123".to_string(),
            doc_type: Some("doc".to_string()),
            href: None,
            name: "Test".to_string(),
            owner: None,
            created_at: None,
            updated_at: None,
            folder_id: None,
        };

        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("doc123"));
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_list_docs_params_defaults() {
        let json = r"{}";
        let params: ListDocsParams = serde_json::from_str(json).unwrap();
        assert!(params.limit.is_none());
        assert!(params.query.is_none());
    }

    #[test]
    fn test_list_docs_params_with_values() {
        let json = r#"{"limit": 10, "query": "test"}"#;
        let params: ListDocsParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.query, Some("test".to_string()));
    }

    #[test]
    fn test_create_doc_params_minimal() {
        let json = r#"{"title": "My New Doc"}"#;
        let params: CreateDocParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.title, "My New Doc");
        assert!(params.folder_id.is_none());
        assert!(params.source_doc.is_none());
        assert!(params.timezone.is_none());
    }

    #[test]
    fn test_create_doc_params_full() {
        let json = r#"{
            "title": "My New Doc",
            "folderId": "folder123",
            "sourceDoc": "template456",
            "timezone": "America/Los_Angeles"
        }"#;
        let params: CreateDocParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.title, "My New Doc");
        assert_eq!(params.folder_id, Some("folder123".to_string()));
        assert_eq!(params.source_doc, Some("template456".to_string()));
        assert_eq!(params.timezone, Some("America/Los_Angeles".to_string()));
    }

    #[test]
    fn test_create_doc_params_serialize_minimal() {
        let params = CreateDocParams {
            title: "Test".to_string(),
            folder_id: None,
            source_doc: None,
            timezone: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"title\":\"Test\""));
        assert!(!json.contains("folderId"));
        assert!(!json.contains("sourceDoc"));
        assert!(!json.contains("timezone"));
    }

    #[test]
    fn test_delete_doc_params() {
        let json = r#"{"doc_id": "doc123"}"#;
        let params: DeleteDocParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.doc_id, "doc123");
    }
}
