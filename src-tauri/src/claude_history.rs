use serde::Deserialize;
use std::path::PathBuf;

/// Claude Code history entry (from ~/.claude/history.jsonl)
#[derive(Debug, Deserialize)]
struct ClaudeHistoryEntry {
    display: String,
    #[allow(dead_code)]
    timestamp: u64,
    #[allow(dead_code)]
    project: Option<String>,
}

/// Get the path to Claude's history file
fn get_history_path() -> Option<PathBuf> {
    let mut path = dirs::home_dir()?;
    path.push(".claude");
    path.push("history.jsonl");
    Some(path)
}

/// Read the last prompt from Claude Code's history
pub fn read_last_claude_prompt() -> Option<String> {
    let history_path = get_history_path()?;
    let content = std::fs::read_to_string(&history_path).ok()?;
    let last_line = content.lines().last()?;
    let entry: ClaudeHistoryEntry = serde_json::from_str(last_line).ok()?;

    let text = entry.display.trim();
    if text.starts_with('/') || text.is_empty() {
        return None;
    }

    Some(text.to_string())
}
