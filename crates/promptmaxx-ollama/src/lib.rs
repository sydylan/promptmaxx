//! promptmaxx-ollama: Ollama integration for prompt enhancement
//!
//! This crate provides:
//! - `OllamaClient`: HTTP client for Ollama API
//! - `PromptOptimizer`: Enhances prompts using context and patterns
//! - `EffectivenessAnalyzer`: Scores prompt/response effectiveness

mod analysis;
mod client;
mod error;

pub use analysis::{AnalysisResult, EffectivenessAnalyzer, PromptOptimizer};
pub use client::OllamaClient;
pub use error::{OllamaError, Result};
