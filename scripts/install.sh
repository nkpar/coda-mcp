#!/bin/bash
set -e

echo "=== Coda MCP Installer for Claude Desktop ==="
echo ""

# 1. Check for cargo
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is required but not installed."
    echo "Install from: https://rustup.rs"
    exit 1
fi

# 2. Check for jq early (needed for token extraction)
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed."
    echo "Install with: brew install jq (macOS) or apt install jq (Linux)"
    exit 1
fi

# 3. Find Claude Desktop config
if [[ "$OSTYPE" == "darwin"* ]]; then
    CONFIG_DIR="$HOME/Library/Application Support/Claude"
else
    CONFIG_DIR="$HOME/.config/claude"
fi
CONFIG_FILE="$CONFIG_DIR/claude_desktop_config.json"

echo "Config location: $CONFIG_FILE"
echo ""

# 4. Check for existing token
EXISTING_TOKEN=""
if [ -f "$CONFIG_FILE" ]; then
    EXISTING_TOKEN=$(jq -r '.mcpServers.coda.env.CODA_API_TOKEN // empty' "$CONFIG_FILE" 2>/dev/null || true)
fi

# 5. Clean existing installation
BINARY_PATH="$HOME/.cargo/bin/coda-mcp"
if [ -f "$BINARY_PATH" ]; then
    echo "Removing existing installation..."
    cargo uninstall coda-mcp 2>/dev/null || rm -f "$BINARY_PATH"
fi

# 6. Install via cargo
echo "Installing coda-mcp from crates.io..."
cargo install coda-mcp
echo "Installed to: $BINARY_PATH"
echo ""

# 7. Get token (reuse existing or prompt)
if [ -n "$EXISTING_TOKEN" ]; then
    echo "Found existing Coda API token in config."
    read -p "Reuse existing token? [Y/n]: " REUSE_TOKEN
    REUSE_TOKEN=${REUSE_TOKEN:-Y}
    if [[ "$REUSE_TOKEN" =~ ^[Yy] ]]; then
        CODA_TOKEN="$EXISTING_TOKEN"
        echo "Reusing existing token."
    else
        EXISTING_TOKEN=""
    fi
fi

if [ -z "$EXISTING_TOKEN" ] || [[ ! "$REUSE_TOKEN" =~ ^[Yy] ]]; then
    echo ""
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
fi

# 8. Create/update config
mkdir -p "$CONFIG_DIR"

if [ -f "$CONFIG_FILE" ]; then
    echo "Updating existing config..."
    jq --arg bin "$BINARY_PATH" --arg token "$CODA_TOKEN" \
       '.mcpServers.coda = {"command": $bin, "env": {"CODA_API_TOKEN": $token}}' \
       "$CONFIG_FILE" > "$CONFIG_FILE.tmp" && mv "$CONFIG_FILE.tmp" "$CONFIG_FILE"
else
    echo "Creating new config..."
    jq -n --arg bin "$BINARY_PATH" --arg token "$CODA_TOKEN" \
       '{"mcpServers":{"coda":{"command": $bin, "env": {"CODA_API_TOKEN": $token}}}}' > "$CONFIG_FILE"
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
