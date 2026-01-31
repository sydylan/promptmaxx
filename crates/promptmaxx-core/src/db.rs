use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{CoreError, Result};
use crate::models::Prompt;

/// Get the data directory (~/.promptmaxx)
pub fn get_data_dir() -> Result<std::path::PathBuf> {
    let mut path = dirs::home_dir().ok_or(CoreError::HomeDirNotFound)?;
    path.push(".promptmaxx");
    std::fs::create_dir_all(&path)
        .map_err(|e| CoreError::DataDir(format!("Could not create data directory: {}", e)))?;
    Ok(path)
}

/// Initialize the database and return a connection
pub fn init_db() -> Result<Connection> {
    let mut db_path = get_data_dir()?;
    db_path.push("prompts.db");

    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompts (
            id TEXT PRIMARY KEY,
            text TEXT NOT NULL,
            repo TEXT,
            branch TEXT,
            timestamp TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompts_text ON prompts(text)",
        [],
    )
    .ok();

    Ok(conn)
}

/// Check if a prompt already exists
pub fn prompt_exists(db: &Connection, text: &str) -> bool {
    let count: i32 = db
        .query_row(
            "SELECT COUNT(*) FROM prompts WHERE text = ?1",
            params![text],
            |row| row.get(0),
        )
        .unwrap_or(0);
    count > 0
}

/// Save a new prompt
pub fn save_prompt(db: &Connection, prompt: &Prompt) -> Result<()> {
    db.execute(
        "INSERT INTO prompts (id, text, repo, branch, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            prompt.id,
            prompt.text,
            prompt.repo,
            prompt.branch,
            prompt.timestamp
        ],
    )?;
    Ok(())
}

/// Create a new prompt with auto-generated id and timestamp
pub fn create_prompt(text: String, repo: Option<String>, branch: Option<String>) -> Prompt {
    Prompt {
        id: Uuid::new_v4().to_string(),
        text,
        repo,
        branch,
        timestamp: Utc::now().to_rfc3339(),
    }
}

/// Get prompts with optional search filter
pub fn get_prompts(db: &Connection, query: Option<&str>) -> Result<Vec<Prompt>> {
    let mut prompts = Vec::new();

    let search_term = query
        .filter(|q| !q.trim().is_empty())
        .map(|q| format!("%{}%", q.to_lowercase()));

    let sql = if search_term.is_some() {
        "SELECT id, text, repo, branch, timestamp FROM prompts WHERE LOWER(text) LIKE ?1 ORDER BY timestamp DESC LIMIT 100"
    } else {
        "SELECT id, text, repo, branch, timestamp FROM prompts ORDER BY timestamp DESC LIMIT 100"
    };

    let mut stmt = db.prepare(sql)?;

    let mut rows = if let Some(ref term) = search_term {
        stmt.query(params![term])?
    } else {
        stmt.query([])?
    };

    while let Some(row) = rows.next()? {
        prompts.push(Prompt {
            id: row.get(0)?,
            text: row.get(1)?,
            repo: row.get(2)?,
            branch: row.get(3)?,
            timestamp: row.get(4)?,
        });
    }

    Ok(prompts)
}

/// Delete a prompt by ID
pub fn delete_prompt(db: &Connection, id: &str) -> Result<bool> {
    let rows_affected = db.execute("DELETE FROM prompts WHERE id = ?1", params![id])?;
    Ok(rows_affected > 0)
}

/// Get total prompt count
pub fn get_prompt_count(db: &Connection) -> Result<i32> {
    let count: i32 = db.query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))?;
    Ok(count)
}
