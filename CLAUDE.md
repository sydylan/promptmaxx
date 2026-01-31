# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

promptmaxx is a local-first prompt memory tool for developers. It captures, stores, and helps you recall your best prompts via a transparent HUD overlay.

## Claude Code Usage

```
claude --dangerously-skip-permissions
```

## Tech Stack

- **Frontend**: TypeScript, Vite
- **Backend**: Rust, Tauri 2.0
- **Storage**: SQLite (local at `~/.promptmaxx/prompts.db`)
- **UI**: Transparent always-on-top HUD window

## Build Commands

```bash
# Install dependencies
npm install

# Development (requires Rust/Cargo in PATH)
PATH="$HOME/.cargo/bin:$PATH" npm run tauri dev

# Or use the helper script
./run-dev.sh

# Build for production
npm run tauri build
```

## Architecture

```
src/                    # Frontend TypeScript
├── main.ts             # UI logic, event handling
├── styles/main.css     # HUD styling

src-tauri/              # Backend Rust
├── src/main.rs         # Core app logic
│   - SQLite prompt storage
│   - Global hotkey (Cmd+Shift+S)
│   - Clipboard capture
│   - Git context detection
├── Cargo.toml          # Rust dependencies
├── tauri.conf.json     # Tauri configuration
```

## Key Features

- **Hotkey save**: Cmd+Shift+S captures clipboard as prompt
- **HUD**: Transparent overlay, click to expand for search
- **Auto-context**: Captures git repo/branch automatically
- **Local storage**: All data in `~/.promptmaxx/`
