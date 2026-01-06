# Script Specification

Scripts are executables that mcpd exposes as MCP tools.

## Discovery

mcpd runs `script --help` to get metadata. The script must exit with code `0`.

### Help Output Format

When mcpd calls `script --help`, it expects:

**stdout**: JSON object with script metadata:
```json
{
  "title": "My Script Title (optional)",
  "description": "What this script does",
  "version": "1.0.0 (optional)",
  "state": false
}
```

**stderr**: JSON object with option definitions (can be empty `{}` if no options):
```json
{
  "option_name": {
    "description": "What this option does",
    "required": true,
    "value_type": "string",
    "default_value": "default",
    "size": {
      "min": 1,
      "max": 100
    }
  }
}
```

**exit code**: Must be `0` (success)

### Complete Example

```bash
#!/bin/bash
if [ "$1" = "--help" ]; then
  # stdout: script metadata
  echo '{"description": "Adds two numbers", "state": false}'
  # stderr: option definitions
  echo '{"a": {"description": "First number", "required": true, "value_type": "integer"}, "b": {"description": "Second number", "required": true, "value_type": "integer"}}' >&2
  exit 0
fi
# ... rest of script
```

## Execution

When a script is called (not with `--help`), options are passed via:

- **stdin**: JSON object `{"option_name": "value"}`
- **Environment**: Each option as `$option_name` (string representation)

Scripts can read from either source. If both are provided, stdin takes precedence for structured data, while environment variables are always available.

### Output

- **stdout**: Response (returned to MCP client as text)
- **stderr**: Logging (can use structured logging prefixes: `INFO`, `ERROR`, `DEBUG`, `WARNING`, `TRACE`)
- **exit code**: `0` = success, non-zero = error

### Structured Logging

Scripts can output structured logs to stderr that mcpd will parse and forward to its logging system:

```
INFO This is an info message
ERROR Something went wrong
DEBUG Debug information
WARNING This is a warning
TRACE Detailed trace information
```

Lines without these prefixes are treated as regular stderr output.

## Options Format

Each option in the stderr JSON must follow this structure:

```json
{
  "option_name": {
    "description": "What this option does",
    "required": true,
    "value_type": "string",
    "default_value": "default",
    "size": {
      "min": 1,
      "max": 100
    }
  }
}
```

### Value Types

- `"string"` - Text value
- `"integer"` - Whole number
- `"float"` - Decimal number
- `"boolean"` - true/false
- `"any"` - Any JSON value
- `{"enum": ["value1", "value2"]}` - One of the listed values (object with "enum" key)

### Size Constraints

The `size` field is optional and applies constraints:
- For `string`: `min`/`max` = character length
- For `integer`/`float`: `min`/`max` = numeric value

### Default Values

- If `required` is `true`, `default_value` is ignored
- If `required` is `false`, `default_value` is required
- Default value type must match `value_type`
- For enum types, default must be one of the enum values

## State

Scripts with `"state": true` in their help output must handle the `--state` flag:

```bash
if [ "$1" = "--state" ]; then
  # Return current state as JSON on stdout
  echo '{"count": 42, "last_updated": "2024-01-01T00:00:00Z"}'
  exit 0
fi
```

When called with `--state`:
- No options are passed (stdin is empty, no environment variables)
- stdout should contain the current state (preferably JSON, but any text is accepted)
- stderr can contain logging
- exit code `0` = success, non-zero = error

Resource URI format: `mcpd://script-path/state`

The script path is relative to the root directory. For example, if your script is at `scripts/utils/counter`, the resource URI is `mcpd://utils/counter/state`.

## Environment Variables

Scripts receive the following environment variables automatically:

- `MCPD_CONFIG_SERVER_HOST` - Server hostname
- `MCPD_CONFIG_SERVER_PORT` - Server port
- `MCPD_CONFIG_SERVER_HTTP_BASE_PATH` - HTTP base path
- `MCPD_CONFIG_SERVER_USERNAME` - Authentication username (if configured)
- `MCPD_CONFIG_SERVER_API_TOKEN` - API token (if configured)
- `MCPD_CONFIG_COMMANDS_ROOT_DIRECTORY` - Root directory path
- `MCPD_CONFIG_SERVER_HTTPS` - `true`/`false` (as string) if HTTPS is enabled
- `MCPD_CONFIG_LOGGING_LEVEL_NAME` - Current logging level
- `MCPD_CONFIGURATION_FILENAME` - Configuration source identifier

Plus any custom configuration passed via `--script-config KEY=VALUE`.

## Exit Codes

| Exit | Meaning |
|------|---------|
| 0 | Success |
| 1 | Internal error |
| 2 | Bad request |
| 3 | Forbidden |
| 4 | Not found |
| 5+ | Error |

mcpd treats non-zero exit codes as errors and returns them to the MCP client.

## Directory Structure

Scripts are discovered recursively from the root directory (up to 5 levels deep). The directory structure maps to MCP tool names:

```
scripts/
├── hello          → tool: "hello"
├── utils/
│   ├── calc       → tool: "utils/calc"
│   └── format     → tool: "utils/format"
└── api/
    └── v1/
        └── fetch  → tool: "api/v1/fetch"
```

The root directory name is stripped from tool paths. Only executable files are discovered.

## Examples

- [Basic](https://github.com/pouriya/mcpd/tree/master/samples/basic) — hello-world, greet, echo
- [Intermediate](https://github.com/pouriya/mcpd/tree/master/samples/intermediate) — calculator, validation, system info
- [Advanced](https://github.com/pouriya/mcpd/tree/master/samples/advanced) — stateful counter, key-value store, task queue
