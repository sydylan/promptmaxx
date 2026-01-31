// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::Utc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use tauri::{
    AppHandle, Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Prompt {
    id: String,
    text: String,
    repo: Option<String>,
    branch: Option<String>,
    timestamp: String,
}

#[derive(Debug, Serialize, Clone)]
struct SaveResult {
    success: bool,
    message: String,
    prompt_preview: String,
    is_duplicate: bool,
}

struct AppState {
    db: Mutex<Connection>,
}

// Get the data directory
fn get_data_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("Could not find home directory");
    path.push(".promptmaxx");
    std::fs::create_dir_all(&path).expect("Could not create data directory");
    path
}

// Initialize the database
fn init_db() -> Connection {
    let mut db_path = get_data_dir();
    db_path.push("prompts.db");

    let conn = Connection::open(&db_path).expect("Could not open database");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompts (
            id TEXT PRIMARY KEY,
            text TEXT NOT NULL,
            repo TEXT,
            branch TEXT,
            timestamp TEXT NOT NULL
        )",
        [],
    )
    .expect("Could not create table");

    // Create index for faster duplicate checking
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompts_text ON prompts(text)",
        [],
    )
    .ok();

    conn
}

// Get git repo info from a directory
fn get_git_info() -> (Option<String>, Option<String>) {
    let repo = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| {
                        s.trim()
                            .split('/')
                            .last()
                            .unwrap_or("")
                            .to_string()
                    })
            } else {
                None
            }
        });

    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    (repo, branch)
}

// Claude Code history entry
#[derive(Debug, Deserialize)]
struct ClaudeHistoryEntry {
    display: String,
    #[allow(dead_code)]
    timestamp: u64,
    #[allow(dead_code)]
    project: Option<String>,
}

// Read the last prompt from Claude Code's history
fn read_last_claude_prompt() -> Option<String> {
    let mut history_path = dirs::home_dir()?;
    history_path.push(".claude");
    history_path.push("history.jsonl");

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

// Check if prompt already exists
fn prompt_exists(db: &Connection, text: &str) -> bool {
    let count: i32 = db
        .query_row(
            "SELECT COUNT(*) FROM prompts WHERE text = ?1",
            params![text],
            |row| row.get(0),
        )
        .unwrap_or(0);
    count > 0
}

// Save a prompt (with deduplication)
#[tauri::command]
fn save_prompt(state: tauri::State<AppState>, text: String) -> Result<SaveResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let preview: String = text.chars().take(60).collect();
    let preview = if text.len() > 60 {
        format!("{}...", preview)
    } else {
        preview
    };

    // Check for duplicate
    if prompt_exists(&db, &text) {
        return Ok(SaveResult {
            success: false,
            message: "Already saved".to_string(),
            prompt_preview: preview,
            is_duplicate: true,
        });
    }

    let (repo, branch) = get_git_info();
    let prompt = Prompt {
        id: Uuid::new_v4().to_string(),
        text,
        repo,
        branch,
        timestamp: Utc::now().to_rfc3339(),
    };

    db.execute(
        "INSERT INTO prompts (id, text, repo, branch, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![prompt.id, prompt.text, prompt.repo, prompt.branch, prompt.timestamp],
    )
    .map_err(|e| e.to_string())?;

    Ok(SaveResult {
        success: true,
        message: "Saved".to_string(),
        prompt_preview: preview,
        is_duplicate: false,
    })
}

// Get prompts with improved search (case-insensitive)
#[tauri::command]
fn get_prompts(state: tauri::State<AppState>, query: Option<String>) -> Result<Vec<Prompt>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut prompts = Vec::new();

    // Build query based on search term
    let search_term = query
        .as_ref()
        .filter(|q| !q.trim().is_empty())
        .map(|q| format!("%{}%", q.to_lowercase()));

    let sql = if search_term.is_some() {
        "SELECT id, text, repo, branch, timestamp FROM prompts WHERE LOWER(text) LIKE ?1 ORDER BY timestamp DESC LIMIT 100"
    } else {
        "SELECT id, text, repo, branch, timestamp FROM prompts ORDER BY timestamp DESC LIMIT 100"
    };

    let mut stmt = db.prepare(sql).map_err(|e| e.to_string())?;

    let mut rows = if let Some(ref term) = search_term {
        stmt.query(params![term]).map_err(|e| e.to_string())?
    } else {
        stmt.query([]).map_err(|e| e.to_string())?
    };

    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        prompts.push(Prompt {
            id: row.get(0).map_err(|e| e.to_string())?,
            text: row.get(1).map_err(|e| e.to_string())?,
            repo: row.get(2).map_err(|e| e.to_string())?,
            branch: row.get(3).map_err(|e| e.to_string())?,
            timestamp: row.get(4).map_err(|e| e.to_string())?,
        });
    }

    Ok(prompts)
}

// Delete a prompt
#[tauri::command]
fn delete_prompt(state: tauri::State<AppState>, id: String) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let rows_affected = db
        .execute("DELETE FROM prompts WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(rows_affected > 0)
}

// Get prompt count
#[tauri::command]
fn get_prompt_count(state: tauri::State<AppState>) -> Result<i32, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let count: i32 = db
        .query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    Ok(count)
}

// Handle the save hotkey
fn handle_save_hotkey(app: &AppHandle) {
    if let Some(text) = read_last_claude_prompt() {
        let state = app.state::<AppState>();
        match save_prompt(state, text) {
            Ok(result) => {
                let _ = app.emit("prompt-saved", result);
            }
            Err(e) => {
                eprintln!("Failed to save prompt: {}", e);
                let _ = app.emit("prompt-error", e);
            }
        }
    } else {
        let _ = app.emit("prompt-error", "No prompt found");
    }
}

// Resize window for expanded/collapsed state
#[tauri::command]
fn set_window_size(app: AppHandle, expanded: bool) {
    if let Some(window) = app.get_webview_window("main") {
        if expanded {
            let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                width: 380.0,
                height: 500.0,
            }));
        } else {
            let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                width: 220.0,
                height: 70.0,
            }));
        }
    }
}

fn main() {
    let db = init_db();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            save_prompt,
            get_prompts,
            delete_prompt,
            get_prompt_count,
            set_window_size
        ])
        .setup(|app| {
            // Register global hotkey: Cmd+Shift+P (less common than Cmd+Shift+S)
            let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyP);
            let app_handle = app.handle().clone();

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, _shortcut, event| {
                        if event.state == ShortcutState::Pressed {
                            handle_save_hotkey(&app_handle);
                        }
                    })
                    .build(),
            )?;

            match app.global_shortcut().register(shortcut) {
                Ok(_) => {
                    eprintln!("Hotkey registered: Cmd+Shift+P");
                    let _ = app.emit("hotkey-registered", "Cmd+Shift+P");
                }
                Err(e) => {
                    eprintln!("Failed to register shortcut: {}", e);
                    let _ = app.emit("hotkey-error", e.to_string());
                }
            }

            // Create system tray
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            let app_handle_for_tray = app.handle().clone();
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("promptmaxx - Cmd+Shift+P to save")
                .on_menu_event(move |_tray, event| {
                    if event.id.as_ref() == "quit" {
                        app_handle_for_tray.exit(0);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let _ = tray.app_handle().emit("toggle-hud", ());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
