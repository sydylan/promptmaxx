// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::Utc;
use promptmaxx_core::{
    db::{self, prompt_exists, save_prompt as db_save_prompt},
    get_git_info, init_db, read_last_claude_prompt, Prompt,
};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{
    AppHandle, Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
struct SaveResult {
    success: bool,
    message: String,
    prompt_preview: String,
    is_duplicate: bool,
}

struct AppState {
    db: Mutex<rusqlite::Connection>,
}

// Save a prompt (with deduplication)
#[tauri::command]
fn save_prompt(state: tauri::State<AppState>, text: String) -> Result<SaveResult, String> {
    let db = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;

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

    let git_info = get_git_info();
    let prompt = Prompt {
        id: Uuid::new_v4().to_string(),
        text,
        repo: git_info.repo,
        branch: git_info.branch,
        timestamp: Utc::now().to_rfc3339(),
    };

    db_save_prompt(&db, &prompt).map_err(|e| e.to_string())?;

    Ok(SaveResult {
        success: true,
        message: "Saved".to_string(),
        prompt_preview: preview,
        is_duplicate: false,
    })
}

// Get prompts with improved search (case-insensitive)
#[tauri::command]
fn get_prompts(
    state: tauri::State<AppState>,
    query: Option<String>,
) -> Result<Vec<Prompt>, String> {
    let db = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    db::get_prompts(&db, query.as_deref()).map_err(|e| e.to_string())
}

// Delete a prompt
#[tauri::command]
fn delete_prompt(state: tauri::State<AppState>, id: String) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    db::delete_prompt(&db, &id).map_err(|e| e.to_string())
}

// Get prompt count
#[tauri::command]
fn get_prompt_count(state: tauri::State<AppState>) -> Result<i32, String> {
    let db = state.db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    db::get_prompt_count(&db).map_err(|e| e.to_string())
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
    let db = init_db().expect("Failed to initialize database");

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
