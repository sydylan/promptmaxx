# promptmaxx UX Prototype

## Philosophy

Your prompts are sacred. They represent breakthrough moments of thought. This tool exists to ensure you never lose them and to make you sharper over time.

**Core principle: Never break flow.** The UI is a transparent HUD that lives on your screen. You never leave your terminal. You never switch apps.

---

## The HUD

A small, always-visible overlay in the corner of your screen (position configurable).

### Idle State

```
┌──────────────────┐
│ ● promptmaxx     │  ← Green dot = Claude Code detected
│   ready          │  ← Status text
└──────────────────┘
```

- **Green dot**: Claude Code is active, ready to capture
- **Gray dot**: No Claude Code detected
- **Pulsing dot**: Currently capturing/processing

The HUD is translucent. You see your work through it. It's there but not demanding attention.

### Save (Hotkey)

You're in Claude Code. Prompt worked. Press `Cmd+Shift+S` (configurable).

**What happens:**
- HUD briefly flashes/pulses
- Text changes to "✓ saved" for 1 second
- Returns to idle
- Done. No dialogs. No input needed.

```
┌──────────────────┐
│ ✓ saved          │  ← Flashes for 1 second
│   "Refactor..."  │  ← First few words of prompt
└──────────────────┘
```

The prompt is captured with full context (repo, branch, file, timestamp). You never stopped working.

### Search/History (Click HUD)

Click the HUD. It expands into a search panel - still transparent, still floating.

```
┌─────────────────────────────────────────────────────────┐
│  promptmaxx                                    [×]      │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ Search your prompts...                            │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
│  Recent:                                                │
│  ┌───────────────────────────────────────────────────┐ │
│  │ "Refactor this function to use early returns..."  │ │
│  │ promptmaxx • main • 2 min ago                     │ │
│  └───────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────┐ │
│  │ "Write a test that would catch this edge case..." │ │
│  │ api-server • feature-auth • 1 hour ago            │ │
│  └───────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────┐ │
│  │ "Before fixing, trace the data flow from..."      │ │
│  │ api-server • main • yesterday                     │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
│  ──────────────────────────────────────────────────── │
│  47 prompts saved • This repo: 12                      │
└─────────────────────────────────────────────────────────┘
```

**Interactions:**
- Type to search (natural language: "that debugging prompt")
- Click a prompt to copy to clipboard
- Press `Esc` or click outside to collapse back to idle
- Still transparent - your terminal visible underneath

### Search Results

```
┌─────────────────────────────────────────────────────────┐
│  promptmaxx                                    [×]      │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ refactoring with early returns                    │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
│  3 results:                                             │
│  ┌───────────────────────────────────────────────────┐ │
│  │ ★ "Refactor this function to use early returns   │ │
│  │    instead of nested conditionals. Keep the same │ │
│  │    behavior but make it readable in one scan."   │ │
│  │                                                   │ │
│  │ promptmaxx • main • Jan 30                 [copy] │ │
│  └───────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────┐ │
│  │ "Rewrite this with guard clauses at the top..."  │ │
│  │                                                   │ │
│  │ api-server • feature-auth • Jan 12        [copy] │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## Detailed Flow: The Save Moment

**Trigger:** You press the hotkey during or after a Claude Code interaction.

**What gets captured automatically:**

```json
{
  "id": "pm_abc123",
  "prompt": "Refactor this function to use early returns instead of nested conditionals...",
  "timestamp": "2024-01-30T14:23:00Z",
  "context": {
    "repo": "promptmaxx",
    "branch": "main",
    "working_directory": "/Users/you/projects/promptmaxx",
    "recent_files": ["src/index.ts", "src/hud.ts"]
  },
  "embedding": [0.123, -0.456, ...],
  "source": "claude-code"
}
```

**No input required from you.** The context is inferred:
- Git repo and branch from cwd
- Recent files from git status or file watcher
- Timestamp automatic
- Embedding generated for semantic search

---

## Detailed Flow: The Search Moment

**Trigger:** Click the HUD.

**Your mental state:** "I had a good prompt for this..."

**What you do:**
1. Click HUD → expands
2. Type: "debugging async" or "that auth middleware prompt"
3. Results appear instantly (local semantic search)
4. Click result → copied to clipboard
5. Paste into Claude Code, tweak as needed

**The search understands:**
- Natural language ("that prompt about clean code")
- Project context ("show me prompts from api-server")
- Task type ("debugging prompts", "test writing")
- Time ("prompts from last week")

---

## Detailed Flow: Cold Start / Warmup

Click HUD in a new session. It knows you're starting fresh.

```
┌─────────────────────────────────────────────────────────┐
│  promptmaxx                                    [×]      │
│                                                         │
│  Good morning. You're in: api-server                   │
│                                                         │
│  Last session (Fri):                                   │
│  You saved 4 prompts working on auth middleware.       │
│  ┌───────────────────────────────────────────────────┐ │
│  │ "Add refresh token rotation to the auth flow..."  │ │
│  │ Your last save • Friday 5:23pm             [copy] │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
│  Your best prompts in this repo:                       │
│  • "Before writing code, analyze the existing..."      │
│  • "Show me the data flow from request to..."          │
│  • "Write a test that would have caught..."            │
│                                                         │
│  [Search...]                                           │
└─────────────────────────────────────────────────────────┘
```

This is automatic - the HUD detects you're in a repo you've worked on before and surfaces relevant context.

---

## Detailed Flow: Ghost Text (Future v1.0)

While typing in Claude Code, the HUD detects similarity to past prompts.

```
Terminal:                          HUD (corner):
                                   ┌──────────────────────┐
You: Refactor this f|              │ Similar prompt:      │
                                   │ "...use early        │
                                   │ returns instead of   │
                                   │ nested conditionals" │
                                   │                      │
                                   │ [Tab] to insert      │
                                   └──────────────────────┘
```

Press Tab → prompt is inserted/completed. Press any other key → suggestion dismissed.

---

## Data Storage

All local. All yours.

```
~/.promptmaxx/
├── config.json           # settings
├── prompts.db            # sqlite - your sacred data
├── embeddings.db         # vector index for search
├── backups/              # automatic daily backups
│   └── YYYY-MM-DD.db
└── exports/              # on-demand exports
```

**config.json:**
```json
{
  "hotkey": "cmd+shift+s",
  "hud_position": "top-right",
  "hud_opacity": 0.9,
  "auto_backup": true,
  "backup_retention_days": 30,
  "claude_api_key": "sk-...",
  "theme": "dark"
}
```

---

## Technical Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Tauri App                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │   HUD UI    │  │   Hotkey    │  │  Claude Code    │ │
│  │  (WebView)  │  │  Listener   │  │   Detector      │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
│         │                │                   │          │
│         └────────────────┼───────────────────┘          │
│                          ▼                              │
│                  ┌─────────────┐                        │
│                  │   Core      │                        │
│                  │  (Rust)     │                        │
│                  └─────────────┘                        │
│                          │                              │
│         ┌────────────────┼────────────────┐            │
│         ▼                ▼                ▼            │
│  ┌───────────┐   ┌─────────────┐  ┌─────────────┐     │
│  │ SQLite DB │   │ Embeddings  │  │  Backups    │     │
│  │ (prompts) │   │  (vectors)  │  │             │     │
│  └───────────┘   └─────────────┘  └─────────────┘     │
└─────────────────────────────────────────────────────────┘
```

**Why Tauri:**
- Lightweight (not Electron bloat)
- Rust backend (fast, reliable)
- Native transparent windows
- Global hotkey support
- TypeScript frontend

---

## MVP Scope (v0.1)

**Week 1 - Proof of concept:**
- [ ] Tauri app with transparent HUD window
- [ ] Global hotkey registration
- [ ] Manual save: hotkey captures clipboard/selection
- [ ] SQLite storage with auto-context (git repo, branch)
- [ ] Basic list view in expanded HUD

**Week 2 - Search:**
- [ ] Local embeddings (all-MiniLM or similar)
- [ ] Semantic search in expanded view
- [ ] Copy to clipboard on click

**Week 3 - Polish:**
- [ ] Configurable position/opacity
- [ ] Backup system
- [ ] Settings panel

**Defer to v0.2:**
- Claude Code process detection
- Ghost text suggestions
- Claude API insights
- Cross-device sync

---

## Open Questions

1. **Claude Code integration:** How do we capture the actual prompt text?
   - Option A: User selects text, presses hotkey (clipboard-based)
   - Option B: Watch Claude Code's conversation files
   - Option C: Claude Code hook/plugin API (if exists)

2. **Embedding model:** Run locally or call API?
   - Local: all-MiniLM-L6-v2 (fast, ~80MB)
   - API: OpenAI embeddings or Claude (costs money, better quality)

3. **Multi-monitor:** How does HUD behave?
   - Follow active window?
   - Stay on primary monitor?
   - User configures which monitor?
