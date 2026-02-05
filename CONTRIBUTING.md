# Contributing

Contributions are welcome!

## Development Setup

1. Clone the repository
2. Copy `.env.example` to `.env` and add your Coda API token
3. Build: `cargo build`
4. Run tests: `cargo test`

## Running Tests

```bash
# Unit and integration tests (no API token needed)
cargo test

# E2E tests (requires CODA_API_TOKEN)
export CODA_API_TOKEN=your_token
cargo test --test e2e_tests -- --ignored --test-threads=1
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix any warnings
- Add tests for new functionality

## Pull Requests

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## Reporting Issues

Please include:
- Rust version (`rustc --version`)
- Steps to reproduce
- Expected vs actual behaviour
