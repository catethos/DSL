# rust-genai Usage Guide (AI Agent Reference)

## Install

```toml
[dependencies]
genai = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Supported Providers

OpenAI, Anthropic (Claude), Gemini, Groq, Ollama, xAI (Grok), DeepSeek, Cohere, Fireworks, Together AI, Zhipu

Auto-detect provider from model name prefix.

## Basic Setup

```rust
use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::default();
    // ready to use
}
```

## Authentication

Auto uses env vars:
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`
- `GEMINI_API_KEY`
- `GROQ_API_KEY`
- etc.

Custom auth:
```rust
use genai::resolver::{AuthResolver, AuthData};

let auth_resolver = AuthResolver::from_resolver_fn(
    |model_iden| {
        let key = std::env::var("OPENAI_API_KEY")?;
        Ok(Some(AuthData::from_single(key)))
    }
);

let client = Client::builder()
    .with_auth_resolver(auth_resolver)
    .build();
```

## Simple Chat

```rust
let chat_req = ChatRequest::new(vec![
    ChatMessage::system("Answer in one sentence"),
    ChatMessage::user("Why is sky blue?"),
]);

let chat_res = client.exec_chat("gpt-4o-mini", chat_req, None).await?;
println!("{}", chat_res.content_text_as_str().unwrap());
```

## Streaming Chat

```rust
let chat_res = client.exec_chat_stream("gpt-4o-mini", chat_req, None).await?;

while let Some(chunk) = chat_res.stream.next().await {
    let chunk = chunk?;
    if let Some(content) = chunk.content_text_as_str() {
        print!("{}", content);
    }
}
```

## Conversation (Multi-turn)

```rust
let mut chat_req = ChatRequest::default()
    .with_system("Answer in one sentence");

// First turn
chat_req = chat_req.append_message(ChatMessage::user("Why is sky blue?"));
let chat_res = client.exec_chat("gpt-4o-mini", chat_req.clone(), None).await?;
let answer = chat_res.content_text_as_str().unwrap();

// Append assistant response
chat_req = chat_req.append_message(ChatMessage::assistant(answer));

// Next turn
chat_req = chat_req.append_message(ChatMessage::user("Why red sometimes?"));
let chat_res = client.exec_chat("gpt-4o-mini", chat_req.clone(), None).await?;
```

## Chat Options

Client-level defaults:
```rust
use genai::chat::{ChatOptions, ClientConfig};

let client_config = ClientConfig::default()
    .with_chat_options(ChatOptions::default()
        .with_temperature(0.0)
        .with_top_p(0.99));

let client = Client::builder()
    .with_config(client_config)
    .build();
```

Per-request override:
```rust
let options = ChatOptions::default()
    .with_max_tokens(1000)
    .with_temperature(0.5);

let chat_res = client.exec_chat("gpt-4o-mini", chat_req, Some(&options)).await?;
```

## Tool/Function Calling

Define tool:
```rust
use genai::chat::Tool;
use serde_json::json;

let tool = Tool::new("get_weather")
    .with_description("Get weather for location")
    .with_schema(json!({
        "type": "object",
        "properties": {
            "location": {"type": "string"},
            "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}
        },
        "required": ["location"]
    }));
```

Request with tools:
```rust
let chat_req = ChatRequest::new(vec![
    ChatMessage::user("What's weather in Tokyo?")
]).with_tools(vec![tool]);

let chat_res = client.exec_chat("gpt-4o-mini", chat_req.clone(), None).await?;
let tool_calls = chat_res.into_tool_calls();
```

Process tool calls:
```rust
for call in tool_calls {
    // Execute your function
    let result = execute_function(&call.name, &call.arguments);

    // Append tool call and result to chat
    chat_req = chat_req.append_message(
        ChatMessage::tool_calls(vec![call])
    );
    chat_req = chat_req.append_message(
        ChatMessage::tool_result(call.id, result)
    );
}

// Get final response
let chat_res = client.exec_chat("gpt-4o-mini", chat_req, None).await?;
```

## Vision/Image Support

```rust
use genai::chat::ContentPart;

let chat_req = ChatRequest::new(vec![
    ChatMessage::user(vec![
        ContentPart::from_text("What is in this picture?"),
        ContentPart::from_binary_url("image/jpg", "https://example.com/image.jpg", None),
    ])
]);

let chat_res = client.exec_chat("gpt-4o-mini", chat_req, None).await?;
```

## Embeddings

Single embedding:
```rust
let embedding = client.embed("text-embedding-3-small", "hello world", None).await?;
println!("Vector: {:?}", embedding.vec);
println!("Dimensions: {}", embedding.dimensions);
```

Batch embeddings:
```rust
let texts = vec!["text1", "text2", "text3"];
let embeddings = client.embed_batch("text-embedding-3-small", texts, None).await?;
```

With options:
```rust
use genai::embed::EmbedOptions;

let options = EmbedOptions::new()
    .with_dimensions(512)
    .with_capture_usage(true)
    .with_user("user-123");

let embedding = client.embed("text-embedding-3-small", "text", Some(&options)).await?;
```

## Model Discovery

```rust
use genai::adapter::AdapterKind;

let models = client.all_model_names(AdapterKind::OpenAI).await?;
for model in models {
    println!("{}", model);
}

// Available: AdapterKind::OpenAI, Anthropic, Gemini, Ollama, Groq, Cohere
```

## Model Name Examples

- OpenAI: `gpt-4o`, `gpt-4o-mini`, `gpt-3.5-turbo`
- Anthropic: `claude-3-5-sonnet-20241022`, `claude-3-opus-20240229`
- Gemini: `gemini-1.5-pro`, `gemini-1.5-flash`
- Groq: `llama-3.1-70b-versatile`, `mixtral-8x7b-32768`
- Ollama: `llama3.2`, `mistral` (local models)

Provider auto-detected from model name.

## Error Handling

```rust
match client.exec_chat("gpt-4o-mini", chat_req, None).await {
    Ok(res) => println!("{}", res.content_text_as_str().unwrap()),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Common Patterns

### Stream and collect response
```rust
let mut full_response = String::new();
let chat_res = client.exec_chat_stream(model, chat_req, None).await?;

while let Some(chunk) = chat_res.stream.next().await {
    let chunk = chunk?;
    if let Some(text) = chunk.content_text_as_str() {
        print!("{}", text);
        full_response.push_str(text);
    }
}

// Append full response to conversation
chat_req = chat_req.append_message(ChatMessage::assistant(full_response));
```

### Multi-provider fallback
```rust
let models = vec!["gpt-4o-mini", "claude-3-5-sonnet-20241022", "gemini-1.5-flash"];

for model in models {
    match client.exec_chat(model, chat_req.clone(), None).await {
        Ok(res) => {
            println!("Success with {}: {}", model, res.content_text_as_str().unwrap());
            break;
        }
        Err(e) => {
            eprintln!("Failed with {}: {}", model, e);
            continue;
        }
    }
}
```

## Key Points

- Client reusable across requests
- ChatRequest cloneable for multi-turn conversations
- Options cascade: client defaults → request overrides
- Streaming returns async iterator
- Tool calling requires manual function execution
- Auto provider routing based on model name
- All operations async, need tokio runtime
