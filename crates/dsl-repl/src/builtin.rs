use crate::sql::SQLExecutor;
use crate::types::TypeRegistry;
use crate::value::Value;
use anyhow::Result;
use simplify_baml::*;
use std::collections::HashMap;
use std::env;

pub struct BuiltinFunctions {
    runtime: Option<BamlRuntime>,
    llm_client: Option<LLMClient>,
    api_key: Option<String>,
    sql_executor: Option<SQLExecutor>,
    pub last_prompt: Option<String>,
}

impl BuiltinFunctions {
    pub fn new() -> Result<Self> {
        // Try to initialize BAML runtime and direct LLM client
        let (runtime, llm_client, api_key) = match env::var("OPENAI_API_KEY") {
            Ok(api_key) => {
                // Build IR with structured types
                let mut ir = IR::new();

                // Define a Person class for structured output
                ir.classes.push(Class {
                    name: "Person".to_string(),
                    description: Some("Information about a person".to_string()),
                    fields: vec![
                        Field {
                            name: "name".to_string(),
                            field_type: FieldType::String,
                            optional: false,
                            description: Some("Full name of the person".to_string()),
                        },
                        Field {
                            name: "age".to_string(),
                            field_type: FieldType::Int,
                            optional: true,
                            description: Some("Age in years".to_string()),
                        },
                        Field {
                            name: "occupation".to_string(),
                            field_type: FieldType::String,
                            optional: true,
                            description: Some("Job title or profession".to_string()),
                        },
                    ],
                });

                // Define the ExtractPerson function with structured output
                ir.functions.push(Function {
                    name: "ExtractPerson".to_string(),
                    inputs: vec![Field {
                        name: "text".to_string(),
                        field_type: FieldType::String,
                        optional: false,
                        description: Some("Text containing person information".to_string()),
                    }],
                    output: FieldType::Class("Person".to_string()),
                    prompt_template: r#"Extract the person's information from the following text:

{{ text }}

Please extract: name, age (if mentioned), and occupation (if mentioned)."#
                        .to_string(),
                    client: "openai".to_string(),
                });

                // Create OpenAI client
                let client = LLMClient::openai(api_key.clone(), "gpt-3.5-turbo".to_string());

                // Also create a direct client for simple string responses
                let direct_client = LLMClient::openai(api_key.clone(), "gpt-3.5-turbo".to_string());

                // Build runtime
                let runtime = RuntimeBuilder::new()
                    .ir(ir)
                    .client("openai", client)
                    .build();

                (Some(runtime), Some(direct_client), Some(api_key))
            }
            Err(_) => {
                // No API key, LLM functions won't work
                (None, None, None)
            }
        };

        // Initialize SQL executor
        let sql_executor = SQLExecutor::new().ok();

        Ok(Self {
            runtime,
            llm_client,
            api_key,
            sql_executor,
            last_prompt: None,
        })
    }

    /// Rebuild the BAML runtime with updated types from the TypeRegistry
    pub fn rebuild_runtime(&mut self, type_registry: &TypeRegistry) -> Result<()> {
        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => return Ok(()), // No API key, nothing to rebuild
        };

        // Build IR with all registered types
        let mut ir = IR::new();

        // Add all user-defined classes and enums
        let (classes, enums) = type_registry.to_ir_types();
        ir.classes.extend(classes);
        ir.enums.extend(enums);

        // Add the built-in ExtractPerson for backwards compatibility
        if !type_registry.has_type("Person") {
            ir.classes.push(Class {
                name: "Person".to_string(),
                description: Some("Information about a person".to_string()),
                fields: vec![
                    Field {
                        name: "name".to_string(),
                        field_type: FieldType::String,
                        optional: false,
                        description: Some("Full name of the person".to_string()),
                    },
                    Field {
                        name: "age".to_string(),
                        field_type: FieldType::Int,
                        optional: true,
                        description: Some("Age in years".to_string()),
                    },
                    Field {
                        name: "occupation".to_string(),
                        field_type: FieldType::String,
                        optional: true,
                        description: Some("Job title or profession".to_string()),
                    },
                ],
            });

            ir.functions.push(Function {
                name: "ExtractPerson".to_string(),
                inputs: vec![Field {
                    name: "text".to_string(),
                    field_type: FieldType::String,
                    optional: false,
                    description: Some("Text containing person information".to_string()),
                }],
                output: FieldType::Class("Person".to_string()),
                prompt_template: r#"Extract the person's information from the following text:

{{ text }}

Please extract: name, age (if mentioned), and occupation (if mentioned)."#
                    .to_string(),
                client: "openai".to_string(),
            });
        }

        // Create new client
        let client = LLMClient::openai(api_key, "gpt-3.5-turbo".to_string());

        // Rebuild runtime
        self.runtime = Some(
            RuntimeBuilder::new()
                .ir(ir)
                .client("openai", client)
                .build(),
        );

        Ok(())
    }

    pub async fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        match name.to_lowercase().as_str() {
            "ask" => self.ask(args).await,
            "extractperson" => self.extract_person(args).await,
            "extractas" => self.extract_as(args).await,
            "sql" => self.sql(args),
            "length" => self.length(args),
            "upper" => self.upper(args),
            "lower" => self.lower(args),
            "join" => self.join(args),
            _ => Err(anyhow::anyhow!("Unknown function: {}", name)),
        }
    }

    /// Call Ask with custom model, base_url, and api_key_env configuration
    pub async fn ask_with_config(
        &mut self,
        prompt: String,
        model: Option<String>,
        base_url: Option<String>,
        api_key_env: Option<String>,
    ) -> Result<Value> {
        self.last_prompt = Some(prompt.clone());

        let client = self.create_client(model, base_url, api_key_env)?;
        let response = client.call(&prompt).await.map_err(|e| {
            let mut error_msg = format!("Failed to call LLM: {}", e);
            let mut source = e.source();
            while let Some(err) = source {
                error_msg.push_str(&format!("\n  Caused by: {}", err));
                source = err.source();
            }
            anyhow::anyhow!(error_msg)
        })?;

        Ok(Value::String(response))
    }

    /// Call ExtractAs with custom model, base_url, and api_key_env configuration
    pub async fn extract_as_with_config(
        &mut self,
        text: String,
        type_identifier: String,
        model: Option<String>,
        base_url: Option<String>,
        api_key_env: Option<String>,
    ) -> Result<Value> {
        let runtime = self.runtime.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "OPENAI_API_KEY not set. Please set it with: export OPENAI_API_KEY=sk-..."
            )
        })?;

        let output_type = self.parse_type_identifier(&type_identifier, runtime.ir())?;
        let mut params = std::collections::HashMap::new();
        params.insert("text".to_string(), BamlValue::String(text.clone()));

        let prompt_template = format!(
            "Extract {} information from the following text:\n\n{{{{ text }}}}\n\nPlease extract all relevant fields.",
            type_identifier
        );

        let prompt = generate_prompt_from_ir(runtime.ir(), &prompt_template, &params, &output_type)
            .map_err(|e| anyhow::anyhow!("Failed to generate prompt: {}", e))?;

        let client = self.create_client(model, base_url, api_key_env)?;
        let raw_response = client
            .call(&prompt)
            .await
            .map_err(|e| anyhow::anyhow!("LLM call failed: {}", e))?;

        let result = parse_llm_response_with_ir(runtime.ir(), &raw_response, &output_type)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse LLM response:\n  Error: {}\n  Raw response: {}",
                    e,
                    raw_response
                )
            })?;

        Ok(self.baml_value_to_value(result))
    }

    /// Create an LLM client with the specified model, base_url, and api_key_env
    fn create_client(
        &self,
        model: Option<String>,
        base_url: Option<String>,
        api_key_env: Option<String>,
    ) -> Result<LLMClient> {
        // Determine which environment variable to use for the API key
        let env_var_name = api_key_env.as_deref().unwrap_or("OPENAI_API_KEY");

        // Get the API key from the specified environment variable
        let api_key = env::var(env_var_name).map_err(|_| {
            anyhow::anyhow!(
                "{} not set. Please set it with: export {}=your-api-key",
                env_var_name,
                env_var_name
            )
        })?;

        let model = model.unwrap_or_else(|| "gpt-4o-mini".to_string());

        let client = if let Some(base_url) = base_url {
            // Use custom base URL (for OpenRouter or other providers)
            LLMClient::custom(api_key, base_url, model)
        } else {
            // Default to OpenAI
            LLMClient::openai(api_key, model)
        };

        Ok(client)
    }

    async fn ask(&mut self, args: Vec<Value>) -> Result<Value> {
        if args.is_empty() {
            return Err(anyhow::anyhow!(
                "Ask() requires at least 1 argument (prompt)"
            ));
        }

        let prompt = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(anyhow::anyhow!("Ask() requires a string prompt")),
        };

        // Store the prompt for debugging with :debug command
        self.last_prompt = Some(prompt.clone());

        // Check if LLM client is available
        let client = self.llm_client.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "OPENAI_API_KEY not set. Please set it with: export OPENAI_API_KEY=sk-..."
            )
        })?;

        // Call the LLM directly (bypassing BAML parser for simple string responses)
        let response = client.call(&prompt).await.map_err(|e| {
            // Show the full error chain for debugging
            let mut error_msg = format!("Failed to call LLM: {}", e);
            let mut source = e.source();
            while let Some(err) = source {
                error_msg.push_str(&format!("\n  Caused by: {}", err));
                source = err.source();
            }
            anyhow::anyhow!(error_msg)
        })?;

        Ok(Value::String(response))
    }

    async fn extract_person(&self, args: Vec<Value>) -> Result<Value> {
        if args.is_empty() {
            return Err(anyhow::anyhow!(
                "ExtractPerson() requires at least 1 argument (text)"
            ));
        }

        let text = match &args[0] {
            Value::String(s) => s.clone(),
            _ => {
                return Err(anyhow::anyhow!(
                    "ExtractPerson() requires a string argument"
                ))
            }
        };

        // Check if runtime is available
        let runtime = self.runtime.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "OPENAI_API_KEY not set. Please set it with: export OPENAI_API_KEY=sk-..."
            )
        })?;

        // Prepare parameters
        let mut params = HashMap::new();
        params.insert("text".to_string(), BamlValue::String(text));

        // For debugging, let's see what the LLM actually returns
        // First, get the prompt that will be sent
        let prompt = generate_prompt_from_ir(
            runtime.ir(),
            &runtime
                .ir()
                .find_function("ExtractPerson")
                .unwrap()
                .prompt_template,
            &params,
            &FieldType::Class("Person".to_string()),
        )
        .map_err(|e| anyhow::anyhow!("Failed to generate prompt: {}", e))?;

        // Get the client directly to see the raw response
        let raw_response = self
            .llm_client
            .as_ref()
            .unwrap()
            .call(&prompt)
            .await
            .map_err(|e| anyhow::anyhow!("LLM call failed: {}", e))?;

        // Try to parse using the IR
        let result = parse_llm_response_with_ir(
            runtime.ir(),
            &raw_response,
            &FieldType::Class("Person".to_string()),
        )
        .map_err(|e| {
            // Show both the error and the raw response for debugging
            anyhow::anyhow!(
                "Failed to parse LLM response:\n  Error: {}\n  Raw response: {}",
                e,
                raw_response
            )
        })?;

        // Convert BamlValue to our Value type
        Ok(self.baml_value_to_value(result))
    }

    async fn extract_as(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() < 2 {
            return Err(anyhow::anyhow!(
                "ExtractAs() requires 2 arguments: ExtractAs(text, TypeName)"
            ));
        }

        // First argument: text to extract from
        let text = match &args[0] {
            Value::String(s) => s.clone(),
            _ => {
                return Err(anyhow::anyhow!(
                    "ExtractAs() first argument must be a string"
                ))
            }
        };

        // Second argument: type identifier (could be a type name like "Person" or a type spec like "[string]")
        let type_identifier = match &args[1] {
            Value::String(s) => s.clone(),
            _ => {
                return Err(anyhow::anyhow!(
                    "ExtractAs() second argument must be a type identifier (string)"
                ))
            }
        };

        // Check if runtime is available
        let runtime = self.runtime.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "OPENAI_API_KEY not set. Please set it with: export OPENAI_API_KEY=sk-..."
            )
        })?;

        // Parse the type identifier to determine the FieldType
        let output_type = self.parse_type_identifier(&type_identifier, runtime.ir())?;

        // Prepare parameters
        let mut params = HashMap::new();
        params.insert("text".to_string(), BamlValue::String(text.clone()));

        // Create a dynamic prompt template
        let prompt_template = format!(
            "Extract {} information from the following text:\n\n{{{{ text }}}}\n\nPlease extract all relevant fields.",
            type_identifier
        );

        // Generate the prompt
        let prompt = generate_prompt_from_ir(runtime.ir(), &prompt_template, &params, &output_type)
            .map_err(|e| anyhow::anyhow!("Failed to generate prompt: {}", e))?;

        // Call the LLM
        let raw_response = self
            .llm_client
            .as_ref()
            .unwrap()
            .call(&prompt)
            .await
            .map_err(|e| anyhow::anyhow!("LLM call failed: {}", e))?;

        // Parse using the IR
        let result = parse_llm_response_with_ir(runtime.ir(), &raw_response, &output_type)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse LLM response:\n  Error: {}\n  Raw response: {}",
                    e,
                    raw_response
                )
            })?;

        // Convert BamlValue to our Value type
        Ok(self.baml_value_to_value(result))
    }

    // Helper function to convert BamlValue to our Value type
    fn baml_value_to_value(&self, baml_value: BamlValue) -> Value {
        match baml_value {
            BamlValue::String(s) => Value::String(s),
            BamlValue::Int(i) => Value::Int(i),
            BamlValue::Float(f) => Value::Float(f),
            BamlValue::Bool(b) => Value::Bool(b),
            BamlValue::Null => Value::Null,
            BamlValue::List(items) => Value::List(
                items
                    .into_iter()
                    .map(|v| self.baml_value_to_value(v))
                    .collect(),
            ),
            BamlValue::Map(map) => Value::Map(
                map.into_iter()
                    .map(|(k, v)| (k, self.baml_value_to_value(v)))
                    .collect(),
            ),
        }
    }

    // Parse a type identifier string (like "Person", "[string]", etc.) into a FieldType
    fn parse_type_identifier(&self, type_id: &str, ir: &IR) -> Result<FieldType> {
        // Check if it's a list type like "[string]"
        if type_id.starts_with('[') && type_id.ends_with(']') {
            let inner_type = type_id[1..type_id.len()-1].trim();
            let inner_field_type = self.parse_type_identifier(inner_type, ir)?;
            return Ok(FieldType::List(Box::new(inner_field_type)));
        }

        // Check for primitive types
        match type_id.to_lowercase().as_str() {
            "string" => return Ok(FieldType::String),
            "int" => return Ok(FieldType::Int),
            "float" => return Ok(FieldType::Float),
            "bool" => return Ok(FieldType::Bool),
            _ => {}
        }

        // Check if it's a registered class or enum
        if ir.find_class(type_id).is_some() {
            return Ok(FieldType::Class(type_id.to_string()));
        }

        if ir.find_enum(type_id).is_some() {
            return Ok(FieldType::Enum(type_id.to_string()));
        }

        Err(anyhow::anyhow!(
            "Type '{}' not found. Use :types to see registered types.",
            type_id
        ))
    }

    fn length(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Length() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::String(s) => Ok(Value::Int(s.len() as i64)),
            Value::List(items) => Ok(Value::Int(items.len() as i64)),
            _ => Err(anyhow::anyhow!(
                "Length() requires a string or list, got {}",
                args[0].type_name()
            )),
        }
    }

    fn upper(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Upper() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            _ => Err(anyhow::anyhow!(
                "Upper() requires a string, got {}",
                args[0].type_name()
            )),
        }
    }

    fn lower(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Lower() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::String(s) => Ok(Value::String(s.to_lowercase())),
            _ => Err(anyhow::anyhow!(
                "Lower() requires a string, got {}",
                args[0].type_name()
            )),
        }
    }

    fn join(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "Join() requires exactly 2 arguments (list, separator)"
            ));
        }

        let items = match &args[0] {
            Value::List(items) => items,
            _ => {
                return Err(anyhow::anyhow!(
                    "Join() requires a list as first argument, got {}",
                    args[0].type_name()
                ))
            }
        };

        let separator = match &args[1] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "Join() requires a string separator as second argument, got {}",
                    args[1].type_name()
                ))
            }
        };

        let strings: Vec<String> = items
            .iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                Value::Int(n) => n.to_string(),
                Value::Float(f) => f.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => v.display(),
            })
            .collect();

        Ok(Value::String(strings.join(separator)))
    }

    fn sql(&mut self, args: Vec<Value>) -> Result<Value> {
        if args.is_empty() {
            return Err(anyhow::anyhow!(
                "SQL() requires at least 1 argument (query string)"
            ));
        }

        let query = match &args[0] {
            Value::String(s) => s.clone(),
            _ => {
                return Err(anyhow::anyhow!(
                    "SQL() first argument must be a query string"
                ))
            }
        };

        let executor = self
            .sql_executor
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("SQL executor failed to initialize"))?;

        // Register additional tables from arguments
        for (i, arg) in args.iter().skip(1).enumerate() {
            let table_name = format!("table{}", i + 1);
            executor
                .register_table(&table_name, arg)
                .map_err(|e| anyhow::anyhow!("Failed to register table: {}", e))?;
        }

        executor
            .execute(&query, &HashMap::new())
            .map_err(|e| anyhow::anyhow!("SQL execution failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_string() {
        let builtins = BuiltinFunctions::new().unwrap();
        let result = builtins
            .length(vec![Value::String("hello".to_string())])
            .unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_length_list() {
        let builtins = BuiltinFunctions::new().unwrap();
        let result = builtins
            .length(vec![Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])])
            .unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_upper() {
        let builtins = BuiltinFunctions::new().unwrap();
        let result = builtins
            .upper(vec![Value::String("hello".to_string())])
            .unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_lower() {
        let builtins = BuiltinFunctions::new().unwrap();
        let result = builtins
            .lower(vec![Value::String("HELLO".to_string())])
            .unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_join() {
        let builtins = BuiltinFunctions::new().unwrap();
        let result = builtins
            .join(vec![
                Value::List(vec![
                    Value::String("a".to_string()),
                    Value::String("b".to_string()),
                    Value::String("c".to_string()),
                ]),
                Value::String(", ".to_string()),
            ])
            .unwrap();
        assert_eq!(result, Value::String("a, b, c".to_string()));
    }
}
