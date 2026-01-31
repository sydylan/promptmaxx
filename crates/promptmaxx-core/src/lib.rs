//! promptmaxx-core: Shared library for promptmaxx

pub mod claude_history;
pub mod db;
pub mod error;
pub mod git;
pub mod models;

pub use db::{get_data_dir, init_db};
pub use error::{CoreError, Result};
pub use git::{get_git_info, GitInfo};
pub use models::Prompt;
pub use claude_history::read_last_claude_prompt;
