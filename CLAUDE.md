# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build
cargo build --release

# Run all tests (unit + integration with mocks)
cargo test

# Run E2E tests against live Coda API (requires token)
export $(cat .env | xargs) && cargo test --test e2e_tests -- --ignored

# Run Docker E2E tests (requires Docker + token + image built)
docker build -t coda-mcp:local .
export $(cat .env | xargs) && cargo test --test docker_e2e -- --ignored

# Run a single test
cargo test test_name

# Format and lint
cargo fmt && cargo clippy

# Run with debug logging
RUST_LOG=debug cargo run
```

## Architecture

This is an MCP (Model Context Protocol) server that enables AI assistants to interact with Coda.io documents, tables, and rows via the Coda API.

### Core Components

- **`src/main.rs`** - MCP server using `rmcp` crate. Uses `#[tool_router]` macro to register 18 tools. JSON-RPC over stdio transport.
- **`src/client.rs`** - HTTP client for Coda API. Key quirk: connection pooling disabled (`pool_max_idle_per_host(0)`) to avoid HTTP/2 multiplexing issues with Coda's API.
- **`src/models/`** - Data models for API requests/responses. Each file follows pattern: response structs, list wrappers with pagination, param structs implementing `JsonSchema`.
- **`src/config.rs`** - Configuration from env vars. Custom `Debug` impl redacts API token.
- **`src/error.rs`** - Error types with actionable messages.

### Key Patterns

**Page content retrieval uses async export workflow:**
1. POST `/docs/{docId}/pages/{pageId}/export` with `{"outputFormat": "html"}`
2. Poll GET `.../export/{exportId}` until `status: "complete"` (max 30 attempts, 1s interval)
3. Download content from `downloadLink`

**Write operations return HTTP 202** - changes are queued, not immediate.

**Row filtering** uses Coda formula syntax: `'Status:"Active"'`

### Testing

- **`tests/integration_tests.rs`** - Mock-based tests using `wiremock`
- **`tests/e2e_tests.rs`** - Live API tests, marked `#[ignore]`, require `CODA_API_TOKEN`
- **`tests/docker_e2e.rs`** - Docker E2E tests, marked `#[ignore]`, require Docker + `CODA_API_TOKEN` + `coda-mcp:local` image

### Docker

Multi-stage build: `rust:1.93-slim-bookworm` → `debian:bookworm-slim`. Uses dependency caching (dummy `main.rs` trick — must clean `.fingerprint/` to force recompilation of real source).

CI publishes multi-arch images (amd64 + arm64) to `ghcr.io/nkpar/coda-mcp` on release tags.

### Security Boundaries

- URL validation in `download_raw()` restricts to: `coda.io`, `codahosted.io`, `storage.googleapis.com`
- Limits capped at 1000 to prevent resource exhaustion
- Token redacted from all debug output

## Release Process

```bash
# Bump version in Cargo.toml, then:
cargo check  # updates Cargo.lock
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to X.Y.Z"
git tag vX.Y.Z
git push origin main --tags
```

CI handles: lint → test → security audit → build (3 platforms) → publish to crates.io → GitHub release with binaries.
