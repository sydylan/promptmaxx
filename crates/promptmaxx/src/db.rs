use rusqlite::{params, Connection};

use crate::error::{Error, Result};
use crate::models::Prompt;

/// Get the data directory (~/.promptmaxx)
fn get_data_dir() -> Result<std::path::PathBuf> {
    let mut path = dirs::home_dir().ok_or(Error::HomeDirNotFound)?;
    path.push(".promptmaxx");
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

/// Get a database connection (initializes if needed)
pub fn get_connection() -> Result<Connection> {
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

/// Check if a prompt exists
pub fn exists(conn: &Connection, text: &str) -> Result<bool> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM prompts WHERE text = ?1",
        params![text],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Insert a prompt
pub fn insert(conn: &Connection, prompt: &Prompt) -> Result<()> {
    conn.execute(
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

/// List prompts with optional search
pub fn list(conn: &Connection, query: Option<&str>) -> Result<Vec<Prompt>> {
    let mut prompts = Vec::new();

    let search_term = query
        .filter(|q| !q.trim().is_empty())
        .map(|q| format!("%{}%", q.to_lowercase()));

    let sql = if search_term.is_some() {
        "SELECT id, text, repo, branch, timestamp FROM prompts WHERE LOWER(text) LIKE ?1 ORDER BY timestamp DESC LIMIT 100"
    } else {
        "SELECT id, text, repo, branch, timestamp FROM prompts ORDER BY timestamp DESC LIMIT 100"
    };

    let mut stmt = conn.prepare(sql)?;

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
pub fn delete(conn: &Connection, id: &str) -> Result<bool> {
    let rows = conn.execute("DELETE FROM prompts WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}

/// Count prompts
pub fn count(conn: &Connection) -> Result<i32> {
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))?;
    Ok(count)
}
