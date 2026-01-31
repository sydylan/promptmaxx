use serde::Deserialize;
use std::path::PathBuf;

use crate::error::{CoreError, Result};

/// Claude Code history entry (from ~/.claude/history.jsonl)
#[derive(Debug, Deserialize)]
pub struct ClaudeHistoryEntry {
    pub display: String,
    #[allow(dead_code)]
    pub timestamp: u64,
    #[allow(dead_code)]
    pub project: Option<String>,
}

/// Get the path to Claude's history file
pub fn get_history_path() -> Result<PathBuf> {
    let mut path = dirs::home_dir().ok_or(CoreError::HomeDirNotFound)?;
    path.push(".claude");
    path.push("history.jsonl");
    Ok(path)
}

/// Read the last prompt from Claude Code's history
pub fn read_last_claude_prompt() -> Option<String> {
    let history_path = get_history_path().ok()?;

    // Read the file and get the last line
    let content = std::fs::read_to_string(&history_path).ok()?;
    let last_line = content.lines().last()?;

    // Parse the JSON
    let entry: ClaudeHistoryEntry = serde_json::from_str(last_line).ok()?;

    // Skip if it looks like a command (starts with /)
    let text = entry.display.trim();
    if text.starts_with('/') || text.is_empty() {
        return None;
    }

    Some(text.to_string())
}

/// Read the last N prompts from Claude Code's history
pub fn read_recent_prompts(count: usize) -> Vec<String> {
    let history_path = match get_history_path() {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let content = match std::fs::read_to_string(&history_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    content
        .lines()
        .rev()
        .filter_map(|line| {
            let entry: ClaudeHistoryEntry = serde_json::from_str(line).ok()?;
            let text = entry.display.trim();
            if text.starts_with('/') || text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        })
        .take(count)
        .collect()
}

/// Get prompts from history filtered by project
pub fn read_prompts_for_project(project: &str, count: usize) -> Vec<String> {
    let history_path = match get_history_path() {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let content = match std::fs::read_to_string(&history_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    content
        .lines()
        .rev()
        .filter_map(|line| {
            let entry: ClaudeHistoryEntry = serde_json::from_str(line).ok()?;

            // Filter by project
            if let Some(ref p) = entry.project {
                if !p.contains(project) {
                    return None;
                }
            } else {
                return None;
            }

            let text = entry.display.trim();
            if text.starts_with('/') || text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        })
        .take(count)
        .collect()
}
