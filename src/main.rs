use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use std::fmt::Write as _;

#[cfg(not(test))]
const MAX_POLL_ATTEMPTS: u32 = 30;
#[cfg(not(test))]
const POLL_INTERVAL_SECS: u64 = 1;

#[cfg(test)]
const MAX_POLL_ATTEMPTS: u32 = 3;
#[cfg(test)]
const POLL_INTERVAL_SECS: u64 = 0;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

mod client;
mod config;
mod error;
mod models;

use client::CodaClient;
use config::Config;
use models::{
    AddRowParams, ColumnList, ControlList, CreateDocParams, DeleteDocParams, DeleteRowParams, Doc,
    DocList, ExportRequest, ExportResponse, Formula, FormulaList, GetDocParams, GetFormulaParams,
    GetPageParams, GetRowParams, GetRowsParams, GetTableParams, ListColumnsParams,
    ListControlsParams, ListDocsParams, ListFormulasParams, ListPagesParams, ListTablesParams,
    Page, PageList, Row, RowList, RowMutationResponse, SearchDocsParams, Table, TableList,
    UpdateRowParams,
};

#[derive(Clone)]
pub struct CodaMcpServer {
    client: Arc<CodaClient>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl CodaMcpServer {
    pub fn new(client: Arc<CodaClient>) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    // === Document Tools ===

    #[tool(description = "List available Coda documents. Returns doc IDs, names, and metadata.")]
    async fn list_docs(
        &self,
        Parameters(params): Parameters<ListDocsParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = params.limit.unwrap_or(50).min(1000);
        let mut path = format!("/docs?limit={limit}");

        if let Some(query) = &params.query {
            let _ = write!(path, "&query={}", urlencoding::encode(query));
        }

        tracing::info!("list_docs: limit={}, query={:?}", limit, params.query);

        let docs: DocList = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let summary = format!("Found {} documents", docs.items.len());
        let json = serde_json::to_string_pretty(&docs.items)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "{summary}\n\n```json\n{json}\n```"
        ))]))
    }

    #[tool(description = "Get detailed information about a specific Coda document.")]
    async fn get_doc(
        &self,
        Parameters(params): Parameters<GetDocParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/docs/{}", params.doc_id);

        tracing::info!("get_doc: doc_id={}", params.doc_id);

        let doc: Doc = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&doc)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Document: {}\n\n```json\n{}\n```",
            doc.name, json
        ))]))
    }

    #[tool(description = "Search for Coda documents by name or content.")]
    async fn search_docs(
        &self,
        Parameters(params): Parameters<SearchDocsParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/docs?query={}", urlencoding::encode(&params.query));

        tracing::info!("search_docs: query={}", params.query);

        let docs: DocList = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let summary = format!(
            "Found {} documents matching '{}'",
            docs.items.len(),
            params.query
        );
        let json = serde_json::to_string_pretty(&docs.items)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "{summary}\n\n```json\n{json}\n```"
        ))]))
    }

    #[tool(
        description = "Create a new Coda document. Optionally specify a folder, source document (template), or timezone."
    )]
    async fn create_doc(
        &self,
        Parameters(params): Parameters<CreateDocParams>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!(
            "create_doc: title={}, folder_id={:?}, source_doc={:?}, timezone={:?}",
            params.title,
            params.folder_id,
            params.source_doc,
            params.timezone
        );

        let doc: Doc = match self.client.post("/docs", &params).await {
            Ok(doc) => doc,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(e.to_string())]));
            }
        };

        let json = serde_json::to_string_pretty(&doc)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Document created successfully!\n\nName: {}\nID: {}\n\n```json\n{}\n```",
            doc.name, doc.id, json
        ))]))
    }

    #[tool(description = "Delete a Coda document. This action is permanent and cannot be undone.")]
    async fn delete_doc(
        &self,
        Parameters(params): Parameters<DeleteDocParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/docs/{}", params.doc_id);

        tracing::info!("delete_doc: doc_id={}", params.doc_id);

        if let Err(e) = self.client.delete(&path).await {
            return Ok(CallToolResult::error(vec![Content::text(e.to_string())]));
        }

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Document '{}' deleted successfully.",
            params.doc_id
        ))]))
    }

    // === Page Tools ===

    #[tool(description = "List all pages in a Coda document.")]
    async fn list_pages(
        &self,
        Parameters(params): Parameters<ListPagesParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/docs/{}/pages", params.doc_id);

        tracing::info!("list_pages: doc_id={}", params.doc_id);

        let pages: PageList = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let summary = format!("Found {} pages", pages.items.len());
        let json = serde_json::to_string_pretty(&pages.items)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "{summary}\n\n```json\n{json}\n```"
        ))]))
    }

    #[tool(description = "Get a specific page's content in HTML format.")]
    async fn get_page(
        &self,
        Parameters(params): Parameters<GetPageParams>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!(
            "get_page: doc_id={}, page_id={}",
            params.doc_id,
            params.page_id
        );

        // Step 1: Initiate export
        let export_path = format!("/docs/{}/pages/{}/export", params.doc_id, params.page_id);
        let export_request = ExportRequest {
            output_format: "html".to_string(),
        };

        tracing::info!("Initiating page export: POST {}", export_path);
        let export: ExportResponse = self
            .client
            .post(&export_path, &export_request)
            .await
            .map_err(|e| {
                tracing::error!("Failed to initiate export: {}", e);
                McpError::internal_error(e.to_string(), None)
            })?;
        tracing::info!(
            "Export initiated: id={}, status={}",
            export.id,
            export.status
        );

        // Step 2: Poll for completion (max 30 attempts, 1s interval)
        let status_path = format!(
            "/docs/{}/pages/{}/export/{}",
            params.doc_id, params.page_id, export.id
        );

        for attempt in 1..=MAX_POLL_ATTEMPTS {
            tracing::info!(
                "Polling export status, attempt {}/{}: GET {}",
                attempt,
                MAX_POLL_ATTEMPTS,
                status_path
            );

            let status: ExportResponse = self.client.get(&status_path).await.map_err(|e| {
                tracing::error!("Failed to poll export status: {}", e);
                McpError::internal_error(e.to_string(), None)
            })?;
            tracing::info!("Export status: {}", status.status);

            match status.status.as_str() {
                "complete" => {
                    // Step 3: Download content from temporary link
                    let download_link = status.download_link.ok_or_else(|| {
                        McpError::internal_error(
                            "Export complete but no download link provided".to_string(),
                            None,
                        )
                    })?;

                    tracing::info!("Export complete, downloading from: {}", download_link);
                    let content = self
                        .client
                        .download_raw(&download_link)
                        .await
                        .map_err(|e| {
                            tracing::error!("Failed to download export: {}", e);
                            McpError::internal_error(e.to_string(), None)
                        })?;
                    tracing::info!("Downloaded {} bytes", content.len());

                    // Get page metadata for the name
                    let page_path = format!("/docs/{}/pages/{}", params.doc_id, params.page_id);
                    let page: Page = self
                        .client
                        .get(&page_path)
                        .await
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "Page: {}\n\nContent:\n{}",
                        page.name, content
                    ))]));
                }
                "failed" => {
                    let error_msg = status.error.unwrap_or_else(|| "Unknown error".to_string());
                    return Err(McpError::internal_error(
                        format!("Export failed: {error_msg}"),
                        None,
                    ));
                }
                _ => {
                    // Still processing, wait and retry
                    tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
                }
            }
        }

        Err(McpError::internal_error(
            format!(
                "Export timed out after {} seconds",
                u64::from(MAX_POLL_ATTEMPTS) * POLL_INTERVAL_SECS
            ),
            None,
        ))
    }

    // === Table Tools ===

    #[tool(description = "List all tables in a Coda document.")]
    async fn list_tables(
        &self,
        Parameters(params): Parameters<ListTablesParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/docs/{}/tables", params.doc_id);

        tracing::info!("list_tables: doc_id={}", params.doc_id);

        let tables: TableList = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let summary = format!("Found {} tables", tables.items.len());
        let json = serde_json::to_string_pretty(&tables.items)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "{summary}\n\n```json\n{json}\n```"
        ))]))
    }

    #[tool(description = "Get detailed information about a specific table.")]
    async fn get_table(
        &self,
        Parameters(params): Parameters<GetTableParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/docs/{}/tables/{}", params.doc_id, params.table_id);

        tracing::info!(
            "get_table: doc_id={}, table_id={}",
            params.doc_id,
            params.table_id
        );

        let table: Table = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&table)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Table: {}\n\n```json\n{}\n```",
            table.name, json
        ))]))
    }

    #[tool(description = "List all columns in a table.")]
    async fn list_columns(
        &self,
        Parameters(params): Parameters<ListColumnsParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/docs/{}/tables/{}/columns", params.doc_id, params.table_id);

        tracing::info!(
            "list_columns: doc_id={}, table_id={}",
            params.doc_id,
            params.table_id
        );

        let columns: ColumnList = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let summary = format!("Found {} columns", columns.items.len());
        let json = serde_json::to_string_pretty(&columns.items)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "{summary}\n\n```json\n{json}\n```"
        ))]))
    }

    // === Row Tools ===

    #[tool(
        description = "Get rows from a table with optional filtering. Returns rows with column values using column names as keys."
    )]
    async fn get_rows(
        &self,
        Parameters(params): Parameters<GetRowsParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = params.limit.unwrap_or(100).min(1000);
        let mut path = format!(
            "/docs/{}/tables/{}/rows?limit={}&useColumnNames=true",
            params.doc_id, params.table_id, limit
        );

        if let Some(query) = &params.query {
            let _ = write!(path, "&query={}", urlencoding::encode(query));
        }

        tracing::info!(
            "get_rows: doc_id={}, table_id={}, limit={}, query={:?}",
            params.doc_id,
            params.table_id,
            limit,
            params.query
        );

        let rows: RowList = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let summary = format!("Found {} rows", rows.items.len());
        let json = serde_json::to_string_pretty(&rows.items)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "{summary}\n\n```json\n{json}\n```"
        ))]))
    }

    #[tool(description = "Get a specific row by ID.")]
    async fn get_row(
        &self,
        Parameters(params): Parameters<GetRowParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!(
            "/docs/{}/tables/{}/rows/{}?useColumnNames=true",
            params.doc_id, params.table_id, params.row_id
        );

        tracing::info!(
            "get_row: doc_id={}, table_id={}, row_id={}",
            params.doc_id,
            params.table_id,
            params.row_id
        );

        let row: Row = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&row)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Row: {}\n\n```json\n{}\n```",
            row.id, json
        ))]))
    }

    #[tool(
        description = "Add a new row to a table. Cells should be a dictionary mapping column names to values."
    )]
    async fn add_row(
        &self,
        Parameters(params): Parameters<AddRowParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/docs/{}/tables/{}/rows", params.doc_id, params.table_id);

        let cells: Vec<serde_json::Value> = params
            .cells
            .iter()
            .map(|(col, val)| {
                serde_json::json!({
                    "column": col,
                    "value": val
                })
            })
            .collect();

        let body = serde_json::json!({
            "rows": [{
                "cells": cells
            }]
        });

        tracing::info!(
            "add_row: doc_id={}, table_id={}, cells={:?}",
            params.doc_id,
            params.table_id,
            params.cells
        );

        let result: RowMutationResponse = self
            .client
            .post(&path, &body)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let added_ids = result
            .added_row_ids
            .map(|ids| ids.join(", "))
            .unwrap_or_default();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Row added successfully.\nRequest ID: {}\nAdded row IDs: {}\n\nNote: Changes may take a few seconds to appear.",
            result.request_id, added_ids
        ))]))
    }

    #[tool(description = "Update an existing row in a table.")]
    async fn update_row(
        &self,
        Parameters(params): Parameters<UpdateRowParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!(
            "/docs/{}/tables/{}/rows/{}",
            params.doc_id, params.table_id, params.row_id
        );

        let cells: Vec<serde_json::Value> = params
            .cells
            .iter()
            .map(|(col, val)| {
                serde_json::json!({
                    "column": col,
                    "value": val
                })
            })
            .collect();

        let body = serde_json::json!({
            "row": {
                "cells": cells
            }
        });

        tracing::info!(
            "update_row: doc_id={}, table_id={}, row_id={}",
            params.doc_id,
            params.table_id,
            params.row_id
        );

        let result: RowMutationResponse = self
            .client
            .put(&path, &body)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Row updated successfully.\nRequest ID: {}\n\nNote: Changes may take a few seconds to appear.",
            result.request_id
        ))]))
    }

    #[tool(description = "Delete a row from a table.")]
    async fn delete_row(
        &self,
        Parameters(params): Parameters<DeleteRowParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!(
            "/docs/{}/tables/{}/rows/{}",
            params.doc_id, params.table_id, params.row_id
        );

        tracing::info!(
            "delete_row: doc_id={}, table_id={}, row_id={}",
            params.doc_id,
            params.table_id,
            params.row_id
        );

        self.client
            .delete(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            "Row deleted successfully.\n\nNote: Changes may take a few seconds to appear."
                .to_string(),
        )]))
    }

    // === Formula Tools ===

    #[tool(description = "List all named formulas in a document.")]
    async fn list_formulas(
        &self,
        Parameters(params): Parameters<ListFormulasParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/docs/{}/formulas", params.doc_id);

        tracing::info!("list_formulas: doc_id={}", params.doc_id);

        let formulas: FormulaList = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let summary = format!("Found {} formulas", formulas.items.len());
        let json = serde_json::to_string_pretty(&formulas.items)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "{summary}\n\n```json\n{json}\n```"
        ))]))
    }

    #[tool(description = "Get a specific formula's current value.")]
    async fn get_formula(
        &self,
        Parameters(params): Parameters<GetFormulaParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/docs/{}/formulas/{}", params.doc_id, params.formula_id);

        tracing::info!(
            "get_formula: doc_id={}, formula_id={}",
            params.doc_id,
            params.formula_id
        );

        let formula: Formula = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&formula)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Formula: {}\n\n```json\n{}\n```",
            formula.name, json
        ))]))
    }

    // === Control Tools ===

    #[tool(description = "List all controls (buttons, sliders, etc.) in a document.")]
    async fn list_controls(
        &self,
        Parameters(params): Parameters<ListControlsParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/docs/{}/controls", params.doc_id);

        tracing::info!("list_controls: doc_id={}", params.doc_id);

        let controls: ControlList = self
            .client
            .get(&path)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let summary = format!("Found {} controls", controls.items.len());
        let json = serde_json::to_string_pretty(&controls.items)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "{summary}\n\n```json\n{json}\n```"
        ))]))
    }
}

#[tool_handler]
impl ServerHandler for CodaMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Coda.io MCP Server - Interact with Coda documents, tables, and rows. \
                 Requires CODA_API_TOKEN environment variable."
                    .into(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup() -> (CodaMcpServer, MockServer) {
        let mock_server = MockServer::start().await;
        let client = Arc::new(CodaClient::new_with_base_url(
            "test_token",
            &mock_server.uri(),
        ));
        let server = CodaMcpServer::new(client);
        (server, mock_server)
    }

    // === Server Info ===

    #[test]
    fn test_get_info() {
        let mock_client = CodaClient::new_with_base_url("tok", "http://localhost:0");
        let server = CodaMcpServer::new(Arc::new(mock_client));
        let info = server.get_info();
        // from_build_env() uses the rmcp crate name, not our package name
        assert!(!info.server_info.name.is_empty());
        assert!(!info.server_info.version.is_empty());
        assert!(info.instructions.is_some());
        assert!(info.instructions.unwrap().contains("Coda.io MCP Server"));
    }

    // === Document Tools ===

    #[tokio::test]
    async fn test_list_docs_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs"))
            .and(query_param("limit", "50"))
            .and(header("Authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [
                    {"id": "doc1", "name": "Doc One"},
                    {"id": "doc2", "name": "Doc Two"}
                ]
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .list_docs(Parameters(ListDocsParams {
                limit: None,
                query: None,
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 2 documents"));
        assert!(text.contains("Doc One"));
    }

    #[tokio::test]
    async fn test_list_docs_with_query() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs"))
            .and(query_param("query", "project"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [{"id": "doc1", "name": "My Project"}]
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .list_docs(Parameters(ListDocsParams {
                limit: Some(10),
                query: Some("project".to_string()),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 1 documents"));
    }

    #[tokio::test]
    async fn test_list_docs_limit_capped_at_1000() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs"))
            .and(query_param("limit", "1000"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": []
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .list_docs(Parameters(ListDocsParams {
                limit: Some(5000),
                query: None,
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 0 documents"));
    }

    #[tokio::test]
    async fn test_list_docs_api_error() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let result = server
            .list_docs(Parameters(ListDocsParams {
                limit: None,
                query: None,
            }))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_doc_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "doc1",
                "name": "Test Document"
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .get_doc(Parameters(GetDocParams {
                doc_id: "doc1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Document: Test Document"));
    }

    #[tokio::test]
    async fn test_search_docs_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs"))
            .and(query_param("query", "hello"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [{"id": "d1", "name": "Hello World"}]
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .search_docs(Parameters(SearchDocsParams {
                query: "hello".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 1 documents matching 'hello'"));
    }

    #[tokio::test]
    async fn test_create_doc_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("POST"))
            .and(path("/docs"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "id": "new-doc",
                "name": "My New Doc"
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .create_doc(Parameters(CreateDocParams {
                title: "My New Doc".to_string(),
                folder_id: None,
                source_doc: None,
                timezone: None,
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Document created successfully"));
        assert!(text.contains("My New Doc"));
    }

    #[tokio::test]
    async fn test_create_doc_with_all_options() {
        let (server, mock_server) = setup().await;

        Mock::given(method("POST"))
            .and(path("/docs"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "id": "new-doc",
                "name": "From Template"
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .create_doc(Parameters(CreateDocParams {
                title: "From Template".to_string(),
                folder_id: Some("folder1".to_string()),
                source_doc: Some("template1".to_string()),
                timezone: Some("Europe/London".to_string()),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Document created successfully"));
    }

    #[tokio::test]
    async fn test_create_doc_api_error_returns_tool_error() {
        let (server, mock_server) = setup().await;

        Mock::given(method("POST"))
            .and(path("/docs"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&mock_server)
            .await;

        let result = server
            .create_doc(Parameters(CreateDocParams {
                title: "Forbidden".to_string(),
                folder_id: None,
                source_doc: None,
                timezone: None,
            }))
            .await
            .unwrap();

        // create_doc returns CallToolResult::error, not Err
        assert!(result.is_error.unwrap_or(false));
    }

    #[tokio::test]
    async fn test_delete_doc_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("DELETE"))
            .and(path("/docs/doc1"))
            .respond_with(ResponseTemplate::new(202))
            .mount(&mock_server)
            .await;

        let result = server
            .delete_doc(Parameters(DeleteDocParams {
                doc_id: "doc1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("deleted successfully"));
    }

    #[tokio::test]
    async fn test_delete_doc_error_returns_tool_error() {
        let (server, mock_server) = setup().await;

        Mock::given(method("DELETE"))
            .and(path("/docs/doc1"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let result = server
            .delete_doc(Parameters(DeleteDocParams {
                doc_id: "doc1".to_string(),
            }))
            .await
            .unwrap();

        assert!(result.is_error.unwrap_or(false));
    }

    // === Page Tools ===

    #[tokio::test]
    async fn test_list_pages_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/pages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [
                    {"id": "p1", "name": "Home"},
                    {"id": "p2", "name": "About"}
                ]
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .list_pages(Parameters(ListPagesParams {
                doc_id: "doc1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 2 pages"));
    }

    #[tokio::test]
    async fn test_get_page_export_failed() {
        let (server, mock_server) = setup().await;

        // Step 1: Initiate export
        Mock::given(method("POST"))
            .and(path("/docs/doc1/pages/p1/export"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "id": "exp1",
                "status": "inProgress"
            })))
            .mount(&mock_server)
            .await;

        // Step 2: Poll returns failed
        Mock::given(method("GET"))
            .and(path("/docs/doc1/pages/p1/export/exp1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "exp1",
                "status": "failed",
                "error": "Page too large"
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .get_page(Parameters(GetPageParams {
                doc_id: "doc1".to_string(),
                page_id: "p1".to_string(),
            }))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Export failed"));
    }

    #[tokio::test]
    async fn test_get_page_complete_no_download_link() {
        let (server, mock_server) = setup().await;

        Mock::given(method("POST"))
            .and(path("/docs/doc1/pages/p1/export"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "id": "exp1",
                "status": "inProgress"
            })))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/pages/p1/export/exp1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "exp1",
                "status": "complete"
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .get_page(Parameters(GetPageParams {
                doc_id: "doc1".to_string(),
                page_id: "p1".to_string(),
            }))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("no download link"));
    }

    // === Table Tools ===

    #[tokio::test]
    async fn test_list_tables_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/tables"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [
                    {"id": "tbl1", "name": "Tasks", "rowCount": 42}
                ]
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .list_tables(Parameters(ListTablesParams {
                doc_id: "doc1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 1 tables"));
        assert!(text.contains("Tasks"));
    }

    #[tokio::test]
    async fn test_get_table_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/tables/tbl1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "tbl1",
                "name": "Tasks",
                "rowCount": 42
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .get_table(Parameters(GetTableParams {
                doc_id: "doc1".to_string(),
                table_id: "tbl1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Table: Tasks"));
    }

    #[tokio::test]
    async fn test_list_columns_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/tables/tbl1/columns"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [
                    {"id": "col1", "name": "Name"},
                    {"id": "col2", "name": "Status"}
                ]
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .list_columns(Parameters(ListColumnsParams {
                doc_id: "doc1".to_string(),
                table_id: "tbl1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 2 columns"));
    }

    // === Row Tools ===

    #[tokio::test]
    async fn test_get_rows_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/tables/tbl1/rows"))
            .and(query_param("useColumnNames", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [
                    {"id": "r1", "name": "Row 1", "values": {"Name": "Alice"}},
                    {"id": "r2", "name": "Row 2", "values": {"Name": "Bob"}}
                ]
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .get_rows(Parameters(GetRowsParams {
                doc_id: "doc1".to_string(),
                table_id: "tbl1".to_string(),
                limit: None,
                query: None,
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 2 rows"));
    }

    #[tokio::test]
    async fn test_get_rows_with_query() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/tables/tbl1/rows"))
            .and(query_param("query", "Status:\"Active\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [{"id": "r1", "name": "Row 1", "values": {"Status": "Active"}}]
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .get_rows(Parameters(GetRowsParams {
                doc_id: "doc1".to_string(),
                table_id: "tbl1".to_string(),
                limit: Some(10),
                query: Some("Status:\"Active\"".to_string()),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 1 rows"));
    }

    #[tokio::test]
    async fn test_get_rows_limit_capped() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/tables/tbl1/rows"))
            .and(query_param("limit", "1000"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": []
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .get_rows(Parameters(GetRowsParams {
                doc_id: "doc1".to_string(),
                table_id: "tbl1".to_string(),
                limit: Some(9999),
                query: None,
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 0 rows"));
    }

    #[tokio::test]
    async fn test_get_row_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/tables/tbl1/rows/r1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "r1",
                "name": "Row 1",
                "values": {"Name": "Alice", "Score": 95}
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .get_row(Parameters(GetRowParams {
                doc_id: "doc1".to_string(),
                table_id: "tbl1".to_string(),
                row_id: "r1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Row: r1"));
    }

    #[tokio::test]
    async fn test_add_row_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("POST"))
            .and(path("/docs/doc1/tables/tbl1/rows"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "requestId": "req-abc",
                "addedRowIds": ["new-row-1"]
            })))
            .mount(&mock_server)
            .await;

        let mut cells = std::collections::HashMap::new();
        cells.insert(
            "Name".to_string(),
            serde_json::Value::String("Charlie".to_string()),
        );
        cells.insert(
            "Score".to_string(),
            serde_json::Value::Number(serde_json::Number::from(100)),
        );

        let result = server
            .add_row(Parameters(AddRowParams {
                doc_id: "doc1".to_string(),
                table_id: "tbl1".to_string(),
                cells,
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Row added successfully"));
        assert!(text.contains("req-abc"));
        assert!(text.contains("new-row-1"));
    }

    #[tokio::test]
    async fn test_update_row_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("PUT"))
            .and(path("/docs/doc1/tables/tbl1/rows/r1"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "requestId": "req-xyz"
            })))
            .mount(&mock_server)
            .await;

        let mut cells = std::collections::HashMap::new();
        cells.insert(
            "Status".to_string(),
            serde_json::Value::String("Done".to_string()),
        );

        let result = server
            .update_row(Parameters(UpdateRowParams {
                doc_id: "doc1".to_string(),
                table_id: "tbl1".to_string(),
                row_id: "r1".to_string(),
                cells,
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Row updated successfully"));
        assert!(text.contains("req-xyz"));
    }

    #[tokio::test]
    async fn test_delete_row_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("DELETE"))
            .and(path("/docs/doc1/tables/tbl1/rows/r1"))
            .respond_with(ResponseTemplate::new(202))
            .mount(&mock_server)
            .await;

        let result = server
            .delete_row(Parameters(DeleteRowParams {
                doc_id: "doc1".to_string(),
                table_id: "tbl1".to_string(),
                row_id: "r1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Row deleted successfully"));
    }

    #[tokio::test]
    async fn test_delete_row_error() {
        let (server, mock_server) = setup().await;

        Mock::given(method("DELETE"))
            .and(path("/docs/doc1/tables/tbl1/rows/r1"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let result = server
            .delete_row(Parameters(DeleteRowParams {
                doc_id: "doc1".to_string(),
                table_id: "tbl1".to_string(),
                row_id: "r1".to_string(),
            }))
            .await;

        assert!(result.is_err());
    }

    // === Formula Tools ===

    #[tokio::test]
    async fn test_list_formulas_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/formulas"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [
                    {"id": "f1", "name": "Total", "value": 42}
                ]
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .list_formulas(Parameters(ListFormulasParams {
                doc_id: "doc1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 1 formulas"));
    }

    #[tokio::test]
    async fn test_get_formula_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/formulas/f1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "f1",
                "name": "Total",
                "value": 42
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .get_formula(Parameters(GetFormulaParams {
                doc_id: "doc1".to_string(),
                formula_id: "f1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Formula: Total"));
    }

    // === Control Tools ===

    #[tokio::test]
    async fn test_list_controls_success() {
        let (server, mock_server) = setup().await;

        Mock::given(method("GET"))
            .and(path("/docs/doc1/controls"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [
                    {"id": "c1", "name": "Submit", "controlType": "button"},
                    {"id": "c2", "name": "Progress", "controlType": "slider", "value": 75}
                ]
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .list_controls(Parameters(ListControlsParams {
                doc_id: "doc1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Found 2 controls"));
    }

    // === get_page full success workflow ===

    #[tokio::test]
    async fn test_get_page_success() {
        let (server, mock_server) = setup().await;

        // Step 1: Initiate export
        Mock::given(method("POST"))
            .and(path("/docs/doc1/pages/p1/export"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "id": "exp1",
                "status": "inProgress"
            })))
            .mount(&mock_server)
            .await;

        // Step 2: Poll returns complete with downloadLink pointing at mock server
        let download_url = format!("{}/export/content.html", mock_server.uri());
        Mock::given(method("GET"))
            .and(path("/docs/doc1/pages/p1/export/exp1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "exp1",
                "status": "complete",
                "downloadLink": download_url
            })))
            .mount(&mock_server)
            .await;

        // Step 3: Download content from the link
        Mock::given(method("GET"))
            .and(path("/export/content.html"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("<html><body>Page content here</body></html>"),
            )
            .mount(&mock_server)
            .await;

        // Step 4: Get page metadata
        Mock::given(method("GET"))
            .and(path("/docs/doc1/pages/p1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "p1",
                "name": "Welcome Page"
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .get_page(Parameters(GetPageParams {
                doc_id: "doc1".to_string(),
                page_id: "p1".to_string(),
            }))
            .await
            .unwrap();

        let text = &result.content[0].raw.as_text().unwrap().text;
        assert!(text.contains("Page: Welcome Page"));
        assert!(text.contains("Page content here"));
    }

    #[tokio::test]
    async fn test_get_page_export_initiation_error() {
        let (server, mock_server) = setup().await;

        // Export POST fails
        Mock::given(method("POST"))
            .and(path("/docs/doc1/pages/p1/export"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let result = server
            .get_page(Parameters(GetPageParams {
                doc_id: "doc1".to_string(),
                page_id: "p1".to_string(),
            }))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_page_poll_error() {
        let (server, mock_server) = setup().await;

        // Export succeeds
        Mock::given(method("POST"))
            .and(path("/docs/doc1/pages/p1/export"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "id": "exp1",
                "status": "inProgress"
            })))
            .mount(&mock_server)
            .await;

        // Poll returns error
        Mock::given(method("GET"))
            .and(path("/docs/doc1/pages/p1/export/exp1"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let result = server
            .get_page(Parameters(GetPageParams {
                doc_id: "doc1".to_string(),
                page_id: "p1".to_string(),
            }))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_page_export_timeout() {
        let (server, mock_server) = setup().await;

        // Export succeeds
        Mock::given(method("POST"))
            .and(path("/docs/doc1/pages/p1/export"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "id": "exp1",
                "status": "inProgress"
            })))
            .mount(&mock_server)
            .await;

        // Poll always returns inProgress — never completes
        Mock::given(method("GET"))
            .and(path("/docs/doc1/pages/p1/export/exp1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "exp1",
                "status": "inProgress"
            })))
            .mount(&mock_server)
            .await;

        let result = server
            .get_page(Parameters(GetPageParams {
                doc_id: "doc1".to_string(),
                page_id: "p1".to_string(),
            }))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("timed out"),
            "Expected timeout error, got: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn test_get_page_download_error() {
        let (server, mock_server) = setup().await;

        Mock::given(method("POST"))
            .and(path("/docs/doc1/pages/p1/export"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "id": "exp1",
                "status": "inProgress"
            })))
            .mount(&mock_server)
            .await;

        let download_url = format!("{}/export/content.html", mock_server.uri());
        Mock::given(method("GET"))
            .and(path("/docs/doc1/pages/p1/export/exp1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "exp1",
                "status": "complete",
                "downloadLink": download_url
            })))
            .mount(&mock_server)
            .await;

        // Download fails
        Mock::given(method("GET"))
            .and(path("/export/content.html"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
            .mount(&mock_server)
            .await;

        let result = server
            .get_page(Parameters(GetPageParams {
                doc_id: "doc1".to_string(),
                page_id: "p1".to_string(),
            }))
            .await;

        assert!(result.is_err());
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (MCP uses stdout for JSON-RPC)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting coda-mcp server v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded, base URL: {}", config.base_url);

    // Create HTTP client
    let client = Arc::new(CodaClient::new(&config));

    // Create and run MCP server
    let server = CodaMcpServer::new(client);
    let service = server.serve(stdio()).await?;

    tracing::info!("Server running, waiting for requests...");
    service.waiting().await?;

    Ok(())
}
