//! LLM integration module
//!
//! Provides HTTP client, prompt rendering, response parsing, and streaming support
//! for LLM function calls.

pub mod client;
pub mod parser;
pub mod partial_parser;
pub mod renderer;
pub mod runtime;
pub mod schema;
pub mod streaming;

// Re-export commonly used types
pub use client::{LLMClient, MockLLMClient};
pub use parser::Parser;
pub use renderer::PromptRenderer;
pub use runtime::{
    generate_prompt_from_ir, parse_llm_response_with_ir, try_parse_partial_response,
    update_streaming_response, LLMRuntime, RuntimeBuilder,
};
pub use schema::SchemaFormatter;
pub use streaming::{CompletionState, StreamingCapable, StreamingValue};
