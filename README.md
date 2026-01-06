# mcpd

MCP daemon — expose simple scripts as MCP tools and resources.

**mcpd** is an MCP (Model Context Protocol) server that turns any executable script into an MCP tool. It discovers scripts in a directory, extracts metadata via `--help`, and exposes them to MCP clients like Claude, Cursor, or any LLM-based agent.

## Features

mcpd is a ~3MB executable that includes:

- **Scripts as Tools**: Any executable becomes an MCP tool automatically
- **Self-describing Scripts**: Scripts define their own options via `--help` output
- **Stateful Resources**: Scripts can expose state via MCP resources
- **HTTP/HTTPS**: JSON-RPC over HTTP transport with TLS support
- **Web Dashboard**: Built-in UI to browse and test tools (enabled by default)
- **Authentication**: Optional username/password, bearer tokens, and CAPTCHA support
- **Cross-platform**: Single binary for Linux, macOS, and Windows

## Quick Start

### Using Docker

```bash
# Create a directory for your scripts
mkdir -p scripts

# Create a date example script with description and options
cat > scripts/date << 'EOF'
#!/bin/sh
if [ "$1" = "--help" ]; then
  # stdout: script metadata
  echo '{"description": "Get the current date and time with optional format", "state": false}'
  # stderr: option definitions
  echo '{"format": {"description": "Date format string (e.g., +%Y-%m-%d)", "required": false, "value_type": "string", "default_value": ""}}' >&2
  exit 0
fi

# Use format from environment variable if provided
if [ -n "$format" ]; then
  date "$format"
else
  date
fi
EOF
chmod +x scripts/date

# Run mcpd with Docker
docker run --rm -it -p 1995:1995 -v "$(pwd)/scripts:/var/lib/mcpd" ghcr.io/pouriya/mcpd:latest
```

Open your browser to `http://localhost:1995` to access the web dashboard and test your tools.

![mcpd web dashboard](https://github.com/user-attachments/assets/360b19b2-cfb8-4631-b083-39b555a718ad)


## Configuration

<details>
<summary><strong>Click to expand all configuration options</strong></summary>

### HTTP Server Options

| Option | Env Variable | Default | Description |
|-------|--------------|---------|-------------|
| `--http-host` | `MCPD_HTTP_HOST` | `127.0.0.1` | HTTP server listen address |
| `--http-port` | `MCPD_HTTP_PORT` | `1995` | HTTP server listen port number |
| `--http-base-path` | `MCPD_HTTP_BASE_PATH` | `/` | HTTP server base path (currently not used) |
| `--http-tls-cert-file` | `MCPD_HTTP_TLS_CERT_FILE` | - | TLS certificate file (enables HTTPS when used with `--http-tls-key-file`) |
| `--http-tls-key-file` | `MCPD_HTTP_TLS_KEY_FILE` | - | TLS private key file (enables HTTPS when used with `--http-tls-cert-file`) |
| `--http-read-timeout-secs` | `MCPD_HTTP_READ_TIMEOUT` | `30` | Read timeout for client connections in seconds |
| `--http-write-timeout-secs` | `MCPD_HTTP_WRITE_TIMEOUT` | `30` | Write timeout for client connections in seconds |

### HTTP Authentication Options

| Option | Env Variable | Default | Description |
|-------|--------------|---------|-------------|
| `--http-auth-username` | `MCPD_HTTP_AUTH_USERNAME` | `` | Authentication username (defaults to `admin` if password is set) |
| `--http-auth-password-file` | `MCPD_HTTP_AUTH_PASSWORD_FILE` | - | File containing SHA512 of password (allows runtime password changes) |
| `--http-auth-password-sha512` | `MCPD_HTTP_AUTH_PASSWORD_SHA512` | - | SHA512 hash of password (static, cannot be changed via API) |
| `--http-auth-captcha` | `MCPD_HTTP_AUTH_CAPTCHA` | `false` | Enable CAPTCHA for authentication |
| `--http-auth-captcha-case-sensitive` | `MCPD_HTTP_AUTH_CAPTCHA_CASE_SENSITIVE` | `false` | Make CAPTCHA case-sensitive |
| `--http-auth-api-token` | `MCPD_HTTP_AUTH_API_TOKEN` | - | Hardcoded bearer token that never expires |
| `--http-auth-token-timeout` | `MCPD_HTTP_AUTH_TOKEN_TIMEOUT` | `604800` | Timeout for dynamically generated tokens in seconds (default: 1 week) |

### Script Options

| Option | Env Variable | Default | Description |
|-------|--------------|---------|-------------|
| `--script-root-directory` | `MCPD_SCRIPT_ROOT_DIRECTORY` | **Required** | Root directory containing executable scripts (must exist) |
| `--script-config` | - | - | Configuration key/value pairs for scripts in `KEY=VALUE` format (JSON values, can be specified multiple times) |

### Web Dashboard Options

| Option | Env Variable | Default | Description |
|-------|--------------|---------|-------------|
| `--www-ui-enable` | `MCPD_WWW_UI_ENABLE` | `true` | Enable/disable the web dashboard |
| `--www-static-directory` | `MCPD_WWW_STATIC_DIRECTORY` | - | Directory to serve custom web files under `/static/*` (can override built-in files) |
| `--www-config` | - | - | Configuration key/value pairs for web dashboard in `KEY=VALUE` format (accessible via `/api/public/configuration`) |

### Logging Options

| Option | Env Variable | Default | Description |
|-------|--------------|---------|-------------|
| `--trace` | - | `false` | Enable trace level logging (shows target and location) |
| `--debug` | - | `false` | Enable debug level logging (shows target) |
| `--quiet` | - | `false` | Disable all logging |

</details>

## MCP Endpoint

```
POST /api/mcp
Content-Type: application/json
```

Supported methods: `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`

The server supports JSON-RPC 2.0 batch requests and notifications.

## Writing Scripts

See [SCRIPT.md](SCRIPT.md) for the complete specification.

### Examples

Learn how to write scripts by exploring the sample implementations:

- **[Basic examples](https://github.com/pouriya/mcpd/tree/master/samples/basic)** — hello-world, greet, echo
- **[Intermediate examples](https://github.com/pouriya/mcpd/tree/master/samples/intermediate)** — calculator, validation, system info
- **[Advanced examples](https://github.com/pouriya/mcpd/tree/master/samples/advanced)** — stateful counter, key-value store, task queue

## Additional HTTP Endpoints

Besides the MCP endpoint, mcpd provides:

- `GET /api/public/captcha` - Get CAPTCHA image (if enabled)
- `GET /api/public/configuration` - Get public configuration
- `GET /api/auth/test` - Test authentication
- `POST /api/auth/token` - Get authentication token
- `POST /api/setPassword` - Change password (requires password file)
- `GET /static/*` - Serve static files (web dashboard)

## License

BSD-3-Clause
