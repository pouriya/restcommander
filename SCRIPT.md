# Script Specification

Scripts are executables that mcpd exposes as MCP tools.

## Discovery

mcpd runs `script --help` to get metadata:

| Stream | Content |
|--------|---------|
| stdout | `{"description": "...", "state": false}` |
| stderr | `{"option_name": {"required": true, "value_type": "string"}}` |
| exit | Must be `0` |

## Execution

Options are passed via:
- **stdin**: JSON object `{"option_name": "value"}`
- **Environment**: Each option as `$option_name`

Output:
- **stdout**: Response (returned to MCP client)
- **stderr**: Logging
- **exit code**: `0` = success, non-zero = error

## Options Format

```json
{
  "name": {
    "description": "What this does",
    "required": true,
    "value_type": "string",
    "default_value": "default"
  }
}
```

Types: `string`, `integer`, `float`, `boolean`, `any`, `{"enum": ["a", "b"]}`

## State

Scripts with `"state": true` must handle `--state`:

```bash
./script --state  # Returns current state on stdout
```

Resource URI: `mcpd://script-path/state`

## Exit Codes

| Exit | Meaning |
|------|---------|
| 0 | Success |
| 1 | Internal error |
| 2 | Bad request |
| 3 | Forbidden |
| 4 | Not found |
| 5+ | Error |

## Examples

- [Basic](https://github.com/pouriya/mcpd/tree/master/samples/basic)
- [Intermediate](https://github.com/pouriya/mcpd/tree/master/samples/intermediate)
- [Advanced](https://github.com/pouriya/mcpd/tree/master/samples/advanced)
