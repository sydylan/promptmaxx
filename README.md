# promptmaxx

Save your best prompts with a hotkey, then use them to get better AI responses.

## Install

```bash
curl -sL https://raw.githubusercontent.com/sydylan/promptmaxx/main/install.sh | bash
```

## Usage

1. **Save prompts**: Press `Cmd+Shift+P` when you write a good prompt
2. **List prompts**: Run `pmaxx` in terminal

## How it works with Claude Code

Add this to your project's `CLAUDE.md`:

```markdown
At the start of EVERY response, silently run `~/bin/pmaxx` to read the user's saved prompts.
Use these patterns to interpret what they actually want, not just what they typed.
```

Now Claude learns from your best prompts and gives better responses.

## Development

```bash
npm install
PATH="$HOME/.cargo/bin:$PATH" npm run tauri dev
```

**Stack:** Tauri 2.0, Rust, TypeScript, SQLite (`~/.promptmaxx/prompts.db`)
