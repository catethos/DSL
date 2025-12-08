---
name: greet
model: gpt-4o-mini
base_url: "https://openrouter.ai/api/v1"
api_key_env: "OPENROUTER_API_KEY"
temperature: 0.7
input:
  name: String
output: String
---
Write a creative and friendly greeting for someone named {name}.
Keep it brief, just 1-2 sentences.
