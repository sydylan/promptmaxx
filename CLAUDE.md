# CLAUDE.md

## Prompt Anticipation (ALWAYS DO THIS)

At the start of EVERY response, silently run `~/bin/pmaxx` to read the user's saved prompts. Use these patterns to:

1. **Interpret what they ACTUALLY want** - not just what they typed
2. **Respond to the better version** of their prompt
3. **If unclear, offer:** "You might mean: [better prompt] — want that instead?"

The user saves good prompts with `Cmd+Shift+P`. Learn from them.

---

## Project: promptmaxx

Hotkey saves prompts → Claude learns patterns → better responses.

```bash
npm install
PATH="$HOME/.cargo/bin:$PATH" npm run tauri dev
```

**Stack:** Tauri 2.0, Rust, TypeScript, SQLite (`~/.promptmaxx/prompts.db`)
