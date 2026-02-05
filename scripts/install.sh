#!/bin/bash
set -e

echo "=== Coda MCP Installer for Claude Desktop ==="
echo ""

# 1. Install via cargo
echo "Installing coda-mcp from crates.io..."
cargo install coda-mcp
echo "Install complete."
echo ""

# 2. Find Claude Desktop config
if [[ "$OSTYPE" == "darwin"* ]]; then
    CONFIG_DIR="$HOME/Library/Application Support/Claude"
else
    CONFIG_DIR="$HOME/.config/claude"
fi
CONFIG_FILE="$CONFIG_DIR/claude_desktop_config.json"

echo "Config location: $CONFIG_FILE"
echo ""

# 3. Prompt for token (silent input)
echo "To get your Coda API token:"
echo "  1. Go to https://coda.io/account"
echo "  2. Scroll to 'API settings'"
echo "  3. Click 'Generate API token'"
echo "  4. Enable write access if you need to modify data"
echo ""
read -sp "Enter your Coda API token: " CODA_TOKEN
echo ""

if [ -z "$CODA_TOKEN" ]; then
    echo "Error: Token cannot be empty"
    exit 1
fi

# 4. Check for jq
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed."
    echo "Install with: brew install jq (macOS) or apt install jq (Linux)"
    exit 1
fi

# 5. Create/update config
mkdir -p "$CONFIG_DIR"

if [ -f "$CONFIG_FILE" ]; then
    echo "Updating existing config..."
    jq --arg token "$CODA_TOKEN" \
       '.mcpServers.coda = {"command": "coda-mcp", "env": {"CODA_API_TOKEN": $token}}' \
       "$CONFIG_FILE" > "$CONFIG_FILE.tmp" && mv "$CONFIG_FILE.tmp" "$CONFIG_FILE"
else
    echo "Creating new config..."
    echo '{"mcpServers":{"coda":{"command":"coda-mcp","env":{"CODA_API_TOKEN":"'"$CODA_TOKEN"'"}}}}' | jq . > "$CONFIG_FILE"
fi

# Secure the config file (contains sensitive token)
chmod 600 "$CONFIG_FILE"

echo ""
echo "coda-mcp installed successfully!"
echo ""
echo "Next steps:"
echo "  1. Restart Claude Desktop"
echo "  2. Coda tools will be available automatically"
echo ""
