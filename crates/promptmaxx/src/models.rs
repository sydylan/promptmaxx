use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A saved prompt
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Prompt {
    pub id: String,
    pub text: String,
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub timestamp: String,
}

impl Prompt {
    pub fn new(text: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            text,
            repo: None,
            branch: None,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    pub fn with_context(text: String, repo: Option<String>, branch: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            text,
            repo,
            branch,
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}
