//! Markdown LLM function file parsing
//!
//! This module handles parsing of `.md` files that define LLM functions using
//! YAML frontmatter and markdown body for the prompt.
//!
//! # Example Format
//!
//! ```markdown
//! ---
//! name: analyze_sentiment
//! model: gpt-4o-mini
//! input:
//!   text: String
//! output: Sentiment
//! ---
//! Analyze the sentiment of the following text.
//!
//! Text: {text}
//! ```

use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur when parsing markdown LLM files
#[derive(Debug, Error)]
pub enum MarkdownError {
    #[error("Missing YAML frontmatter (file must start with ---)")]
    MissingFrontmatter,

    #[error("Unclosed frontmatter (missing closing ---)")]
    UnclosedFrontmatter,

    #[error("Invalid YAML frontmatter: {0}")]
    YamlError(#[from] serde_yml::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Represents a parsed markdown LLM function definition
#[derive(Debug, Clone)]
pub struct MarkdownLlmDef {
    /// Function name (valid Lattice identifier)
    pub name: String,

    /// Model identifier (e.g., "gpt-4o-mini", "claude-3-sonnet")
    pub model: String,

    /// API base URL (optional, defaults to OpenRouter)
    pub base_url: Option<String>,

    /// Environment variable name for API key
    pub api_key_env: Option<String>,

    /// Temperature for sampling (0.0-2.0)
    pub temperature: Option<f64>,

    /// Maximum tokens in response
    pub max_tokens: Option<usize>,

    /// Reference to an llm_config declaration in the importing file
    pub use_config: Option<String>,

    /// Input parameter definitions
    pub input: InputDef,

    /// Output type definition
    pub output: OutputDef,

    /// OpenRouter provider routing configuration
    pub provider: Option<ProviderDef>,

    /// The prompt body (markdown content after frontmatter)
    pub prompt_body: String,
}

/// Input parameter definitions
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum InputDef {
    /// Multiple named parameters: { text: String, count: Int }
    Map(HashMap<String, String>),

    /// Single parameter shorthand: "text: String"
    Single(String),
}

/// Output type definition
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OutputDef {
    /// Reference to existing type: "Sentiment"
    TypeRef(String),

    /// Inline struct definition: { field: Type, ... } or { field: { type: T, description: "..." }, ... }
    Inline(HashMap<String, FieldDef>),
}

/// Field definition - can be just a type string or a full definition with description
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum FieldDef {
    /// Simple type: "String"
    Simple(String),

    /// Full definition: { type: "String", description: "..." }
    WithDescription { r#type: String, description: String },
}

/// OpenRouter provider configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProviderDef {
    /// Provider preference order
    #[serde(default)]
    pub order: Option<Vec<String>>,

    /// Allow fallback to other providers
    #[serde(default)]
    pub allow_fallbacks: Option<bool>,

    /// Providers to ignore
    #[serde(default)]
    pub ignore: Option<Vec<String>>,

    /// Only use these providers
    #[serde(default)]
    pub only: Option<Vec<String>>,
}

/// Raw frontmatter structure for serde deserialization
#[derive(Debug, Deserialize)]
struct RawFrontmatter {
    name: String,
    model: String,

    #[serde(default)]
    base_url: Option<String>,

    #[serde(default)]
    api_key_env: Option<String>,

    #[serde(default)]
    temperature: Option<f64>,

    #[serde(default)]
    max_tokens: Option<usize>,

    /// `use` is a reserved keyword in Rust, so we rename it
    #[serde(default, rename = "use")]
    use_config: Option<String>,

    input: InputDef,
    output: OutputDef,

    #[serde(default)]
    provider: Option<ProviderDef>,
}

/// Extract YAML frontmatter and body from markdown content.
///
/// # Arguments
///
/// * `content` - The full markdown file content
///
/// # Returns
///
/// A tuple of (yaml_frontmatter, markdown_body) on success.
///
/// # Errors
///
/// Returns an error if:
/// - The file doesn't start with `---`
/// - The closing `---` is missing
pub fn split_frontmatter(content: &str) -> Result<(&str, &str), MarkdownError> {
    let content = content.trim_start();

    if !content.starts_with("---") {
        return Err(MarkdownError::MissingFrontmatter);
    }

    // Skip the opening ---
    let after_start = &content[3..];

    // Find the closing --- (must be on its own line)
    let end_pos = after_start
        .find("\n---")
        .ok_or(MarkdownError::UnclosedFrontmatter)?;

    let yaml = after_start[..end_pos].trim();

    // Body starts after the closing ---
    let body_start = end_pos + 4; // "\n---".len()
    let body = if body_start < after_start.len() {
        after_start[body_start..].trim()
    } else {
        ""
    };

    Ok((yaml, body))
}

/// Parse markdown content into a MarkdownLlmDef.
///
/// # Arguments
///
/// * `content` - The full markdown file content
///
/// # Returns
///
/// A parsed MarkdownLlmDef on success.
///
/// # Errors
///
/// Returns an error if:
/// - The frontmatter is missing or malformed
/// - Required fields are missing
/// - YAML parsing fails
pub fn parse_markdown_llm(content: &str) -> Result<MarkdownLlmDef, MarkdownError> {
    let (yaml, body) = split_frontmatter(content)?;

    let raw: RawFrontmatter = serde_yml::from_str(yaml)?;

    Ok(MarkdownLlmDef {
        name: raw.name,
        model: raw.model,
        base_url: raw.base_url,
        api_key_env: raw.api_key_env,
        temperature: raw.temperature,
        max_tokens: raw.max_tokens,
        use_config: raw.use_config,
        input: raw.input,
        output: raw.output,
        provider: raw.provider,
        prompt_body: body.to_string(),
    })
}

// =============================================================================
// Transpilation: MarkdownLlmDef -> Lattice source code
// =============================================================================

impl MarkdownLlmDef {
    /// Convert markdown LLM definition to equivalent Lattice source code.
    ///
    /// This generates:
    /// 1. An inline type definition if the output uses inline struct syntax
    /// 2. The LLM function definition with all config fields and prompt
    ///
    /// # Example
    ///
    /// Given a markdown file:
    /// ```markdown
    /// ---
    /// name: analyze
    /// model: gpt-4
    /// input:
    ///   text: String
    /// output: Sentiment
    /// ---
    /// Analyze: {text}
    /// ```
    ///
    /// Produces:
    /// ```lattice
    /// def analyze(text: String) -> Sentiment {
    ///     model: "gpt-4"
    ///     prompt: """
    /// Analyze: ${text}
    ///     """
    /// }
    /// ```
    pub fn to_lattice_source(&self) -> String {
        let mut output = String::new();

        // Generate inline type if output uses inline struct syntax
        if let OutputDef::Inline(fields) = &self.output {
            output.push_str(&self.generate_inline_type(fields));
            output.push('\n');
        }

        // Generate the function definition
        output.push_str(&self.generate_function());

        output
    }

    /// Generate an inline type definition for inline output structs.
    fn generate_inline_type(&self, fields: &HashMap<String, FieldDef>) -> String {
        let type_name = format!("{}Output", to_pascal_case(&self.name));
        let mut result = format!("type {} {{\n", type_name);

        // Sort fields for deterministic output
        let mut sorted_fields: Vec<_> = fields.iter().collect();
        sorted_fields.sort_by_key(|(k, _)| *k);

        let field_count = sorted_fields.len();
        for (i, (name, field_def)) in sorted_fields.iter().enumerate() {
            let (typ, desc) = match field_def {
                FieldDef::Simple(t) => (t.as_str(), None),
                FieldDef::WithDescription { r#type, description } => {
                    (r#type.as_str(), Some(description.as_str()))
                }
            };

            // Add field with type
            result.push_str(&format!("    {}: {}", name, typ));

            // Add description if present
            if let Some(d) = desc {
                // Escape quotes in description
                let escaped = d.replace('\\', "\\\\").replace('"', "\\\"");
                result.push_str(&format!(" @\"{}\"", escaped));
            }

            // Add comma if not last field
            if i < field_count - 1 {
                result.push(',');
            }
            result.push('\n');
        }
        result.push_str("}\n");
        result
    }

    /// Generate the LLM function definition.
    fn generate_function(&self) -> String {
        let mut result = String::new();

        // Function signature
        let params = self.format_params();
        let return_type = self.format_return_type();
        result.push_str(&format!(
            "def {}({}) -> {} {{\n",
            self.name, params, return_type
        ));

        // Config fields (in specific order for readability)
        if let Some(use_ref) = &self.use_config {
            result.push_str(&format!("    use: {}\n", use_ref));
        }
        if let Some(url) = &self.base_url {
            result.push_str(&format!("    base_url: \"{}\"\n", url));
        }
        result.push_str(&format!("    model: \"{}\"\n", self.model));
        if let Some(key) = &self.api_key_env {
            result.push_str(&format!("    api_key_env: \"{}\"\n", key));
        }
        if let Some(temp) = self.temperature {
            result.push_str(&format!("    temperature: {}\n", temp));
        }
        if let Some(tokens) = self.max_tokens {
            result.push_str(&format!("    max_tokens: {}\n", tokens));
        }

        // Provider config if present
        if let Some(provider) = &self.provider {
            result.push_str(&self.format_provider(provider));
        }

        // Prompt body - convert {var} to ${var}
        let prompt = convert_template_vars(&self.prompt_body);
        result.push_str(&format!("    prompt: \"\"\"\n{}\n    \"\"\"\n", prompt));

        result.push_str("}\n");
        result
    }

    /// Format input parameters as Lattice function parameters.
    fn format_params(&self) -> String {
        match &self.input {
            InputDef::Map(map) => {
                // Sort for deterministic output
                let mut sorted: Vec<_> = map.iter().collect();
                sorted.sort_by_key(|(k, _)| *k);
                sorted
                    .iter()
                    .map(|(name, typ)| format!("{}: {}", name, typ))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
            InputDef::Single(s) => s.clone(),
        }
    }

    /// Format the return type.
    fn format_return_type(&self) -> String {
        match &self.output {
            OutputDef::TypeRef(name) => name.clone(),
            OutputDef::Inline(_) => format!("{}Output", to_pascal_case(&self.name)),
        }
    }

    /// Format provider configuration as Lattice syntax.
    fn format_provider(&self, provider: &ProviderDef) -> String {
        let mut result = String::from("    provider: {\n");

        if let Some(ref order) = provider.order {
            let items: Vec<_> = order.iter().map(|s| format!("\"{}\"", s)).collect();
            result.push_str(&format!("        order: [{}]\n", items.join(", ")));
        }
        if let Some(allow) = provider.allow_fallbacks {
            result.push_str(&format!("        allow_fallbacks: {}\n", allow));
        }
        if let Some(ref ignore) = provider.ignore {
            let items: Vec<_> = ignore.iter().map(|s| format!("\"{}\"", s)).collect();
            result.push_str(&format!("        ignore: [{}]\n", items.join(", ")));
        }
        if let Some(ref only) = provider.only {
            let items: Vec<_> = only.iter().map(|s| format!("\"{}\"", s)).collect();
            result.push_str(&format!("        only: [{}]\n", items.join(", ")));
        }

        result.push_str("    }\n");
        result
    }
}

/// Convert template variables from {var} to ${var}.
///
/// This converts markdown-style template variables to Lattice's template syntax.
/// Only converts {identifier} patterns, not arbitrary expressions.
/// Already-converted ${var} patterns are left unchanged.
fn convert_template_vars(text: &str) -> String {
    use regex::Regex;

    // Match {identifier} but NOT preceded by $
    // We use a two-step approach: first check each match isn't preceded by $
    let re = Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

    let mut result = String::new();
    let mut last_end = 0;

    for cap in re.captures_iter(text) {
        let full_match = cap.get(0).unwrap();
        let start = full_match.start();

        // Add text before this match
        result.push_str(&text[last_end..start]);

        // Check if preceded by $
        let preceded_by_dollar = start > 0 && text.as_bytes()[start - 1] == b'$';

        if preceded_by_dollar {
            // Already in ${var} form, keep as-is
            result.push_str(full_match.as_str());
        } else {
            // Convert {var} to ${var}
            result.push_str(&format!("${{{}}}", &cap[1]));
        }

        last_end = full_match.end();
    }

    // Add remaining text
    result.push_str(&text[last_end..]);

    result
}

/// Convert snake_case to PascalCase.
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter_basic() {
        let content = r#"---
name: test
model: gpt-4
---
This is the body."#;

        let (yaml, body) = split_frontmatter(content).unwrap();
        assert!(yaml.contains("name: test"));
        assert!(yaml.contains("model: gpt-4"));
        assert_eq!(body, "This is the body.");
    }

    #[test]
    fn test_split_frontmatter_empty_body() {
        let content = r#"---
name: test
model: gpt-4
---"#;

        let (yaml, body) = split_frontmatter(content).unwrap();
        assert!(yaml.contains("name: test"));
        assert_eq!(body, "");
    }

    #[test]
    fn test_split_frontmatter_multiline_body() {
        let content = r#"---
name: test
model: gpt-4
---
Line 1.
Line 2.

Line 4 after blank."#;

        let (_, body) = split_frontmatter(content).unwrap();
        assert!(body.contains("Line 1."));
        assert!(body.contains("Line 2."));
        assert!(body.contains("Line 4 after blank."));
    }

    #[test]
    fn test_missing_frontmatter() {
        let content = "Just some text without frontmatter";
        let result = split_frontmatter(content);
        assert!(matches!(result, Err(MarkdownError::MissingFrontmatter)));
    }

    #[test]
    fn test_unclosed_frontmatter() {
        let content = r#"---
name: test
model: gpt-4
No closing delimiter"#;

        let result = split_frontmatter(content);
        assert!(matches!(result, Err(MarkdownError::UnclosedFrontmatter)));
    }

    #[test]
    fn test_parse_simple_markdown() {
        let content = r#"---
name: analyze
model: gpt-4
input:
  text: String
output: Sentiment
---
Analyze this: {text}"#;

        let def = parse_markdown_llm(content).unwrap();
        assert_eq!(def.name, "analyze");
        assert_eq!(def.model, "gpt-4");
        assert!(def.prompt_body.contains("{text}"));

        match def.input {
            InputDef::Map(map) => {
                assert_eq!(map.get("text"), Some(&"String".to_string()));
            }
            _ => panic!("Expected Map input"),
        }

        match def.output {
            OutputDef::TypeRef(t) => assert_eq!(t, "Sentiment"),
            _ => panic!("Expected TypeRef output"),
        }
    }

    #[test]
    fn test_parse_with_optional_fields() {
        let content = r#"---
name: chat
model: claude-3-sonnet
base_url: https://api.anthropic.com/v1
api_key_env: ANTHROPIC_API_KEY
temperature: 0.7
max_tokens: 1000
input:
  message: String
output: String
---
{message}"#;

        let def = parse_markdown_llm(content).unwrap();
        assert_eq!(def.name, "chat");
        assert_eq!(def.model, "claude-3-sonnet");
        assert_eq!(
            def.base_url,
            Some("https://api.anthropic.com/v1".to_string())
        );
        assert_eq!(def.api_key_env, Some("ANTHROPIC_API_KEY".to_string()));
        assert_eq!(def.temperature, Some(0.7));
        assert_eq!(def.max_tokens, Some(1000));
    }

    #[test]
    fn test_parse_inline_output_type() {
        let content = r#"---
name: test
model: gpt-4
input:
  x: String
output:
  result: String
  score: Float
---
prompt"#;

        let def = parse_markdown_llm(content).unwrap();
        match def.output {
            OutputDef::Inline(map) => {
                assert!(matches!(map.get("result"), Some(FieldDef::Simple(s)) if s == "String"));
                assert!(matches!(map.get("score"), Some(FieldDef::Simple(s)) if s == "Float"));
            }
            _ => panic!("Expected inline output"),
        }
    }

    #[test]
    fn test_parse_inline_output_with_descriptions() {
        let content = r#"---
name: test
model: gpt-4
input:
  x: String
output:
  greeting:
    type: String
    description: "The greeting message"
  reason:
    type: String
    description: "Why this greeting was chosen"
---
prompt"#;

        let def = parse_markdown_llm(content).unwrap();
        match def.output {
            OutputDef::Inline(map) => {
                match map.get("greeting") {
                    Some(FieldDef::WithDescription { r#type, description }) => {
                        assert_eq!(r#type, "String");
                        assert_eq!(description, "The greeting message");
                    }
                    _ => panic!("Expected WithDescription for greeting"),
                }
                match map.get("reason") {
                    Some(FieldDef::WithDescription { r#type, description }) => {
                        assert_eq!(r#type, "String");
                        assert_eq!(description, "Why this greeting was chosen");
                    }
                    _ => panic!("Expected WithDescription for reason"),
                }
            }
            _ => panic!("Expected inline output"),
        }
    }

    #[test]
    fn test_parse_with_provider_config() {
        let content = r#"---
name: routed
model: gpt-4
input:
  text: String
output: String
provider:
  order:
    - openai
    - azure
  allow_fallbacks: true
---
{text}"#;

        let def = parse_markdown_llm(content).unwrap();
        let provider = def.provider.unwrap();
        assert_eq!(provider.order, Some(vec!["openai".to_string(), "azure".to_string()]));
        assert_eq!(provider.allow_fallbacks, Some(true));
    }

    #[test]
    fn test_parse_with_use_config() {
        let content = r#"---
name: with_config
model: gpt-4
use: my_openai_config
input:
  text: String
output: String
---
{text}"#;

        let def = parse_markdown_llm(content).unwrap();
        assert_eq!(def.use_config, Some("my_openai_config".to_string()));
    }

    #[test]
    fn test_parse_single_input() {
        let content = r#"---
name: simple
model: gpt-4
input: "text: String"
output: String
---
{text}"#;

        let def = parse_markdown_llm(content).unwrap();
        match def.input {
            InputDef::Single(s) => assert_eq!(s, "text: String"),
            _ => panic!("Expected Single input"),
        }
    }

    #[test]
    fn test_parse_missing_required_field() {
        // Missing 'model' field
        let content = r#"---
name: incomplete
input:
  text: String
output: String
---
prompt"#;

        let result = parse_markdown_llm(content);
        assert!(result.is_err());
    }

    // =========================================================================
    // Transpilation tests
    // =========================================================================

    #[test]
    fn test_transpile_simple() {
        let content = r#"---
name: greet
model: gpt-4
input:
  name: String
output: String
---
Say hello to {name}"#;

        let def = parse_markdown_llm(content).unwrap();
        let source = def.to_lattice_source();

        assert!(source.contains("def greet(name: String) -> String {"));
        assert!(source.contains("model: \"gpt-4\""));
        assert!(source.contains("${name}"));
        assert!(source.contains("prompt: \"\"\""));
    }

    #[test]
    fn test_transpile_with_all_options() {
        let content = r#"---
name: analyze
model: gpt-4o-mini
base_url: https://api.openai.com/v1
api_key_env: OPENAI_KEY
temperature: 0.5
max_tokens: 500
input:
  text: String
output: Sentiment
---
Analyze: {text}"#;

        let def = parse_markdown_llm(content).unwrap();
        let source = def.to_lattice_source();

        assert!(source.contains("def analyze(text: String) -> Sentiment {"));
        assert!(source.contains("base_url: \"https://api.openai.com/v1\""));
        assert!(source.contains("model: \"gpt-4o-mini\""));
        assert!(source.contains("api_key_env: \"OPENAI_KEY\""));
        assert!(source.contains("temperature: 0.5"));
        assert!(source.contains("max_tokens: 500"));
    }

    #[test]
    fn test_transpile_inline_output_type() {
        let content = r#"---
name: analyze_sentiment
model: gpt-4
input:
  text: String
output:
  sentiment: String
  confidence: Float
---
Analyze: {text}"#;

        let def = parse_markdown_llm(content).unwrap();
        let source = def.to_lattice_source();

        // Should generate inline type with commas between fields
        assert!(source.contains("type AnalyzeSentimentOutput {"));
        // First field (alphabetically) should have comma, second should not
        assert!(source.contains("confidence: Float,"));
        assert!(source.contains("sentiment: String\n"));

        // Function should use the generated type
        assert!(source.contains("-> AnalyzeSentimentOutput {"));
    }

    #[test]
    fn test_transpile_inline_output_with_descriptions() {
        let content = r#"---
name: greet
model: gpt-4
input:
  name: String
output:
  greeting:
    type: String
    description: "The greeting message"
  reason:
    type: String
    description: "Why this greeting was chosen"
---
Greet {name}"#;

        let def = parse_markdown_llm(content).unwrap();
        let source = def.to_lattice_source();

        // Should generate type with descriptions
        assert!(source.contains("type GreetOutput {"));
        assert!(source.contains(r#"greeting: String @"The greeting message","#));
        assert!(source.contains(r#"reason: String @"Why this greeting was chosen""#));
    }

    #[test]
    fn test_transpile_multiple_inputs() {
        let content = r#"---
name: translate
model: gpt-4
input:
  text: String
  target: String
output: String
---
Translate {text} to {target}"#;

        let def = parse_markdown_llm(content).unwrap();
        let source = def.to_lattice_source();

        // Both params should be in signature (sorted alphabetically)
        assert!(source.contains("target: String"));
        assert!(source.contains("text: String"));
        // Variables should be converted
        assert!(source.contains("${text}"));
        assert!(source.contains("${target}"));
    }

    #[test]
    fn test_transpile_with_use_config() {
        let content = r#"---
name: query
model: gpt-4
use: my_config
input:
  q: String
output: String
---
{q}"#;

        let def = parse_markdown_llm(content).unwrap();
        let source = def.to_lattice_source();

        assert!(source.contains("use: my_config"));
    }

    #[test]
    fn test_convert_template_vars() {
        assert_eq!(convert_template_vars("Hello {name}!"), "Hello ${name}!");
        assert_eq!(
            convert_template_vars("{a} and {b}"),
            "${a} and ${b}"
        );
        // Should not convert already-converted vars
        assert_eq!(
            convert_template_vars("${already_done}"),
            "${already_done}"
        );
        // Should not convert non-identifiers
        assert_eq!(convert_template_vars("{123}"), "{123}");
        assert_eq!(convert_template_vars("{a,b}"), "{a,b}");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("analyze_sentiment"), "AnalyzeSentiment");
        assert_eq!(to_pascal_case("greet"), "Greet");
        assert_eq!(to_pascal_case("get_user_info"), "GetUserInfo");
        assert_eq!(to_pascal_case("a_b_c"), "ABC");
    }
}
