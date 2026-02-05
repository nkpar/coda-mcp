use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnFormat {
    #[serde(rename = "type")]
    pub format_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: String,
    #[serde(rename = "type")]
    pub column_type: Option<String>,
    pub href: Option<String>,
    pub name: String,
    pub format: Option<ColumnFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnList {
    pub items: Vec<Column>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListColumnsParams {
    /// The document ID
    pub doc_id: String,
    /// The table ID or name
    pub table_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_deserialize() {
        let json = r#"{
            "id": "col-abc",
            "type": "column",
            "href": "https://coda.io/apis/v1/docs/doc1/tables/tbl1/columns/col-abc",
            "name": "Status",
            "format": {"type": "select"}
        }"#;

        let col: Column = serde_json::from_str(json).unwrap();
        assert_eq!(col.id, "col-abc");
        assert_eq!(col.name, "Status");
        assert_eq!(col.format.unwrap().format_type, Some("select".to_string()));
    }

    #[test]
    fn test_column_without_format() {
        let json = r#"{"id": "col1", "name": "Name"}"#;
        let col: Column = serde_json::from_str(json).unwrap();
        assert!(col.format.is_none());
    }

    #[test]
    fn test_column_list_deserialize() {
        let json = r#"{
            "items": [
                {"id": "c1", "name": "Col 1"},
                {"id": "c2", "name": "Col 2", "format": {"type": "text"}}
            ]
        }"#;

        let list: ColumnList = serde_json::from_str(json).unwrap();
        assert_eq!(list.items.len(), 2);
        assert!(list.items[1].format.is_some());
    }
}
