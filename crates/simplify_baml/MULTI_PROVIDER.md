# Multi-Provider Support Guide

This guide explains how to use multiple LLM providers with simplify_baml.

## Overview

The crate supports three provider configurations:
1. **OpenAI** - Direct OpenAI API access
2. **Anthropic** - Configured but requires OpenRouter (see below)
3. **Custom** - Any OpenAI-compatible endpoint

## Current Implementation Status

The HTTP client currently uses **OpenAI's API format**. This means:

| Provider | Support Level | Notes |
|----------|---------------|-------|
| OpenAI | ✅ Full | Native support |
| OpenRouter | ✅ Full | **Recommended** for multi-provider access |
| OpenAI-compatible APIs | ✅ Full | LocalAI, LM Studio, vLLM, etc. |
| Anthropic Direct | ⚠️ Limited | Use OpenRouter instead |
| Google Gemini Direct | ⚠️ Limited | Use OpenRouter instead |

## Recommended Approach: OpenRouter

[OpenRouter](https://openrouter.ai) is the **recommended solution** for accessing multiple providers because:

- ✅ Single API key for 100+ models
- ✅ Uses OpenAI-compatible API (works out of the box)
- ✅ Access to Claude, GPT-4, Gemini, Llama, and more
- ✅ Automatic failover and load balancing
- ✅ Simple pricing (pay-as-you-go)

### Setup

1. Get your API key at https://openrouter.ai/keys
2. Set environment variable:
   ```bash
   export OPENROUTER_API_KEY="your-key-here"
   ```

### Single Provider Example

```rust
use simplify_baml::*;
use simplify_baml_macros::BamlClient;

#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "https://openrouter.ai/api/v1",
    model = "anthropic/claude-3.5-sonnet"
)]
struct ClaudeClient;

// Use it
let client = ClaudeClient::new(api_key);
let runtime = RuntimeBuilder::new()
    .ir(ir)
    .client("claude", client)
    .build();
```

### Multiple Providers Example

```rust
use simplify_baml::*;
use simplify_baml_macros::BamlClient;

// Claude 3.5 Sonnet via OpenRouter
#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "https://openrouter.ai/api/v1",
    model = "anthropic/claude-3.5-sonnet"
)]
struct ClaudeClient;

// GPT-4 Turbo via OpenRouter
#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "https://openrouter.ai/api/v1",
    model = "openai/gpt-4-turbo"
)]
struct GPT4Client;

// Gemini Pro via OpenRouter
#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "https://openrouter.ai/api/v1",
    model = "google/gemini-pro"
)]
struct GeminiClient;

// Llama 3 70B via OpenRouter
#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "https://openrouter.ai/api/v1",
    model = "meta-llama/llama-3-70b-instruct"
)]
struct LlamaClient;

// Register all clients
let runtime = RuntimeBuilder::new()
    .ir(ir)
    .client("claude", ClaudeClient::new(api_key.clone()))
    .client("gpt4", GPT4Client::new(api_key.clone()))
    .client("gemini", GeminiClient::new(api_key.clone()))
    .client("llama", LlamaClient::new(api_key.clone()))
    .build();
```

### Using Different Providers for Different Functions

```rust
use simplify_baml_macros::baml_function;

// Use Claude for complex reasoning
#[baml_function(client = "claude")]
fn analyze_complex_data(data: String) -> Analysis {
    "Perform deep analysis on: {{ data }}"
}

// Use GPT-4 for creative tasks
#[baml_function(client = "gpt4")]
fn generate_creative_content(prompt: String) -> Content {
    "Generate creative content for: {{ prompt }}"
}

// Use Llama for fast, simple tasks
#[baml_function(client = "llama")]
fn quick_extraction(text: String) -> Summary {
    "Quick summary of: {{ text }}"
}
```

## Available Models on OpenRouter

### Anthropic
- `anthropic/claude-3.5-sonnet` - Latest, most capable
- `anthropic/claude-3-opus` - Previous flagship
- `anthropic/claude-3-sonnet` - Balanced
- `anthropic/claude-3-haiku` - Fast and cheap

### OpenAI
- `openai/gpt-4-turbo` - Latest GPT-4
- `openai/gpt-4` - Original GPT-4
- `openai/gpt-3.5-turbo` - Fast and cheap

### Google
- `google/gemini-pro` - Google's flagship
- `google/gemini-pro-vision` - With image support

### Meta
- `meta-llama/llama-3-70b-instruct` - Large, capable
- `meta-llama/llama-3-8b-instruct` - Fast, efficient

### And Many More
See the full list at: https://openrouter.ai/models

## Direct OpenAI Example

For direct OpenAI access without OpenRouter:

```rust
use simplify_baml_macros::BamlClient;

#[derive(BamlClient)]
#[baml(provider = "OpenAI", model = "gpt-4-turbo")]
struct OpenAIClient;

let client = OpenAIClient::new(openai_api_key);
let runtime = RuntimeBuilder::new()
    .ir(ir)
    .client("openai", client)
    .build();
```

## Other OpenAI-Compatible Services

Many services provide OpenAI-compatible APIs:

### LocalAI (Run models locally)
```rust
#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "http://localhost:8080/v1",
    model = "local-model"
)]
struct LocalClient;
```

### LM Studio (Local development)
```rust
#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "http://localhost:1234/v1",
    model = "local-model"
)]
struct LMStudioClient;
```

### vLLM (Production inference)
```rust
#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "http://your-vllm-server:8000/v1",
    model = "your-model"
)]
struct VLLMClient;
```

## Running the Example

```bash
# Set your OpenRouter API key
export OPENROUTER_API_KEY="your-key-here"

# Run the complete OpenRouter example
cargo run --example openrouter_example
```

## Cost Optimization Strategy

Different models have different costs. A smart strategy:

1. **Use cheaper models for simple tasks** (Llama, GPT-3.5)
2. **Use expensive models for complex tasks** (Claude Opus, GPT-4)
3. **Use fast models for real-time needs** (Haiku, GPT-3.5 Turbo)

Example:
```rust
// Cheap, fast extraction
#[baml_function(client = "llama")]
fn quick_extract(text: String) -> BasicInfo { /* ... */ }

// Expensive, smart analysis
#[baml_function(client = "claude")]
fn deep_analysis(data: String) -> ComplexAnalysis { /* ... */ }
```

## Future Improvements

To add native support for other providers, the HTTP client (`src/client.rs`) would need to:
- Detect provider from configuration
- Format requests according to each provider's API spec
- Handle provider-specific response formats

For now, OpenRouter provides the simplest and most flexible solution.
