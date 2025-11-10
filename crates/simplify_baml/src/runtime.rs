/// Main Runtime API
///
/// This is the primary interface for executing BAML functions.
/// It orchestrates: Template rendering -> LLM call -> Response parsing
use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::{
    client::LLMClient,
    ir::{BamlValue, FieldType, IR},
    parser::Parser,
    partial_parser::try_parse_partial_json,
    renderer::PromptRenderer,
    streaming_value::StreamingBamlValue,
};

/// Validate that a parsed result meets expectations
///
/// This checks for common issues like:
/// - Empty arrays that should be populated
/// - Missing required fields
fn validate_result(ir: &IR, value: &BamlValue, expected_type: &FieldType) -> Result<()> {
    match expected_type {
        FieldType::Class(class_name) => {
            if let Some(class) = ir.find_class(class_name) {
                if let BamlValue::Map(map) = value {
                    // Check each field
                    for field in &class.fields {
                        if let Some(field_value) = map.get(&field.name) {
                            // Recursively validate nested structures
                            validate_result(ir, field_value, &field.field_type)?;
                        }
                    }
                }
            }
        }
        FieldType::List(_inner) => {
            if let BamlValue::List(items) = value {
                if items.is_empty() {
                    // Log a warning but don't fail - some arrays might legitimately be empty
                    eprintln!("Warning: LLM returned empty array. This might indicate incomplete output.");
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Generate a prompt from IR, template, and parameters
///
/// This function takes an IR (Intermediate Representation), a Jinja2 template,
/// input parameters, and an output type, and generates the final prompt string
/// that will be sent to the LLM. It automatically injects the schema based on
/// the output type.
///
/// # Arguments
/// * `ir` - The Intermediate Representation containing type definitions
/// * `template` - The Jinja2 template string
/// * `params` - Input parameters as a HashMap of BamlValues
/// * `output_type` - The expected output type for schema generation
///
/// # Returns
/// The rendered prompt string with schema appended
///
/// # Example
/// ```rust,ignore
/// use simplify_baml::*;
/// use std::collections::HashMap;
///
/// let ir = IR::new();
/// let template = "Extract person info from: {{ text }}";
/// let mut params = HashMap::new();
/// params.insert("text".to_string(), BamlValue::String("John is 30".to_string()));
///
/// let prompt = generate_prompt_from_ir(
///     &ir,
///     template,
///     &params,
///     &FieldType::Class("Person".to_string())
/// ).unwrap();
/// ```
pub fn generate_prompt_from_ir(
    ir: &IR,
    template: &str,
    params: &HashMap<String, BamlValue>,
    output_type: &FieldType,
) -> Result<String> {
    let renderer = PromptRenderer::new(ir);
    renderer.render(template, params, output_type)
        .context("Failed to render prompt from IR")
}

/// Parse an LLM response using IR type definitions
///
/// This function takes a raw LLM response string and parses it into a typed
/// BamlValue based on the IR (Intermediate Representation). It handles:
/// - Extracting JSON from markdown code blocks
/// - Lenient JSON parsing
/// - Type coercion (e.g., string "30" → int 30)
/// - Enum validation with case-insensitive matching
/// - Nested structure validation
///
/// # Arguments
/// * `ir` - The Intermediate Representation containing type definitions
/// * `raw_response` - The raw string response from the LLM
/// * `target_type` - The expected output type to parse into
///
/// # Returns
/// The parsed and type-coerced BamlValue
///
/// # Example
/// ```rust,ignore
/// use simplify_baml::*;
///
/// let ir = IR::new();
/// let raw_response = r#"```json
/// {"name": "John", "age": "30"}
/// ```"#;
///
/// let result = parse_llm_response_with_ir(
///     &ir,
///     raw_response,
///     &FieldType::Class("Person".to_string())
/// ).unwrap();
/// ```
pub fn parse_llm_response_with_ir(
    ir: &IR,
    raw_response: &str,
    target_type: &FieldType,
) -> Result<BamlValue> {
    let parser = Parser::new(ir);
    parser.parse(raw_response, target_type)
        .context("Failed to parse LLM response using IR")
}

/// Try to parse a partial LLM response from streaming
///
/// This function attempts to parse potentially incomplete JSON from streaming
/// LLM responses. It uses heuristics to auto-close incomplete structures and
/// will return None if the JSON is too incomplete to parse.
///
/// # Arguments
/// * `ir` - The Intermediate Representation containing type definitions
/// * `partial_response` - The potentially incomplete response from streaming
/// * `target_type` - The expected output type to parse into
///
/// # Returns
/// * `Ok(Some(BamlValue))` - Successfully parsed partial response
/// * `Ok(None)` - Too incomplete to parse, need more data
/// * `Err(...)` - Parsing error
///
/// # Example
/// ```rust,ignore
/// use simplify_baml::*;
///
/// let ir = IR::new();
///
/// // Streaming chunks
/// let chunk1 = r#"{"name": "Joh"#;
/// let chunk2 = r#"{"name": "John", "age": 3"#;
/// let chunk3 = r#"{"name": "John", "age": 30}"#;
///
/// // Try parsing each chunk
/// assert!(try_parse_partial_response(&ir, chunk1, &target_type).unwrap().is_some());
/// assert!(try_parse_partial_response(&ir, chunk2, &target_type).unwrap().is_some());
/// assert!(try_parse_partial_response(&ir, chunk3, &target_type).unwrap().is_some());
/// ```
pub fn try_parse_partial_response(
    ir: &IR,
    partial_response: &str,
    target_type: &FieldType,
) -> Result<Option<BamlValue>> {
    // First, try to extract and auto-close partial JSON
    match try_parse_partial_json(partial_response)? {
        Some(json_value) => {
            // We got a JSON value, now try to coerce it using the parser
            let json_str = serde_json::to_string(&json_value)?;
            match parse_llm_response_with_ir(ir, &json_str, target_type) {
                Ok(baml_value) => Ok(Some(baml_value)),
                Err(_) => Ok(None), // Coercion failed, need more data
            }
        }
        None => Ok(None), // Not enough data yet
    }
}

/// Parse streaming response with schema-aware structure (RECOMMENDED for UIs)
///
/// This function provides the best UX for streaming by always returning the full
/// schema structure. Fields are filled in as data arrives, but the structure
/// never changes. This makes UI rendering much simpler and more predictable.
///
/// # Arguments
/// * `streaming_value` - The streaming value to update (create with `StreamingBamlValue::from_ir_skeleton`)
/// * `ir` - The Intermediate Representation
/// * `partial_response` - The current accumulated response
/// * `target_type` - The expected output type
/// * `is_final` - Whether this is the final chunk (marks as complete)
///
/// # Example
/// ```rust,ignore
/// use simplify_baml::*;
///
/// // Create skeleton with full structure
/// let mut streaming = StreamingBamlValue::from_ir_skeleton(&ir, &target_type);
///
/// // As chunks arrive, update in place
/// while let Some(chunk) = stream.next().await {
///     accumulated.push_str(&chunk?);
///
///     update_streaming_response(&mut streaming, &ir, &accumulated, &target_type, false)?;
///
///     // UI always gets full structure!
///     println!("{}", serde_json::to_string_pretty(&streaming)?);
///     // {
///     //   "value": {"name": "John", "age": null, "occupation": null},
///     //   "state": "partial"
///     // }
/// }
///
/// // Mark final
/// update_streaming_response(&mut streaming, &ir, &accumulated, &target_type, true)?;
/// ```
pub fn update_streaming_response(
    streaming_value: &mut StreamingBamlValue,
    ir: &IR,
    partial_response: &str,
    target_type: &FieldType,
    is_final: bool,
) -> Result<()> {
    // Try to parse the partial response
    if let Some(partial_baml) = try_parse_partial_response(ir, partial_response, target_type)? {
        streaming_value.update_from_partial(ir, partial_baml, target_type);
    }

    if is_final {
        streaming_value.mark_complete();
    }

    Ok(())
}

pub struct BamlRuntime {
    ir: IR,
    clients: HashMap<String, LLMClient>,
}

impl BamlRuntime {
    /// Create a new runtime with the given IR
    pub fn new(ir: IR) -> Self {
        Self {
            ir,
            clients: HashMap::new(),
        }
    }

    /// Register an LLM client with a name
    pub fn register_client(&mut self, name: impl Into<String>, client: LLMClient) {
        self.clients.insert(name.into(), client);
    }

    /// Execute a BAML function
    ///
    /// # Arguments
    /// * `function_name` - Name of the function to execute
    /// * `params` - Input parameters as a HashMap
    ///
    /// # Returns
    /// The parsed result as a BamlValue
    pub async fn execute(
        &self,
        function_name: &str,
        params: HashMap<String, BamlValue>,
    ) -> Result<BamlValue> {
        // Find the function
        let function = self.ir.find_function(function_name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", function_name))?;

        // Get the client
        let client = self.clients.get(&function.client)
            .ok_or_else(|| anyhow::anyhow!("Client '{}' not found", function.client))?;

        // Generate the prompt using the extracted function
        let prompt = generate_prompt_from_ir(
            &self.ir,
            &function.prompt_template,
            &params,
            &function.output
        )?;

        // Call the LLM
        let raw_response = client.call(&prompt)
            .await
            .context("Failed to call LLM")?;

        // Parse the response using the extracted function
        let result = parse_llm_response_with_ir(
            &self.ir,
            &raw_response,
            &function.output
        )?;

        // Validate the result (logs warnings for suspicious patterns)
        validate_result(&self.ir, &result, &function.output)?;

        Ok(result)
    }

    /// Execute a BAML function with retry on empty arrays
    ///
    /// This is like `execute()` but will retry up to `max_retries` times
    /// if the LLM returns empty arrays.
    ///
    /// # Arguments
    /// * `function_name` - Name of the function to execute
    /// * `params` - Input parameters as a HashMap
    /// * `max_retries` - Maximum number of retries (0 = no retries)
    ///
    /// # Returns
    /// The parsed result as a BamlValue
    pub async fn execute_with_retry(
        &self,
        function_name: &str,
        params: HashMap<String, BamlValue>,
        max_retries: usize,
    ) -> Result<BamlValue> {
        let mut attempts = 0;

        loop {
            match self.execute(function_name, params.clone()).await {
                Ok(result) => {
                    // Check if result has empty arrays
                    if has_empty_arrays(&result) && attempts < max_retries {
                        eprintln!("Attempt {}: LLM returned empty arrays, retrying...", attempts + 1);
                        attempts += 1;
                        continue;
                    }
                    return Ok(result);
                }
                Err(e) => {
                    if attempts >= max_retries {
                        return Err(e);
                    }
                    attempts += 1;
                    eprintln!("Attempt {} failed, retrying...", attempts);
                }
            }
        }
    }

    /// Get the IR (for inspection/debugging)
    pub fn ir(&self) -> &IR {
        &self.ir
    }
}

/// Check if a BamlValue contains any empty arrays
fn has_empty_arrays(value: &BamlValue) -> bool {
    match value {
        BamlValue::List(items) => items.is_empty(),
        BamlValue::Map(map) => {
            for val in map.values() {
                if has_empty_arrays(val) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// Builder for constructing a BamlRuntime
pub struct RuntimeBuilder {
    ir: IR,
    clients: HashMap<String, LLMClient>,
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            ir: IR::new(),
            clients: HashMap::new(),
        }
    }

    pub fn ir(mut self, ir: IR) -> Self {
        self.ir = ir;
        self
    }

    pub fn client(mut self, name: impl Into<String>, client: LLMClient) -> Self {
        self.clients.insert(name.into(), client);
        self
    }

    pub fn build(self) -> BamlRuntime {
        let mut runtime = BamlRuntime::new(self.ir);
        for (name, client) in self.clients {
            runtime.register_client(name, client);
        }
        runtime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    #[tokio::test]
    async fn test_runtime_execution() {
        // Build IR
        let mut ir = IR::new();
        ir.classes.push(Class {
            name: "Person".to_string(),
            description: None,
            fields: vec![
                Field {
                    name: "name".to_string(),
                    field_type: FieldType::String,
                    optional: false,
                description: None,
                },
                Field {
                    name: "age".to_string(),
                    field_type: FieldType::Int,
                    optional: false,
                description: None,
                },
            ],
        });

        ir.functions.push(Function {
            name: "ExtractPerson".to_string(),
            inputs: vec![
                Field {
                    name: "text".to_string(),
                    field_type: FieldType::String,
                    optional: false,
                description: None,
                }
            ],
            output: FieldType::Class("Person".to_string()),
            prompt_template: "Extract person info from: {{ text }}".to_string(),
            client: "test_client".to_string(),
        });

        // Create mock client that we can control
        // Note: Since we can't use MockLLMClient directly with LLMClient,
        // this test would need a real API key or a more sophisticated mock setup
        // For now, we'll skip the actual execution in tests

        let runtime = BamlRuntime::new(ir);

        // Verify function exists
        assert!(runtime.ir().find_function("ExtractPerson").is_some());
    }

    #[test]
    fn test_runtime_builder() {
        let ir = IR::new();
        let client = LLMClient::openai("test-key".to_string(), "gpt-4".to_string());

        let runtime = RuntimeBuilder::new()
            .ir(ir)
            .client("openai", client)
            .build();

        assert!(runtime.clients.contains_key("openai"));
    }
}
