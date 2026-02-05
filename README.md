# coda-mcp

[![MCP Registry](https://img.shields.io/badge/MCP-Registry-blue)](https://registry.modelcontextprotocol.io/)
[![Crates.io](https://img.shields.io/crates/v/coda-mcp.svg)](https://crates.io/crates/coda-mcp)
[![CI](https://github.com/nkpar/coda-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/nkpar/coda-mcp/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/nkpar/coda-mcp/graph/badge.svg)](https://codecov.io/gh/nkpar/coda-mcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.93-orange.svg)](https://www.rust-lang.org)

MCP (Model Context Protocol) server for [Coda.io](https://coda.io) API. Enables AI assistants to read and write Coda documents, tables, and rows.

## Features

- Full CRUD operations on Coda tables
- Document and page content retrieval
- Formula and control access
- Async export workflow for canvas pages
- Rate limit handling

## Quick Start

### 1. Install

```bash
cargo install coda-mcp
```

### 2. Get API Token

Get your token from [coda.io/account](https://coda.io/account) → API settings.

### 3. Configure Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `~/.config/claude/claude_desktop_config.json` (Linux):

```json
{
  "mcpServers": {
    "coda": {
      "command": "/Users/YOUR_USERNAME/.cargo/bin/coda-mcp",
      "env": {
        "CODA_API_TOKEN": "your_token_here"
      }
    }
  }
}
```

### 4. Restart Claude Desktop

The Coda tools will now be available.

## Installation

### From crates.io (Recommended)

```bash
cargo install coda-mcp
```

### From Source

```bash
git clone https://github.com/nkpar/coda-mcp.git
cd coda-mcp
cargo build --release
```

Binary will be at `./target/release/coda-mcp`

### Pre-built Binaries

Check the [Releases](https://github.com/nkpar/coda-mcp/releases) page.

### Docker

```bash
docker pull ghcr.io/nkpar/coda-mcp:latest
```

Claude Desktop config for Docker:

```json
{
  "mcpServers": {
    "coda": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "-e", "CODA_API_TOKEN", "ghcr.io/nkpar/coda-mcp:latest"],
      "env": {
        "CODA_API_TOKEN": "your_token_here"
      }
    }
  }
}
```

### One-Click Install Script

For Claude Desktop with automatic configuration:

```bash
git clone https://github.com/nkpar/coda-mcp.git
cd coda-mcp
./scripts/install.sh
```

## Configuration

### 1. Get API Token

Get your Coda API token from [coda.io/account](https://coda.io/account) → API settings.

**Important:** For write operations (`create_doc`, `delete_doc`, `add_row`, `update_row`, `delete_row`), ensure your token has write permissions enabled. Read-only tokens will return 403 Forbidden for these operations.

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

**Claude Code** (CLI):

```bash
claude mcp add coda -e CODA_API_TOKEN=your_token_here -- $HOME/.cargo/bin/coda-mcp
```

Or via `.mcp.json` in project root:

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
| `create_doc` | Create a new document (optional: folder, template, timezone) |
| `delete_doc` | Delete a document (permanent) |
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

# Create a new document
create_doc title="My New Doc"

# Create from template in specific folder
create_doc title="Project Plan" folder_id="fl-abc" source_doc="template-xyz"

# Delete a document (permanent!)
delete_doc doc_id="AbCdEfGh"

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

## Security

- API tokens are redacted from all log output
- Download URLs validated against trusted hosts only (coda.io, codahosted.io, storage.googleapis.com)
- Request limits capped at 1000 to prevent resource exhaustion
- Install script uses silent input for tokens and sets restrictive file permissions (600)

## Development

```bash
# Build
cargo build --release

# Run unit & integration tests
cargo test

# Run E2E tests (requires write-enabled API token)
export $(cat .env | xargs) && cargo test --test e2e_tests -- --ignored

# Run with debug logging
RUST_LOG=debug cargo run

# Format & lint
cargo fmt && cargo clippy
```

See [DEVELOPER.md](DEVELOPER.md) for API details and [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## License

MIT License. See [LICENSE](LICENSE) for details.
