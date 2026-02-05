use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub id: String,
    #[serde(rename = "type")]
    pub table_type: Option<String>,
    pub href: Option<String>,
    pub name: String,
    #[serde(rename = "rowCount")]
    pub row_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableList {
    pub items: Vec<Table>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListTablesParams {
    /// The document ID
    pub doc_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetTableParams {
    /// The document ID
    pub doc_id: String,
    /// The table ID or name
    pub table_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_deserialize() {
        let json = r#"{
            "id": "grid-abc123",
            "type": "table",
            "href": "https://coda.io/apis/v1/docs/doc1/tables/grid-abc123",
            "name": "Tasks",
            "rowCount": 42
        }"#;

        let table: Table = serde_json::from_str(json).unwrap();
        assert_eq!(table.id, "grid-abc123");
        assert_eq!(table.name, "Tasks");
        assert_eq!(table.row_count, Some(42));
    }

    #[test]
    fn test_table_list_deserialize() {
        let json = r#"{
            "items": [
                {"id": "tbl1", "name": "Table 1"},
                {"id": "tbl2", "name": "Table 2", "rowCount": 10}
            ]
        }"#;

        let list: TableList = serde_json::from_str(json).unwrap();
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[1].row_count, Some(10));
    }

    #[test]
    fn test_table_serialize() {
        let table = Table {
            id: "tbl1".to_string(),
            table_type: Some("table".to_string()),
            href: None,
            name: "My Table".to_string(),
            row_count: Some(100),
        };

        let json = serde_json::to_string(&table).unwrap();
        assert!(json.contains("\"rowCount\":100"));
    }
}
