# mcpd

MCP daemon — expose simple scripts as MCP tools and resources.

**mcpd** is an MCP (Model Context Protocol) server that turns any executable script into an MCP tool. It discovers scripts in a directory, extracts metadata via `--help`, and exposes them to MCP clients like Claude, Cursor, or any LLM-based agent.

## Features

- **Scripts as Tools**: Any executable becomes an MCP tool automatically
- **Self-describing Scripts**: Scripts define their own options via `--help` output
- **Stateful Resources**: Scripts can expose state via MCP resources
- **Multiple Transports**: HTTP/HTTPS server with JSON-RPC over HTTP
- **Web Dashboard**: Built-in UI to browse and test tools
- **Authentication**: Bearer tokens, basic auth, and CAPTCHA support
- **Cross-platform**: Single binary for Linux, macOS, and Windows

## Quick Start

```bash
# Create a scripts directory
mkdir scripts

# Create a simple tool
cat > scripts/greet << 'EOF'
#!/usr/bin/env sh
if [ "$1" = "--help" ]; then
  echo '{"description": "Greet someone", "state": false}'
  echo '{"name": {"description": "Name to greet", "required": true, "value_type": "string"}}' >&2
  exit 0
fi
input=$(cat)
name=$(echo "$input" | grep -o '"name":"[^"]*"' | cut -d'"' -f4)
echo "{\"greeting\": \"Hello, $name!\"}"
EOF
chmod +x scripts/greet

# Start mcpd
mcpd --root-directory scripts
```

MCP clients can now discover and call the `greet` tool.

## Installation

### From Source

```bash
cargo install --path .
```

### Pre-built Binaries

Download from [releases](https://github.com/pouriya/mcpd/releases):

- **Linux**: `mcpd-latest-x86_64-unknown-linux-gnu`
- **macOS**: `mcpd-latest-x86_64-apple-darwin`
- **Windows**: `mcpd-latest-x86_64-pc-windows-msvc.exe`

### Docker

```bash
docker pull pouriya/mcpd
docker run -p 1995:1995 -v ./scripts:/scripts pouriya/mcpd
```

## Configuration

```bash
mcpd --help
```

Key options:

| Option | Env Variable | Default | Description |
|--------|--------------|---------|-------------|
| `--root-directory` | `MCPD_COMMANDS_ROOT_DIRECTORY` | `.` | Scripts directory |
| `--host` | `MCPD_SERVER_HOST` | `127.0.0.1` | Listen address |
| `--port` | `MCPD_SERVER_PORT` | `1995` | Listen port |
| `--enabled` | `MCPD_WWW_ENABLED` | `false` | Enable web dashboard |
| `--debug` | - | `false` | Debug logging |

## MCP Protocol

mcpd implements the [Model Context Protocol](https://modelcontextprotocol.io/) specification (2024-11-05).

### Endpoint

```
POST /api/mcp
Content-Type: application/json
```

### Supported Methods

| Method | Description |
|--------|-------------|
| `initialize` | Initialize MCP session |
| `tools/list` | List available tools |
| `tools/call` | Execute a tool |
| `resources/list` | List available resources |
| `resources/read` | Read a resource |

### Example

```bash
# List tools
curl -X POST http://localhost:1995/api/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'

# Call a tool
curl -X POST http://localhost:1995/api/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "id":2,
    "method":"tools/call",
    "params":{"name":"greet","arguments":{"name":"World"}}
  }'
```

## Writing Scripts

See [SCRIPT.md](SCRIPT.md) for the full script specification.

### Minimal Script

```bash
#!/usr/bin/env sh
if [ "$1" = "--help" ]; then
  echo '{"description": "My tool"}'
  exit 0
fi
echo "Hello from my tool!"
```

### With Options

```bash
#!/usr/bin/env sh
if [ "$1" = "--help" ]; then
  echo '{"description": "Calculator", "state": false}'
  echo '{"a": {"required": true, "value_type": "integer"}, "b": {"required": true, "value_type": "integer"}}' >&2
  exit 0
fi
# Options come via stdin as JSON
input=$(cat)
# Or via environment: $MCPD_OPT_a, $MCPD_OPT_b
echo "{\"sum\": $((MCPD_OPT_a + MCPD_OPT_b))}"
```

### Stateful Script

```bash
#!/usr/bin/env sh
STATE_FILE="/tmp/counter"

if [ "$1" = "--help" ]; then
  echo '{"description": "Counter", "state": true}'
  exit 0
fi

if [ "$1" = "--state" ]; then
  cat "$STATE_FILE" 2>/dev/null || echo '{"count": 0}'
  exit 0
fi

count=$(cat "$STATE_FILE" 2>/dev/null | grep -o '[0-9]*' || echo 0)
new_count=$((count + 1))
echo "{\"count\": $new_count}" > "$STATE_FILE"
echo "{\"count\": $new_count}"
```

## Web Dashboard

Enable with `--enabled`:

```bash
mcpd --root-directory scripts --enabled
```

Open http://localhost:1995 to browse tools and test them interactively.

## Security

- **Authentication**: Use `--username` and `--password-file` for basic auth
- **API Token**: Use `--api-token` for bearer token authentication  
- **HTTPS**: Use `--tls-cert-file` and `--tls-key-file` for TLS
- **CAPTCHA**: Enable with `--captcha`

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

BSD-3-Clause
