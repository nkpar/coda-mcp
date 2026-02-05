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

const MAX_POLL_ATTEMPTS: u32 = 30;
const POLL_INTERVAL_SECS: u64 = 1;
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
    AddRowParams, ColumnList, ControlList, CreateDocParams, DeleteDocParams, DeleteRowParams,
    Doc, DocList, ExportRequest, ExportResponse, Formula, FormulaList, GetDocParams,
    GetFormulaParams, GetPageParams, GetRowParams, GetRowsParams, GetTableParams,
    ListColumnsParams, ListControlsParams, ListDocsParams, ListFormulasParams, ListPagesParams,
    ListTablesParams, Page, PageList, Row, RowList, RowMutationResponse, SearchDocsParams, Table,
    TableList, UpdateRowParams,
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
