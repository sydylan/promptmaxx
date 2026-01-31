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

/// An interaction record tracking prompt enhancement and response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Interaction {
    pub id: String,
    pub original_prompt: String,
    pub enhanced_prompt: String,
    pub response_summary: Option<String>,
    pub effectiveness_score: Option<f64>,
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub model: Option<String>,
    pub duration_ms: Option<i64>,
    pub timestamp: String,
}

/// A learned pattern for prompt enhancement
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pattern {
    pub id: String,
    pub pattern_type: String,
    pub description: String,
    pub success_count: i32,
    pub failure_count: i32,
}

impl Pattern {
    /// Calculate success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            0.5 // Neutral for new patterns
        } else {
            self.success_count as f64 / total as f64
        }
    }
}
