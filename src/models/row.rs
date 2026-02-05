use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub id: String,
    #[serde(rename = "type")]
    pub row_type: Option<String>,
    pub href: Option<String>,
    pub name: Option<String>,
    pub index: Option<u32>,
    pub values: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowList {
    pub items: Vec<Row>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowMutationResponse {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "addedRowIds")]
    pub added_row_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRowsParams {
    /// The document ID
    pub doc_id: String,
    /// The table ID or name
    pub table_id: String,
    /// Maximum rows to return (default: 100)
    pub limit: Option<u32>,
    /// Query to filter rows (Coda formula syntax)
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRowParams {
    /// The document ID
    pub doc_id: String,
    /// The table ID or name
    pub table_id: String,
    /// The row ID
    pub row_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddRowParams {
    /// The document ID
    pub doc_id: String,
    /// The table ID or name
    pub table_id: String,
    /// Cell values as key-value pairs (column name -> value)
    pub cells: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateRowParams {
    /// The document ID
    pub doc_id: String,
    /// The table ID or name
    pub table_id: String,
    /// The row ID to update
    pub row_id: String,
    /// Cell values to update (column name -> value)
    pub cells: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteRowParams {
    /// The document ID
    pub doc_id: String,
    /// The table ID or name
    pub table_id: String,
    /// The row ID to delete
    pub row_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_deserialize() {
        let json = r#"{
            "id": "row123",
            "type": "row",
            "href": "https://coda.io/apis/v1/docs/doc1/tables/tbl1/rows/row123",
            "name": "Row 1",
            "index": 0,
            "values": {"Name": "John", "Age": 30}
        }"#;

        let row: Row = serde_json::from_str(json).unwrap();
        assert_eq!(row.id, "row123");
        assert_eq!(row.name, Some("Row 1".to_string()));
        assert_eq!(row.index, Some(0));

        let values = row.values.unwrap();
        assert_eq!(values.get("Name").unwrap(), "John");
        assert_eq!(values.get("Age").unwrap(), 30);
    }

    #[test]
    fn test_row_list_deserialize() {
        let json = r#"{
            "items": [
                {"id": "row1", "values": {"Col1": "val1"}},
                {"id": "row2", "values": {"Col1": "val2"}}
            ]
        }"#;

        let list: RowList = serde_json::from_str(json).unwrap();
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].id, "row1");
    }

    #[test]
    fn test_row_mutation_response() {
        let json = r#"{
            "requestId": "req123",
            "addedRowIds": ["row1", "row2"]
        }"#;

        let resp: RowMutationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.request_id, "req123");
        assert_eq!(
            resp.added_row_ids,
            Some(vec!["row1".to_string(), "row2".to_string()])
        );
    }

    #[test]
    fn test_add_row_params() {
        let json = r#"{
            "doc_id": "doc123",
            "table_id": "table456",
            "cells": {"Name": "John", "Email": "john@example.com"}
        }"#;

        let params: AddRowParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.doc_id, "doc123");
        assert_eq!(params.table_id, "table456");
        assert_eq!(params.cells.len(), 2);
        assert_eq!(params.cells.get("Name").unwrap(), "John");
    }

    #[test]
    fn test_get_rows_params_defaults() {
        let json = r#"{"doc_id": "doc1", "table_id": "tbl1"}"#;
        let params: GetRowsParams = serde_json::from_str(json).unwrap();
        assert!(params.limit.is_none());
        assert!(params.query.is_none());
    }

    #[test]
    fn test_get_rows_params_with_query() {
        let json =
            r#"{"doc_id": "doc1", "table_id": "tbl1", "limit": 50, "query": "Status:\"Active\""}"#;
        let params: GetRowsParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, Some(50));
        assert_eq!(params.query, Some("Status:\"Active\"".to_string()));
    }
}
