use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{OllamaError, Result};

const DEFAULT_BASE_URL: &str = "http://localhost:11434";

#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
    #[allow(dead_code)]
    done: bool,
}


/// Client for interacting with Ollama API
pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    /// Create a new Ollama client with the specified model
    pub fn new(model: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            model: model.to_string(),
        }
    }

    /// Create a client with a custom base URL
    pub fn with_base_url(model: &str, base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            model: model.to_string(),
        }
    }

    /// Check if Ollama is available and the model exists
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Generate a response from Ollama
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);

        let request = GenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| OllamaError::NotAvailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OllamaError::NotAvailable(format!(
                "Ollama returned status {}",
                response.status()
            )));
        }

        let result: GenerateResponse = response.json().await?;
        Ok(result.response)
    }

    /// Get the model name
    pub fn model(&self) -> &str {
        &self.model
    }
}
