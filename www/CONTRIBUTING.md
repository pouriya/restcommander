# Frontend Contributing

The web dashboard is built with vanilla JavaScript and Bootstrap 5. No build tools required.

## Development Setup

### Using Docker

```bash
# Pull the image
docker pull pouriya/mcpd

# Create directories
mkdir mcpd-frontend mcpd-scripts

# Copy frontend files (or clone the repo)
cp www/* mcpd-frontend/

# Create a test script
cat > mcpd-scripts/test << 'EOF'
#!/usr/bin/env sh
if [ "$1" = "--help" ]; then
  echo '{"description": "Test script"}'
  exit 0
fi
echo '{"ok": true}'
EOF
chmod +x mcpd-scripts/test

# Run with mounted directories
docker run --init -it -p 1995:1995 \
  -v $(pwd)/mcpd-frontend:/mcpd/www \
  -v $(pwd)/mcpd-scripts:/mcpd/scripts \
  pouriya/mcpd
```

Open https://127.0.0.1:1995 and edit files in `mcpd-frontend/`. Refresh to see changes.

### Using Rust

If you have the Rust toolchain:

```bash
make start-dev
```

This builds and runs the server with the `www/` directory mounted.

## Guidelines

**Important**: We use vanilla JavaScript and Bootstrap 5 only. No additional frameworks or build tools.

### Files

| File | Purpose |
|------|---------|
| `index.html` | Main dashboard |
| `index.js` | Dashboard logic |
| `login.html` | Login page |
| `login.js` | Authentication |
| `tools.html` | Tools browser |
| `mcp.js` | MCP client |
| `api.js` | API utilities |
| `utils.js` | Shared utilities |
| `styles.css` | Custom styles |
| `configuration.js` | Config handling |
| `theme.js` | Theme switching |

### Style Guide

- Use Bootstrap 5 classes where possible
- Keep JavaScript simple and readable
- Test on latest Chrome, Firefox, Safari
- Ensure mobile responsiveness

## Submitting Changes

1. Fork and clone the repository
2. Create a branch: `git checkout -b fix/my-fix`
3. Make changes and test locally
4. Copy files back to `www/` in the repo
5. Commit and push
6. Open a pull request
