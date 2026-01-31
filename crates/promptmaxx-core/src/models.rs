use serde::{Deserialize, Serialize};

/// A saved prompt with metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Prompt {
    pub id: String,
    pub text: String,
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub timestamp: String,
}
