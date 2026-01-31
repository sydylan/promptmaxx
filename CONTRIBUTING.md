# Contributing to promptmaxx

## Architecture

```
promptmaxx/
├── crates/
│   ├── promptmaxx/     # Library - the core API
│   └── pmaxx/          # CLI - thin wrapper around library
├── src-tauri/          # Desktop app - hotkey capture + UI
└── src/                # Frontend - TypeScript/HTML
```

### The Library (`crates/promptmaxx/`)

This is the foundation. Everything else is a thin consumer.

```rust
use promptmaxx::{save, list, search, delete, count, exists, Prompt};

// Save a prompt
let prompt = save("my prompt text")?;

// Save with git context
let prompt = save_with_context("text", Some("repo".into()), Some("main".into()))?;

// List all prompts (newest first)
let prompts: Vec<Prompt> = list()?;

// Search by text
let results = search("keyword")?;

// Delete by ID
delete("uuid-here")?;

// Count saved prompts
let n: i32 = count()?;

// Check if exists
if exists("exact text")? {
    println!("Already saved");
}
```

**Data storage:** `~/.promptmaxx/prompts.db` (SQLite)

**Prompt struct:**
```rust
pub struct Prompt {
    pub id: String,        // UUID
    pub text: String,      // The prompt text
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub timestamp: String, // RFC3339
}
```

### The CLI (`crates/pmaxx/`)

60 lines. Just calls the library.

```bash
pmaxx           # List all prompts
pmaxx list      # Same
pmaxx last      # Show most recent
pmaxx count     # Show count
pmaxx <text>    # Suggest better prompt using Claude CLI
```

### The App (`src-tauri/`)

Tauri desktop app that:
- Registers global hotkey (Cmd+Shift+P)
- Reads last prompt from Claude Code history (`~/.claude/history.jsonl`)
- Saves it using the library
- Shows a small HUD notification

App-specific modules (not in library):
- `git.rs` - Gets current repo/branch for context
- `claude_history.rs` - Reads Claude Code's history file

## Building on Top

### Option 1: Use the library directly

Add to your `Cargo.toml`:
```toml
[dependencies]
promptmaxx = { git = "https://github.com/sydylan/promptmaxx" }
```

Then:
```rust
use promptmaxx::{list, save};

fn main() {
    // Read user's saved prompts
    for prompt in list().unwrap() {
        println!("{}", prompt.text);
    }
}
```

### Option 2: Use the CLI

Shell out to `pmaxx`:
```bash
# Get all prompts
prompts=$(pmaxx)

# Get count
count=$(pmaxx count)
```

### Option 3: Read the database directly

SQLite at `~/.promptmaxx/prompts.db`:
```sql
SELECT text, timestamp FROM prompts ORDER BY timestamp DESC;
```

## Extension Ideas

- **New capture methods**: VS Code extension, browser extension, Raycast plugin
- **New consumers**: Obsidian plugin, Alfred workflow, API server
- **Analysis**: Prompt categorization, similarity search, usage stats

## Development

```bash
# Install deps
npm install

# Run in dev mode
PATH="$HOME/.cargo/bin:$PATH" npm run tauri dev

# Build release
npm run tauri build

# Run tests
cargo test

# Check formatting
cargo fmt --check
```

## Code Style

- Keep it simple. No abstractions for hypothetical futures.
- Library functions handle their own DB connection.
- Errors bubble up with `?`, consumers decide how to handle.
- No traits unless you have 2+ implementations today.
