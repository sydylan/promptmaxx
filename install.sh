#!/bin/bash
set -e

echo "=== Installing pmaxx ==="

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="$HOME/.cargo/env:$PATH"
fi

# Check for Node
if ! command -v npm &> /dev/null; then
    echo "Error: npm required. Install Node.js first: https://nodejs.org"
    exit 1
fi

# Clone or update
if [ -d "$HOME/.pmaxx-src" ]; then
    cd "$HOME/.pmaxx-src"
    git pull
else
    git clone --depth 1 https://github.com/sydylan/promptmaxx.git "$HOME/.pmaxx-src"
    cd "$HOME/.pmaxx-src"
fi

# Install npm deps
npm install

# Build CLI
echo "Building CLI..."
PATH="$HOME/.cargo/bin:$PATH" cargo build -p pmaxx --release

# Build app
echo "Building app..."
PATH="$HOME/.cargo/bin:$PATH" npm run tauri build

# Install CLI
mkdir -p "$HOME/bin"
cp target/release/pmaxx "$HOME/bin/"

# Install app
cp -r target/release/bundle/macos/promptmaxx.app /Applications/

# Add to PATH
if ! grep -q 'HOME/bin' "$HOME/.zshrc" 2>/dev/null; then
    echo 'export PATH="$HOME/bin:$PATH"' >> "$HOME/.zshrc"
fi

# Start app
open /Applications/promptmaxx.app

echo ""
echo "=== Done! ==="
echo ""
echo "Restart your terminal, then:"
echo "  1. Use Claude normally"
echo "  2. Good prompt? Cmd+Shift+P to save"
echo "  3. Say 'pmaxx <vague prompt>' for a better version"
