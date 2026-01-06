# mcpd

MCP daemon — expose simple scripts as MCP tools and resources.

**mcpd** is an MCP (Model Context Protocol) server that turns any executable script into an MCP tool. It discovers scripts in a directory, extracts metadata via `--help`, and exposes them to MCP clients like Claude, Cursor, or any LLM-based agent.

## Features

- **Scripts as Tools**: Any executable becomes an MCP tool automatically
- **Self-describing Scripts**: Scripts define their own options via `--help` output
- **Stateful Resources**: Scripts can expose state via MCP resources
- **HTTP/HTTPS**: JSON-RPC over HTTP transport
- **Web Dashboard**: Built-in UI to browse and test tools
- **Cross-platform**: Single binary for Linux, macOS, and Windows

## Quick Start

```bash
mkdir scripts
mcpd --root-directory scripts
```

MCP clients can now connect to `http://localhost:1995/api/mcp`.

## Installation

```bash
cargo install --path .
```

Or download from [releases](https://github.com/pouriya/mcpd/releases).

## Configuration

```bash
mcpd --help
```

| Option | Env Variable | Default | Description |
|--------|--------------|---------|-------------|
| `--root-directory` | `MCPD_COMMANDS_ROOT_DIRECTORY` | `.` | Scripts directory |
| `--host` | `MCPD_SERVER_HOST` | `127.0.0.1` | Listen address |
| `--port` | `MCPD_SERVER_PORT` | `1995` | Listen port |
| `--enabled` | `MCPD_WWW_ENABLED` | `false` | Enable web dashboard |

## MCP Endpoint

```
POST /api/mcp
Content-Type: application/json
```

Supported methods: `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`

## Writing Scripts

See [SCRIPT.md](SCRIPT.md) for the specification.

### Examples

- [Basic examples](https://github.com/pouriya/mcpd/tree/master/samples/basic) — hello-world, greet, echo
- [Intermediate examples](https://github.com/pouriya/mcpd/tree/master/samples/intermediate) — calculator, validation, system info
- [Advanced examples](https://github.com/pouriya/mcpd/tree/master/samples/advanced) — stateful counter, key-value store, task queue

## License

BSD-3-Clause
