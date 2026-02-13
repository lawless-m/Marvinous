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
        self.generate_with_retry(prompt, 3, 30).await
    }

    /// Generate with OOM retry pattern
    /// Implements the GPU memory sharing pattern: retry with delays to allow other services to unload
    async fn generate_with_retry(
        &self,
        prompt: &str,
        max_retries: u32,
        retry_delay_secs: u64,
    ) -> Result<String, OllamaError> {
        let url = format!("{}/api/generate", self.endpoint);

        let request = GenerateRequest {
            model: &self.model,
            prompt,
            stream: false,
        };

        for attempt in 1..=max_retries {
            tracing::debug!(
                "Sending prompt to Ollama ({} chars) - attempt {}/{}",
                prompt.len(),
                attempt,
                max_retries
            );

            let response = match self.client.post(&url).json(&request).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    if attempt < max_retries {
                        tracing::warn!(
                            "Request failed (attempt {}/{}): {} - retrying in {}s",
                            attempt,
                            max_retries,
                            e,
                            retry_delay_secs
                        );
                        tokio::time::sleep(Duration::from_secs(retry_delay_secs)).await;
                        continue;
                    } else {
                        return Err(OllamaError::RequestError(e));
                    }
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                let error_msg = format!("HTTP {}: {}", status, body);

                // Check if this might be an OOM or resource error
                let is_resource_error = body.to_lowercase().contains("memory")
                    || body.to_lowercase().contains("resource")
                    || status == 503
                    || status == 500;

                if is_resource_error && attempt < max_retries {
                    tracing::warn!(
                        "GPU/resource error (attempt {}/{}): {} - waiting {}s for other services to unload",
                        attempt,
                        max_retries,
                        error_msg,
                        retry_delay_secs
                    );
                    tokio::time::sleep(Duration::from_secs(retry_delay_secs)).await;
                    continue;
                } else {
                    return Err(OllamaError::ApiError(error_msg));
                }
            }

            let result: GenerateResponse = response.json().await?;

            if let Some(error) = result.error {
                // Check if the error indicates OOM/resource issues
                let is_resource_error = error.to_lowercase().contains("memory")
                    || error.to_lowercase().contains("resource")
                    || error.to_lowercase().contains("cuda");

                if is_resource_error && attempt < max_retries {
                    tracing::warn!(
                        "Ollama reported resource error (attempt {}/{}): {} - waiting {}s",
                        attempt,
                        max_retries,
                        error,
                        retry_delay_secs
                    );
                    tokio::time::sleep(Duration::from_secs(retry_delay_secs)).await;
                    continue;
                } else {
                    return Err(OllamaError::ApiError(error));
                }
            }

            if !result.done {
                return Err(OllamaError::ParseError(
                    "Response marked as incomplete".to_string(),
                ));
            }

            tracing::info!(
                "Received response ({} chars) on attempt {}/{}",
                result.response.len(),
                attempt,
                max_retries
            );

            return Ok(result.response);
        }

        // Should never reach here due to the loop logic, but satisfy the compiler
        Err(OllamaError::ApiError(
            "Max retries exceeded".to_string(),
        ))
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
