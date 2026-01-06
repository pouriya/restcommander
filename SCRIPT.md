# Script Specification

Scripts are executable files that mcpd discovers and exposes as MCP tools. Any script in any language (shell, Python, Perl, etc.) can become an MCP tool by following this specification.

## Overview

mcpd translates between MCP protocol and script execution:

```
MCP Client  ←→  mcpd  ←→  Script
   │                        │
   │ tools/call             │ stdin (JSON)
   │ {"name":"X",           │ env vars (MCPD_OPT_*)
   │  "arguments":{...}}    │
   │                        │
   │ Response               │ stdout (response)
   │ {"content":[...]}      │ stderr (logging)
   │                        │ exit code
```

## Script Discovery

When mcpd starts, it scans the `--root-directory` for executables. For each script, it runs `script --help` to extract metadata.

### `--help` Output Format

| Stream | Content | Format |
|--------|---------|--------|
| **stdout** | Metadata | JSON object |
| **stderr** | Options definition | JSON object (or empty) |
| **exit code** | Must be `0` | Non-zero = discovery failure |

**stdout metadata:**

```json
{
  "title": "Optional title",
  "description": "What this tool does",
  "version": "1.0.0",
  "state": false
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `title` | No | Display name (defaults to filename) |
| `description` | No | Tool description for LLM |
| `version` | No | Version string |
| `state` | No | `true` if script supports `--state` |

**stderr options:**

```json
{
  "option_name": {
    "description": "What this option does",
    "required": true,
    "value_type": "string",
    "default_value": "default",
    "size": {"min": 0, "max": 100}
  }
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `description` | No | Option description |
| `required` | Yes | `true` or `false` |
| `value_type` | No | Type constraint (see below) |
| `default_value` | If not required | Default value |
| `size` | No | Size constraints |

**value_type options:**
- `"string"` — Text value
- `"integer"` — Whole number
- `"float"` — Decimal number
- `"boolean"` — `true` or `false`
- `"any"` — No type validation
- `{"enum": ["a", "b", "c"]}` — One of listed values

## Script Execution

When an MCP client calls a tool:

1. mcpd validates input against the script's options
2. Options are passed via stdin (JSON) and environment variables
3. Script runs and produces output
4. mcpd wraps output in MCP response format

### Input

**stdin:** JSON object with validated options

```json
{"name": "John", "count": 5}
```

**Environment variables:** Each option as `MCPD_OPT_<name>`

```bash
MCPD_OPT_name="John"
MCPD_OPT_count="5"
```

### Output

| Stream | Purpose |
|--------|---------|
| **stdout** | Response content (returned to MCP client) |
| **stderr** | Logging (parsed for log levels) |
| **exit code** | Success/failure indicator |

**Exit codes:**

| Exit | Meaning |
|------|---------|
| 0 | Success |
| 1 | Internal error |
| 2 | Bad request (invalid input) |
| 3 | Forbidden |
| 4 | Not found |
| 5 | Service unavailable |
| 6 | Not acceptable |
| 7 | Not implemented |
| 8 | Conflict |
| 9 | Timeout |

**stderr logging prefixes:**

```bash
echo "INFO Starting process"    >&2
echo "DEBUG Variable x=5"       >&2
echo "WARNING Low memory"       >&2
echo "ERROR Failed to connect"  >&2
echo "TRACE Entering function"  >&2
```

## Stateful Scripts

Scripts with `"state": true` must support `--state`:

```bash
./my-script --state
```

**Output:**
- **stdout**: Current state (JSON or string)
- **stderr**: Log messages
- **exit code**: 0 for success

MCP clients can read state via the `resources/read` method with URI:

```
mcpd://script-path/state
```

## Examples

### Minimal Tool

```bash
#!/usr/bin/env sh

if [ "$1" = "--help" ]; then
  echo '{"description": "Says hello"}'
  exit 0
fi

echo '{"message": "Hello, World!"}'
```

### Tool with Options

```bash
#!/usr/bin/env sh

if [ "$1" = "--help" ]; then
  echo '{"description": "Greet someone by name"}'
  echo '{"name": {"description": "Name to greet", "required": true, "value_type": "string"}}' >&2
  exit 0
fi

# Read from environment variable
echo "{\"greeting\": \"Hello, ${MCPD_OPT_name}!\"}"
```

### Tool with Multiple Options

```bash
#!/usr/bin/env sh

if [ "$1" = "--help" ]; then
  echo '{"description": "Calculator", "version": "1.0"}'
  cat >&2 << 'EOF'
{
  "operation": {
    "description": "Math operation",
    "required": true,
    "value_type": {"enum": ["add", "subtract", "multiply", "divide"]}
  },
  "a": {"required": true, "value_type": "float"},
  "b": {"required": true, "value_type": "float"}
}
EOF
  exit 0
fi

a="$MCPD_OPT_a"
b="$MCPD_OPT_b"
op="$MCPD_OPT_operation"

case "$op" in
  add)      result=$(echo "$a + $b" | bc -l) ;;
  subtract) result=$(echo "$a - $b" | bc -l) ;;
  multiply) result=$(echo "$a * $b" | bc -l) ;;
  divide)   result=$(echo "$a / $b" | bc -l) ;;
esac

echo "{\"result\": $result}"
```

### Stateful Counter

```bash
#!/usr/bin/env sh

STATE_FILE="/tmp/mcpd-counter.json"

if [ "$1" = "--help" ]; then
  echo '{"description": "Persistent counter", "state": true}'
  echo '{"increment": {"description": "Amount to add", "required": false, "value_type": "integer", "default_value": 1}}' >&2
  exit 0
fi

if [ "$1" = "--state" ]; then
  [ -f "$STATE_FILE" ] && cat "$STATE_FILE" || echo '{"count": 0}'
  exit 0
fi

# Read current count
count=$(cat "$STATE_FILE" 2>/dev/null | grep -o '"count":[0-9]*' | grep -o '[0-9]*' || echo 0)
increment="${MCPD_OPT_increment:-1}"
new_count=$((count + increment))

# Save and return
echo "{\"count\": $new_count}" > "$STATE_FILE"
echo "{\"count\": $new_count, \"incremented_by\": $increment}"
```

### Python Tool

```python
#!/usr/bin/env python3
import sys
import json
import os

if len(sys.argv) > 1 and sys.argv[1] == "--help":
    print(json.dumps({
        "description": "Process data with Python",
        "version": "1.0.0"
    }))
    print(json.dumps({
        "text": {
            "description": "Text to process",
            "required": True,
            "value_type": "string"
        },
        "uppercase": {
            "description": "Convert to uppercase",
            "required": False,
            "value_type": "boolean",
            "default_value": False
        }
    }), file=sys.stderr)
    sys.exit(0)

# Read options from environment
text = os.environ.get("MCPD_OPT_text", "")
uppercase = os.environ.get("MCPD_OPT_uppercase", "false").lower() == "true"

if uppercase:
    text = text.upper()

print(json.dumps({"processed": text, "length": len(text)}))
```

### Error Handling

```bash
#!/usr/bin/env sh

if [ "$1" = "--help" ]; then
  echo '{"description": "Read a file"}'
  echo '{"path": {"description": "File path", "required": true, "value_type": "string"}}' >&2
  exit 0
fi

path="$MCPD_OPT_path"

if [ ! -f "$path" ]; then
  echo "ERROR File not found: $path" >&2
  echo '{"error": "File not found"}'
  exit 4  # 404 Not Found
fi

if [ ! -r "$path" ]; then
  echo "ERROR Permission denied: $path" >&2
  echo '{"error": "Permission denied"}'
  exit 3  # 403 Forbidden
fi

echo "INFO Reading file: $path" >&2
cat "$path"
```

## MCP Translation

mcpd translates script metadata to MCP format:

**Script options → JSON Schema:**

```bash
# Script stderr on --help:
'{"count": {"required": true, "value_type": "integer", "size": {"min": 1, "max": 100}}}'
```

```json
// MCP inputSchema:
{
  "type": "object",
  "properties": {
    "count": {
      "type": "integer",
      "minimum": 1,
      "maximum": 100
    }
  },
  "required": ["count"]
}
```

**Script stdout → MCP content:**

```bash
# Script stdout:
echo '{"result": 42}'
```

```json
// MCP tools/call response:
{
  "content": [{
    "type": "text",
    "text": "{\"result\": 42}"
  }],
  "isError": false
}
```

## Directory Structure

Scripts can be organized in directories:

```
scripts/
├── greet           → tool: "greet"
├── math/
│   ├── add         → tool: "math/add"
│   └── multiply    → tool: "math/multiply"
└── system/
    └── info        → tool: "system/info"
```

Tool names are paths relative to the root directory.
