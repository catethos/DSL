// OpenRouter Multi-Provider Example
//
// This example demonstrates how to use multiple LLM providers
// through OpenRouter's unified API.
//
// Setup:
// 1. Get your OpenRouter API key from: https://openrouter.ai/keys
// 2. export OPENROUTER_API_KEY="sk-or-v1-your-key-here"
// 3. Run: cargo run
// 4. Load this file: :load examples/openrouter_multi_provider.dsl

// Define a custom type for analysis results
type Analysis {
  summary: String
  sentiment: String
  key_points: [String]
}

// Claude 3.5 Sonnet via OpenRouter
def AnalyzeWithClaude(text: String) -> Analysis {
  base_url: "https://openrouter.ai/api/v1"
  model: "anthropic/claude-3.5-sonnet"
  api_key_env: "OPENROUTER_API_KEY"
  temperature: 0.7
  prompt: """
    Analyze the following text and provide:
    1. A brief summary
    2. The overall sentiment (positive, negative, or neutral)
    3. Key points as a list

    Text: ${text}
  """
}

// GPT-4 Turbo via OpenRouter
def AnalyzeWithGPT4(text: String) -> Analysis {
  base_url: "https://openrouter.ai/api/v1"
  model: "openai/gpt-4-turbo"
  api_key_env: "OPENROUTER_API_KEY"
  temperature: 0.7
  prompt: """
    Analyze the following text and provide:
    1. A brief summary
    2. The overall sentiment (positive, negative, or neutral)
    3. Key points as a list

    Text: ${text}
  """
}

// Gemini Pro via OpenRouter
def AnalyzeWithGemini(text: String) -> Analysis {
  base_url: "https://openrouter.ai/api/v1"
  model: "google/gemini-pro"
  api_key_env: "OPENROUTER_API_KEY"
  temperature: 0.7
  prompt: """
    Analyze the following text and provide:
    1. A brief summary
    2. The overall sentiment (positive, negative, or neutral)
    3. Key points as a list

    Text: ${text}
  """
}

// Llama 3.1 70B via OpenRouter
def AnalyzeWithLlama(text: String) -> Analysis {
  base_url: "https://openrouter.ai/api/v1"
  model: "meta-llama/llama-3.1-70b-instruct"
  api_key_env: "OPENROUTER_API_KEY"
  temperature: 0.7
  prompt: """
    Analyze the following text and provide:
    1. A brief summary
    2. The overall sentiment (positive, negative, or neutral)
    3. Key points as a list

    Text: ${text}
  """
}

// Example usage:
// Compare results from different models

"Artificial Intelligence is transforming industries worldwide. From healthcare to finance, AI systems are improving efficiency and enabling new capabilities. However, concerns about job displacement and ethical considerations remain important topics of discussion." as sample_text

// Analyze with Claude
AnalyzeWithClaude(sample_text) as claude_result

// Analyze with GPT-4
AnalyzeWithGPT4(sample_text) as gpt4_result

// Analyze with Gemini
AnalyzeWithGemini(sample_text) as gemini_result

// Analyze with Llama
AnalyzeWithLlama(sample_text) as llama_result

// Display results
claude_result
gpt4_result
gemini_result
llama_result
