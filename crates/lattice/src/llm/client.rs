//! HTTP client for calling LLM APIs
//!
//! Simplified wrapper around reqwest that handles common LLM API patterns.
//! Supports OpenAI-compatible APIs.

use crate::vm::bytecode::ProviderConfig;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LLM Client configuration
#[derive(Debug, Clone)]
pub struct LLMClient {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub provider: Option<ProviderConfig>,
}

impl LLMClient {
    /// Create a new OpenAI client
    pub fn openai(api_key: String, model: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            model,
            max_tokens: None,
            temperature: None,
            provider: None,
        }
    }

    /// Create a new Anthropic client
    pub fn anthropic(api_key: String, model: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            model,
            max_tokens: None,
            temperature: None,
            provider: None,
        }
    }

    /// Create a custom client (e.g., OpenRouter)
    pub fn custom(api_key: String, base_url: String, model: String) -> Self {
        Self {
            api_key,
            base_url,
            model,
            max_tokens: None,
            temperature: None,
            provider: None,
        }
    }

    /// Set the temperature for sampling
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the maximum tokens to generate
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set the provider configuration (for OpenRouter)
    pub fn with_provider(mut self, provider: ProviderConfig) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Call the LLM with a prompt (creates a new HTTP client per call)
    /// For better performance with multiple calls, use `call_with_client` with a shared client.
    pub async fn call(&self, prompt: &str) -> Result<String> {
        let client = reqwest::Client::new();
        self.call_with_client(prompt, &client).await
    }

    /// Call the LLM with a prompt using a shared HTTP client for connection pooling
    pub async fn call_with_client(&self, prompt: &str, http_client: &reqwest::Client) -> Result<String> {
        // Build the request body (OpenAI format, with optional OpenRouter provider field)
        let request_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            provider: self.provider.clone(),
        };

        // Make the HTTP request using the shared client
        let response = http_client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to LLM API")?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API error ({}): {}", status, error_text);
        }

        // Parse the response
        let response_body: ChatCompletionResponse = response
            .json()
            .await
            .context("Failed to parse LLM API response")?;

        // Extract the content
        response_body
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response from LLM"))
    }
}

/// OpenAI Chat Completion Request (with optional OpenRouter provider field)
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    /// OpenRouter provider routing configuration (ignored by non-OpenRouter providers)
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<ProviderConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// OpenAI Chat Completion Response
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

/// Mock client for testing (doesn't make real API calls)
pub struct MockLLMClient {
    responses: HashMap<String, String>,
}

impl Default for MockLLMClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MockLLMClient {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    /// Add a mock response for a specific prompt pattern
    pub fn add_response(&mut self, pattern: &str, response: &str) {
        self.responses
            .insert(pattern.to_string(), response.to_string());
    }

    /// Call the mock client
    pub fn call(&self, prompt: &str) -> Result<String> {
        // Find the first matching pattern
        for (pattern, response) in &self.responses {
            if prompt.contains(pattern) {
                return Ok(response.clone());
            }
        }

        // Default response
        Ok(r#"{"name": "Mock Response", "age": 25}"#.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_client() {
        let mut client = MockLLMClient::new();
        client.add_response("Extract person", r#"{"name": "John", "age": 30}"#);

        let response = client.call("Extract person info from text").unwrap();
        assert_eq!(response, r#"{"name": "John", "age": 30}"#);
    }

    #[test]
    fn test_client_configuration() {
        let client = LLMClient::openai("test-key".to_string(), "gpt-4".to_string());
        assert_eq!(client.model, "gpt-4");
        assert_eq!(client.base_url, "https://api.openai.com/v1");
    }
}
