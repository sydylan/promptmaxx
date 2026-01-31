//! promptmaxx - Save and retrieve prompts
//!
//! ```rust,ignore
//! use promptmaxx::{save, list};
//!
//! // Save a prompt
//! save("my prompt text")?;
//!
//! // List all prompts
//! let prompts = list()?;
//! ```

mod db;
mod error;
mod models;

pub use error::{Error, Result};
pub use models::Prompt;

use db::get_connection;

/// Save a prompt. Returns the created Prompt.
pub fn save(text: &str) -> Result<Prompt> {
    let conn = get_connection()?;

    if db::exists(&conn, text)? {
        return Err(Error::Duplicate);
    }

    let prompt = Prompt::new(text.to_string());
    db::insert(&conn, &prompt)?;
    Ok(prompt)
}

/// Save a prompt with repo/branch context.
pub fn save_with_context(text: &str, repo: Option<String>, branch: Option<String>) -> Result<Prompt> {
    let conn = get_connection()?;

    if db::exists(&conn, text)? {
        return Err(Error::Duplicate);
    }

    let prompt = Prompt::with_context(text.to_string(), repo, branch);
    db::insert(&conn, &prompt)?;
    Ok(prompt)
}

/// List all prompts (most recent first).
pub fn list() -> Result<Vec<Prompt>> {
    let conn = get_connection()?;
    db::list(&conn, None)
}

/// Search prompts by text.
pub fn search(query: &str) -> Result<Vec<Prompt>> {
    let conn = get_connection()?;
    db::list(&conn, Some(query))
}

/// Delete a prompt by ID.
pub fn delete(id: &str) -> Result<bool> {
    let conn = get_connection()?;
    db::delete(&conn, id)
}

/// Get the number of saved prompts.
pub fn count() -> Result<i32> {
    let conn = get_connection()?;
    db::count(&conn)
}

/// Check if a prompt already exists.
pub fn exists(text: &str) -> Result<bool> {
    let conn = get_connection()?;
    db::exists(&conn, text)
}

/// Update a prompt's text by ID.
pub fn update(id: &str, text: &str) -> Result<bool> {
    let conn = get_connection()?;
    db::update(&conn, id, text)
}
