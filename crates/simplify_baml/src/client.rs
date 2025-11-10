/// HTTP client for calling LLM APIs
///
/// Simplified wrapper around reqwest that handles common LLM API patterns.
/// Supports OpenAI-compatible APIs.
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
        }
    }

    /// Create a custom client
    pub fn custom(api_key: String, base_url: String, model: String) -> Self {
        Self {
            api_key,
            base_url,
            model,
            max_tokens: None,
            temperature: None,
        }
    }

    /// Call the LLM with a prompt
    pub async fn call(&self, prompt: &str) -> Result<String> {
        // Build the request body (OpenAI format)
        let request_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };

        // Make the HTTP request
        let client = reqwest::Client::new();
        let response = client
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

/// OpenAI Chat Completion Request
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
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
        self.responses.insert(pattern.to_string(), response.to_string());
    }

    /// Call the mock client
    pub async fn call(&self, prompt: &str) -> Result<String> {
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

    #[tokio::test]
    async fn test_mock_client() {
        let mut client = MockLLMClient::new();
        client.add_response("Extract person", r#"{"name": "John", "age": 30}"#);

        let response = client.call("Extract person info from text").await.unwrap();
        assert_eq!(response, r#"{"name": "John", "age": 30}"#);
    }

    #[test]
    fn test_client_configuration() {
        let client = LLMClient::openai("test-key".to_string(), "gpt-4".to_string());
        assert_eq!(client.model, "gpt-4");
        assert_eq!(client.base_url, "https://api.openai.com/v1");
    }
}
