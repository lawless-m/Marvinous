//! Ollama API client
//!
//! "Do you want me to sit in a corner and rust, or just fall apart where I'm standing?"

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OllamaError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Ollama returned an error: {0}")]
    ApiError(String),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

#[derive(Debug, Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
    done: bool,
    #[serde(default)]
    error: Option<String>,
}

pub struct OllamaClient {
    endpoint: String,
    model: String,
    client: Client,
}

impl OllamaClient {
    pub fn new(endpoint: &str, model: &str, timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            model: model.to_string(),
            client,
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, OllamaError> {
        let url = format!("{}/api/generate", self.endpoint);

        let request = GenerateRequest {
            model: &self.model,
            prompt,
            stream: false,
        };

        tracing::debug!("Sending prompt to Ollama ({} chars)", prompt.len());

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OllamaError::ApiError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let result: GenerateResponse = response.json().await?;

        if let Some(error) = result.error {
            return Err(OllamaError::ApiError(error));
        }

        if !result.done {
            return Err(OllamaError::ParseError(
                "Response marked as incomplete".to_string(),
            ));
        }

        tracing::debug!("Received response ({} chars)", result.response.len());

        Ok(result.response)
    }

    /// Check if Ollama is reachable
    pub async fn health_check(&self) -> Result<(), OllamaError> {
        let url = format!("{}/api/tags", self.endpoint);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(OllamaError::ApiError(format!(
                "Health check failed: HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }
}
