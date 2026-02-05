# Coda MCP Server Specification

## Overview

MCP server for Coda.io API integration. Enables reading and writing documents, tables, and rows.

## Auth

- Environment variable: `CODA_API_TOKEN`
- Header: `Authorization: Bearer {token}`
- Base URL: `https://coda.io/apis/v1`

## Tools

### list_docs
List available documents.
- `limit: int = 50` — max count
- `query: str = ""` — filter by name

### get_doc
Get document metadata.
- `doc_id: str` — document ID

### list_pages
List pages in a document.
- `doc_id: str`

### get_page
Get page content (HTML). Uses async export workflow for canvas pages.
- `doc_id: str`
- `page_id: str`

**Workflow:**
1. POST `/docs/{doc_id}/pages/{page_id}/export` with `{"outputFormat": "html"}`
2. Poll GET `/docs/{doc_id}/pages/{page_id}/export/{export_id}` until status is `complete`
3. Download content from `downloadLink`

Max polling: 30 attempts, 1s interval (30s timeout)

### list_tables
List tables in a document.
- `doc_id: str`

### get_table
Get table metadata.
- `doc_id: str`
- `table_id: str`

### list_columns
List table columns.
- `doc_id: str`
- `table_id: str`

### get_rows
Get table rows.
- `doc_id: str`
- `table_id: str`
- `limit: int = 100`
- `query: str = ""` — filter using Coda formula syntax
- Query param: `useColumnNames=true`

### get_row
Get a single row.
- `doc_id: str`
- `table_id: str`
- `row_id: str`

### add_row
Add a new row.
- `doc_id: str`
- `table_id: str`
- `cells: dict` — `{column_name: value}`
- POST body: `{"rows": [{"cells": [{"column": k, "value": v}, ...]}]}`

### update_row
Update an existing row.
- `doc_id: str`
- `table_id: str`
- `row_id: str`
- `cells: dict`
- PUT body: `{"row": {"cells": [{"column": k, "value": v}, ...]}}`

### delete_row
Delete a row.
- `doc_id: str`
- `table_id: str`
- `row_id: str`

### search_docs
Search documents.
- `query: str`

### create_doc
Create a new document. Optionally specify folder, template (source document), or timezone.
- `title: str` — document title
- `folder_id: str = null` — folder ID (optional)
- `source_doc: str = null` — template document ID to copy from (optional)
- `timezone: str = null` — timezone (optional, e.g., "America/Los_Angeles")

### delete_doc
Delete a document. This action is permanent.
- `doc_id: str` — document ID to delete

### list_formulas
List named formulas in a document.
- `doc_id: str`

### get_formula
Get formula value.
- `doc_id: str`
- `formula_id: str`

### list_controls
List controls (buttons, sliders).
- `doc_id: str`

## API Endpoints

```
GET  /docs
POST /docs
GET  /docs/{doc_id}
DELETE /docs/{doc_id}
GET  /docs/{doc_id}/pages
GET  /docs/{doc_id}/pages/{page_id}
POST /docs/{doc_id}/pages/{page_id}/export
GET  /docs/{doc_id}/pages/{page_id}/export/{export_id}
GET  /docs/{doc_id}/tables
GET  /docs/{doc_id}/tables/{table_id}
GET  /docs/{doc_id}/tables/{table_id}/columns
GET  /docs/{doc_id}/tables/{table_id}/rows
GET  /docs/{doc_id}/tables/{table_id}/rows/{row_id}
POST /docs/{doc_id}/tables/{table_id}/rows
PUT  /docs/{doc_id}/tables/{table_id}/rows/{row_id}
DELETE /docs/{doc_id}/tables/{table_id}/rows/{row_id}
GET  /docs/{doc_id}/formulas
GET  /docs/{doc_id}/formulas/{formula_id}
GET  /docs/{doc_id}/controls
```

## Stack

- Rust with strong typing

## Notes

- All responses are JSON
- Row query syntax: `'ColumnName:"value"'`
- `useColumnNames=true` returns column names instead of IDs

## Developer Notes

### HTTP Client Configuration

The reqwest client is configured with specific settings to avoid Coda API issues:

```rust
Client::builder()
    .pool_max_idle_per_host(0)  // Disable connection pooling
    .timeout(Duration::from_secs(60))
    .connect_timeout(Duration::from_secs(30))
    .build()
```

**Why disable connection pooling (`pool_max_idle_per_host(0)`):**
- Investigation showed that curl requests work fine, but reqwest requests fail with 404
- Hypothesis: Coda API may mishandle HTTP/2 multiplexed requests on the same connection
- Disabling connection pooling forces each request to use a fresh connection

### Debugging

Run with verbose logging:

```bash
RUST_LOG=info CODA_API_TOKEN=xxx ./target/release/coda-mcp 2>&1 | tee /tmp/coda-debug.log
```

The client logs HTTP request URLs at INFO level when debugging. Response status is logged at DEBUG level.

### Dependencies

Tokio is configured with minimal features to reduce binary size:
- `macros` - for `#[tokio::main]` and `#[tokio::test]`
- `rt-multi-thread` - runtime
- `time` - for `tokio::time::sleep` in export polling

Other notable dependencies:
- `flate2` - required for decompressing raw gzip from external URLs (reqwest's auto-decompression doesn't work for these)
- `url` - required for security validation of download URLs

### Security

The following security measures are implemented:

1. **Token redaction**: The `Config` struct implements custom `Debug` to redact the API token from log output
2. **No token logging**: HTTP client does not log authorization headers or token previews
3. **URL validation**: `download_raw()` validates that download URLs are from trusted hosts only:
   - `coda.io`
   - `codahosted.io`
   - `storage.googleapis.com`
4. **Limit bounds**: User-provided limits are capped at 1000 to prevent excessive resource usage
5. **Install script security**:
   - Token input is silent (`read -sp`)
   - Config file permissions set to 600 (owner read/write only)
