# Contributing to mcpd

We welcome contributions! Before opening a PR, please create an issue to discuss your proposed changes.

## Development Setup

### Prerequisites

- Rust toolchain (stable)
- GNU make

### Building

```bash
# Development build
make dev

# Release build
make release

# Run tests
make test

# Lint
make lint
```

### Running Locally

```bash
# Start development server
make start-dev
```

This creates a `_build` directory with:
- `bin/mcpd` — The executable
- `etc/mcpd/config.toml` — Configuration
- `scripts/` — Sample scripts

The server starts on `https://127.0.0.1:1995/` with debug logging.

### Project Structure

```
src/
├── main.rs       # Entry point
├── settings.rs   # CLI parsing and configuration
├── http.rs       # HTTP server and routing
├── mcp.rs        # MCP protocol implementation
├── cmd/          # Script discovery and execution
│   ├── mod.rs    # Command types
│   ├── tree.rs   # Script tree building
│   └── runner.rs # Script execution
├── captcha.rs    # CAPTCHA generation
├── utils.rs      # Utilities
└── www/          # Embedded web assets
```

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` to check for common issues
- Follow existing code patterns

## Pull Request Guidelines

1. Create an issue first to discuss the change
2. Fork the repository
3. Create a feature branch: `git checkout -b feature/my-feature`
4. Make your changes
5. Run tests: `make test`
6. Run linter: `make lint`
7. Commit with clear messages
8. Push and open a PR

## Frontend Development

See [www/CONTRIBUTING.md](www/CONTRIBUTING.md) for frontend contribution guidelines.

## License

By contributing, you agree that your contributions will be licensed under BSD-3-Clause.
