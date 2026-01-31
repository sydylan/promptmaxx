#!/bin/bash
set -e

REPO="sydylan/promptmaxx"
LATEST=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
    echo "Error: Could not fetch latest release"
    exit 1
fi

echo "Installing pmaxx $LATEST..."

# Download CLI
TMP=$(mktemp -d)
curl -sL "https://github.com/$REPO/releases/download/$LATEST/pmaxx-mac.zip" -o "$TMP/pmaxx.zip"
unzip -q "$TMP/pmaxx.zip" -d "$TMP"
mkdir -p "$HOME/bin"
mv "$TMP/pmaxx" "$HOME/bin/"
chmod +x "$HOME/bin/pmaxx"

# Download App
curl -sL "https://github.com/$REPO/releases/download/$LATEST/promptmaxx-app-mac.zip" -o "$TMP/app.zip"
unzip -q "$TMP/app.zip" -d "$TMP"
rm -rf /Applications/promptmaxx.app 2>/dev/null || true
mv "$TMP/promptmaxx.app" /Applications/

# Cleanup
rm -rf "$TMP"

# Add to PATH
if ! grep -q 'HOME/bin' "$HOME/.zshrc" 2>/dev/null; then
    echo 'export PATH="$HOME/bin:$PATH"' >> "$HOME/.zshrc"
fi

# Start app
open /Applications/promptmaxx.app

echo ""
echo "Done!"
echo ""
echo "Restart terminal, then:"
echo "  • Good prompt? Cmd+Shift+P"
echo "  • Better prompt? pmaxx <your prompt>"
