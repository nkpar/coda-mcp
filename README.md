# coda-mcp

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)

MCP (Model Context Protocol) server for [Coda.io](https://coda.io) API integration. Enables AI assistants to read and write Coda documents, tables, and data.

## Features

- Full CRUD operations on Coda tables
- Document and page content retrieval
- Formula and control access
- Async export workflow for canvas pages
- Rate limit handling

## Installation

### From Source

```bash
git clone https://github.com/parity-asia/coda-mcp.git
cd coda-mcp
cargo build --release
```

Binary will be at `./target/release/coda-mcp`

### Pre-built Binaries

Check the [Releases](https://github.com/parity-asia/coda-mcp/releases) page.

## Configuration

### 1. Get API Token

Get your Coda API token from [coda.io/account](https://coda.io/account) → API settings.

### 2. Configure MCP Client

**Claude Desktop** (`~/.config/claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "coda": {
      "command": "/path/to/coda-mcp",
      "env": {
        "CODA_API_TOKEN": "your_token_here"
      }
    }
  }
}
```

**Claude Code** (`.mcp.json` in project root):

```json
{
  "mcpServers": {
    "coda": {
      "command": "/path/to/coda-mcp",
      "env": {
        "CODA_API_TOKEN": "your_token_here"
      }
    }
  }
}
```

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `CODA_API_TOKEN` | Yes | Your Coda API token |
| `CODA_BASE_URL` | No | API base URL (default: `https://coda.io/apis/v1`) |
| `RUST_LOG` | No | Log level: `error`, `warn`, `info`, `debug`, `trace` |

## Tools

| Tool | Description |
|------|-------------|
| `list_docs` | List available documents |
| `get_doc` | Get document details |
| `search_docs` | Search documents by name |
| `list_pages` | List pages in a document |
| `get_page` | Get page content (HTML) |
| `list_tables` | List tables in a document |
| `get_table` | Get table details |
| `list_columns` | List columns in a table |
| `get_rows` | Get rows with optional filtering |
| `get_row` | Get a specific row |
| `add_row` | Add a new row |
| `update_row` | Update an existing row |
| `delete_row` | Delete a row |
| `list_formulas` | List named formulas |
| `get_formula` | Get formula value |
| `list_controls` | List controls (buttons, sliders) |

## Usage Examples

```
# List all documents
list_docs

# Get rows from a table
get_rows doc_id="AbCdEfGh" table_id="grid-xyz" limit=50

# Filter rows
get_rows doc_id="AbCdEfGh" table_id="grid-xyz" query="Status:Active"

# Add a row
add_row doc_id="AbCdEfGh" table_id="grid-xyz" cells={"Name": "John", "Email": "john@example.com"}

# Get page content
get_page doc_id="AbCdEfGh" page_id="canvas-xyz"
```

## Rate Limits

Coda API has rate limits:
- **Reading**: 100 requests per 6 seconds
- **Writing**: 10 requests per 6 seconds

Write operations return HTTP 202 (queued). Changes may take a few seconds to appear.

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Lint
cargo clippy
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## License

MIT License. See [LICENSE](LICENSE) for details.
