use thiserror::Error;

#[derive(Error, Debug)]
pub enum OllamaError {
    #[error("Ollama not available: {0}")]
    NotAvailable(String),

    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Enhancement failed: {0}")]
    Enhancement(String),

    #[error("Analysis failed: {0}")]
    Analysis(String),
}

pub type Result<T> = std::result::Result<T, OllamaError>;
