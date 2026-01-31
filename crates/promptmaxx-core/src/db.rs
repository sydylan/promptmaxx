use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{CoreError, Result};
use crate::models::{Interaction, Pattern, Prompt};

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

    // Prompts table (existing)
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

    // Index for faster duplicate checking
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompts_text ON prompts(text)",
        [],
    )
    .ok();

    // Interactions table (new)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS interactions (
            id TEXT PRIMARY KEY,
            original_prompt TEXT NOT NULL,
            enhanced_prompt TEXT NOT NULL,
            response_summary TEXT,
            effectiveness_score REAL,
            repo TEXT,
            branch TEXT,
            model TEXT,
            duration_ms INTEGER,
            timestamp TEXT NOT NULL
        )",
        [],
    )?;

    // Patterns table (new)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS patterns (
            id TEXT PRIMARY KEY,
            pattern_type TEXT NOT NULL,
            description TEXT NOT NULL,
            success_count INTEGER DEFAULT 0,
            failure_count INTEGER DEFAULT 0
        )",
        [],
    )?;

    Ok(conn)
}

// =============================================================================
// Prompt operations
// =============================================================================

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

// =============================================================================
// Interaction operations
// =============================================================================

/// Save a new interaction
pub fn save_interaction(db: &Connection, interaction: &Interaction) -> Result<()> {
    db.execute(
        "INSERT INTO interactions (id, original_prompt, enhanced_prompt, response_summary, effectiveness_score, repo, branch, model, duration_ms, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            interaction.id,
            interaction.original_prompt,
            interaction.enhanced_prompt,
            interaction.response_summary,
            interaction.effectiveness_score,
            interaction.repo,
            interaction.branch,
            interaction.model,
            interaction.duration_ms,
            interaction.timestamp
        ],
    )?;
    Ok(())
}

/// Create a new interaction with auto-generated id and timestamp
pub fn create_interaction(
    original_prompt: String,
    enhanced_prompt: String,
    repo: Option<String>,
    branch: Option<String>,
    model: Option<String>,
) -> Interaction {
    Interaction {
        id: Uuid::new_v4().to_string(),
        original_prompt,
        enhanced_prompt,
        response_summary: None,
        effectiveness_score: None,
        repo,
        branch,
        model,
        duration_ms: None,
        timestamp: Utc::now().to_rfc3339(),
    }
}

/// Update interaction with response analysis
pub fn update_interaction_analysis(
    db: &Connection,
    id: &str,
    response_summary: Option<&str>,
    effectiveness_score: Option<f64>,
    duration_ms: Option<i64>,
) -> Result<()> {
    db.execute(
        "UPDATE interactions SET response_summary = ?1, effectiveness_score = ?2, duration_ms = ?3 WHERE id = ?4",
        params![response_summary, effectiveness_score, duration_ms, id],
    )?;
    Ok(())
}

/// Get recent interactions
pub fn get_interactions(db: &Connection, limit: usize) -> Result<Vec<Interaction>> {
    let mut interactions = Vec::new();

    let mut stmt = db.prepare(
        "SELECT id, original_prompt, enhanced_prompt, response_summary, effectiveness_score, repo, branch, model, duration_ms, timestamp
         FROM interactions ORDER BY timestamp DESC LIMIT ?1",
    )?;

    let mut rows = stmt.query(params![limit as i32])?;

    while let Some(row) = rows.next()? {
        interactions.push(Interaction {
            id: row.get(0)?,
            original_prompt: row.get(1)?,
            enhanced_prompt: row.get(2)?,
            response_summary: row.get(3)?,
            effectiveness_score: row.get(4)?,
            repo: row.get(5)?,
            branch: row.get(6)?,
            model: row.get(7)?,
            duration_ms: row.get(8)?,
            timestamp: row.get(9)?,
        });
    }

    Ok(interactions)
}

/// Get high-scoring interactions for pattern learning
pub fn get_successful_interactions(db: &Connection, min_score: f64) -> Result<Vec<Interaction>> {
    let mut interactions = Vec::new();

    let mut stmt = db.prepare(
        "SELECT id, original_prompt, enhanced_prompt, response_summary, effectiveness_score, repo, branch, model, duration_ms, timestamp
         FROM interactions WHERE effectiveness_score >= ?1 ORDER BY effectiveness_score DESC LIMIT 50",
    )?;

    let mut rows = stmt.query(params![min_score])?;

    while let Some(row) = rows.next()? {
        interactions.push(Interaction {
            id: row.get(0)?,
            original_prompt: row.get(1)?,
            enhanced_prompt: row.get(2)?,
            response_summary: row.get(3)?,
            effectiveness_score: row.get(4)?,
            repo: row.get(5)?,
            branch: row.get(6)?,
            model: row.get(7)?,
            duration_ms: row.get(8)?,
            timestamp: row.get(9)?,
        });
    }

    Ok(interactions)
}

// =============================================================================
// Pattern operations
// =============================================================================

/// Save a new pattern
pub fn save_pattern(db: &Connection, pattern: &Pattern) -> Result<()> {
    db.execute(
        "INSERT INTO patterns (id, pattern_type, description, success_count, failure_count) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            pattern.id,
            pattern.pattern_type,
            pattern.description,
            pattern.success_count,
            pattern.failure_count
        ],
    )?;
    Ok(())
}

/// Get all patterns
pub fn get_patterns(db: &Connection) -> Result<Vec<Pattern>> {
    let mut patterns = Vec::new();

    let mut stmt =
        db.prepare("SELECT id, pattern_type, description, success_count, failure_count FROM patterns ORDER BY (success_count * 1.0 / (success_count + failure_count + 1)) DESC")?;

    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        patterns.push(Pattern {
            id: row.get(0)?,
            pattern_type: row.get(1)?,
            description: row.get(2)?,
            success_count: row.get(3)?,
            failure_count: row.get(4)?,
        });
    }

    Ok(patterns)
}

/// Update pattern success/failure counts
pub fn update_pattern_counts(
    db: &Connection,
    id: &str,
    success_delta: i32,
    failure_delta: i32,
) -> Result<()> {
    db.execute(
        "UPDATE patterns SET success_count = success_count + ?1, failure_count = failure_count + ?2 WHERE id = ?3",
        params![success_delta, failure_delta, id],
    )?;
    Ok(())
}

// =============================================================================
// Stats
// =============================================================================

/// Get interaction statistics
pub fn get_stats(db: &Connection) -> Result<InteractionStats> {
    let total_interactions: i32 =
        db.query_row("SELECT COUNT(*) FROM interactions", [], |row| row.get(0))?;

    let avg_score: Option<f64> = db
        .query_row(
            "SELECT AVG(effectiveness_score) FROM interactions WHERE effectiveness_score IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .ok();

    let total_prompts: i32 = db.query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))?;

    let total_patterns: i32 =
        db.query_row("SELECT COUNT(*) FROM patterns", [], |row| row.get(0))?;

    Ok(InteractionStats {
        total_interactions,
        average_effectiveness: avg_score,
        total_prompts,
        total_patterns,
    })
}

/// Statistics summary
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InteractionStats {
    pub total_interactions: i32,
    pub average_effectiveness: Option<f64>,
    pub total_prompts: i32,
    pub total_patterns: i32,
}

use serde::{Deserialize, Serialize};
