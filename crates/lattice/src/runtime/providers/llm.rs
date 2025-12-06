//! LLM Provider trait for injectable LLM capabilities
//!
//! This module provides the `LlmProvider` trait that allows the Lattice runtime
//! to make LLM API calls through an injectable implementation. This enables:
//!
//! - Default HTTP-based calls (via `DefaultLlmProvider`)
//! - Host-language callbacks (e.g., Elixir NIFs calling back to Elixir GenServer)
//! - Disabled LLM support (via `NoLlmProvider`)
//! - Testing with mock responses

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Error type for LLM operations
#[derive(Debug, Clone)]
pub enum LlmError {
    /// LLM provider is not configured
    NotConfigured(String),
    /// Network or HTTP error
    NetworkError(String),
    /// API returned an error response
    ApiError { status: u16, message: String },
    /// Failed to parse response
    ParseError(String),
    /// Other errors
    Other(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::NotConfigured(msg) => write!(f, "LLM not configured: {}", msg),
            LlmError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            LlmError::ApiError { status, message } => {
                write!(f, "API error ({}): {}", status, message)
            }
            LlmError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LlmError::Other(msg) => write!(f, "LLM error: {}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

/// A message in the LLM conversation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmMessage {
    /// Role of the message sender ("system", "user", or "assistant")
    pub role: String,
    /// Content of the message
    pub content: String,
}

impl LlmMessage {
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// OpenRouter provider routing configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderRouting {
    /// Order of providers to try
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
    /// Allow fallback to other providers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    /// Require specific providers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require: Option<Vec<String>>,
}

/// Configuration for an LLM API call
#[derive(Clone, Debug)]
pub struct LlmRequest {
    /// Base URL for the API (e.g., "https://api.openai.com/v1")
    pub base_url: String,
    /// Model identifier (e.g., "gpt-4", "anthropic/claude-3.5-sonnet")
    pub model: String,
    /// API key (already resolved from environment)
    pub api_key: String,
    /// Messages in the conversation
    pub messages: Vec<LlmMessage>,
    /// Temperature for sampling (0.0 - 2.0)
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Provider routing configuration (for OpenRouter)
    pub provider: Option<ProviderRouting>,
}

impl LlmRequest {
    /// Create a new request with a single user message
    pub fn new(base_url: String, model: String, api_key: String, prompt: String) -> Self {
        Self {
            base_url,
            model,
            api_key,
            messages: vec![LlmMessage::user(prompt)],
            temperature: None,
            max_tokens: None,
            provider: None,
        }
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set provider routing
    pub fn with_provider(mut self, provider: ProviderRouting) -> Self {
        self.provider = Some(provider);
        self
    }
}

/// Token usage information from an LLM response
#[derive(Clone, Debug, Default)]
pub struct LlmUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    /// Number of tokens in the completion
    pub completion_tokens: u32,
}

impl LlmUsage {
    /// Total tokens used
    pub fn total(&self) -> u32 {
        self.prompt_tokens + self.completion_tokens
    }
}

/// Response from an LLM API call
#[derive(Clone, Debug)]
pub struct LlmResponse {
    /// The generated content
    pub content: String,
    /// Token usage (if provided by the API)
    pub usage: Option<LlmUsage>,
}

/// Trait for LLM providers
///
/// Implement this trait to provide custom LLM handling. The trait is designed
/// to be object-safe so it can be used as `Arc<dyn LlmProvider>`.
///
/// # Example
///
/// ```ignore
/// use lattice::runtime::providers::{LlmProvider, LlmRequest, LlmResponse, LlmError};
///
/// struct MyCustomProvider;
///
/// impl LlmProvider for MyCustomProvider {
///     fn call(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
///         // Custom implementation...
///         Ok(LlmResponse {
///             content: "response".to_string(),
///             usage: None,
///         })
///     }
/// }
/// ```
pub trait LlmProvider: Send + Sync {
    /// Make a synchronous LLM API call
    ///
    /// This is synchronous because the VM execution model is synchronous.
    /// Implementations should handle their own async runtime internally if needed.
    fn call(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;

    /// Check if this provider supports streaming
    ///
    /// Default is false. Override to enable streaming support.
    fn supports_streaming(&self) -> bool {
        false
    }
}

/// Default LLM provider using reqwest HTTP client
///
/// This provider makes direct HTTP calls to OpenAI-compatible APIs.
/// It manages its own tokio runtime for async operations.
pub struct DefaultLlmProvider {
    client: reqwest::Client,
    runtime: tokio::runtime::Runtime,
}

impl DefaultLlmProvider {
    /// Create a new default provider
    pub fn new() -> Result<Self, LlmError> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| LlmError::Other(format!("Failed to create tokio runtime: {}", e)))?;

        Ok(Self {
            client: reqwest::Client::new(),
            runtime,
        })
    }

    /// Create with a shared HTTP client
    pub fn with_client(client: reqwest::Client) -> Result<Self, LlmError> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| LlmError::Other(format!("Failed to create tokio runtime: {}", e)))?;

        Ok(Self { client, runtime })
    }
}

impl Default for DefaultLlmProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create default LLM provider")
    }
}

// Internal request/response types for HTTP serialization
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<LlmMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<ProviderRouting>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    #[serde(default)]
    usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

impl LlmProvider for DefaultLlmProvider {
    fn call(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        self.runtime.block_on(async {
            let request_body = ChatCompletionRequest {
                model: request.model,
                messages: request.messages,
                max_tokens: request.max_tokens,
                temperature: request.temperature,
                provider: request.provider,
            };

            let response = self
                .client
                .post(format!("{}/chat/completions", request.base_url))
                .header("Authorization", format!("Bearer {}", request.api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| LlmError::NetworkError(e.to_string()))?;

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let error_text = response.text().await.unwrap_or_default();
                return Err(LlmError::ApiError {
                    status,
                    message: error_text,
                });
            }

            let response_body: ChatCompletionResponse = response
                .json()
                .await
                .map_err(|e| LlmError::ParseError(e.to_string()))?;

            let content = response_body
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .ok_or_else(|| LlmError::ParseError("No response from LLM".to_string()))?;

            let usage = response_body.usage.map(|u| LlmUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
            });

            Ok(LlmResponse { content, usage })
        })
    }
}

/// No-op provider for runtimes without LLM support
///
/// This provider always returns an error. Use it when embedding Lattice
/// in contexts where LLM calls should be disabled.
pub struct NoLlmProvider;

impl LlmProvider for NoLlmProvider {
    fn call(&self, _request: LlmRequest) -> Result<LlmResponse, LlmError> {
        Err(LlmError::NotConfigured(
            "LLM provider not configured. Use RuntimeBuilder::with_llm_provider() to enable LLM support.".to_string(),
        ))
    }
}

/// Type alias for boxed LLM provider
pub type BoxedLlmProvider = Arc<dyn LlmProvider>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_message_constructors() {
        let system = LlmMessage::system("You are helpful");
        assert_eq!(system.role, "system");
        assert_eq!(system.content, "You are helpful");

        let user = LlmMessage::user("Hello");
        assert_eq!(user.role, "user");
        assert_eq!(user.content, "Hello");

        let assistant = LlmMessage::assistant("Hi there!");
        assert_eq!(assistant.role, "assistant");
        assert_eq!(assistant.content, "Hi there!");
    }

    #[test]
    fn test_llm_request_builder() {
        let request = LlmRequest::new(
            "https://api.openai.com/v1".to_string(),
            "gpt-4".to_string(),
            "sk-test".to_string(),
            "Hello".to_string(),
        )
        .with_temperature(0.7)
        .with_max_tokens(100);

        assert_eq!(request.base_url, "https://api.openai.com/v1");
        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].content, "Hello");
    }

    #[test]
    fn test_llm_usage_total() {
        let usage = LlmUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
        };
        assert_eq!(usage.total(), 150);
    }

    #[test]
    fn test_no_llm_provider_returns_error() {
        let provider = NoLlmProvider;
        let request = LlmRequest::new(
            "https://api.openai.com/v1".to_string(),
            "gpt-4".to_string(),
            "key".to_string(),
            "test".to_string(),
        );

        let result = provider.call(request);
        assert!(result.is_err());
        match result {
            Err(LlmError::NotConfigured(_)) => (),
            _ => panic!("Expected NotConfigured error"),
        }
    }

    #[test]
    fn test_llm_error_display() {
        let errors = vec![
            (LlmError::NotConfigured("test".to_string()), "LLM not configured: test"),
            (LlmError::NetworkError("timeout".to_string()), "Network error: timeout"),
            (LlmError::ApiError { status: 401, message: "unauthorized".to_string() }, "API error (401): unauthorized"),
            (LlmError::ParseError("invalid json".to_string()), "Parse error: invalid json"),
            (LlmError::Other("misc".to_string()), "LLM error: misc"),
        ];

        for (error, expected) in errors {
            assert_eq!(format!("{}", error), expected);
        }
    }
}
