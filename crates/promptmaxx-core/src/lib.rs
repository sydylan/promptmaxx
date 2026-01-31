//! promptmaxx-core: Shared library for promptmaxx
//!
//! This crate provides common functionality used by both the Tauri HUD app
//! and the pmaxx CLI wrapper.

pub mod claude_history;
pub mod db;
pub mod error;
pub mod git;
pub mod models;

// Re-export commonly used items
pub use db::{get_data_dir, init_db, InteractionStats};
pub use error::{CoreError, Result};
pub use git::{get_git_info, GitInfo};
pub use models::{Interaction, Pattern, Prompt};

// Re-export claude_history functions
pub use claude_history::read_last_claude_prompt;
