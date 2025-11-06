use crate::eval::sql::SQLExecutor;
use crate::types::{TypeRegistry, Value};
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
            "rendermarkdown" => self.render_markdown(args),
            "sql" => self.sql(args),
            "length" => self.length(args),
            "upper" => self.upper(args),
            "lower" => self.lower(args),
            "join" => self.join(args),
            "par" => Ok(self.par(args)),
            "not" => self.not(args),
            "generatebarchart" => self.generate_bar_chart(args),
            "generatelinechart" => self.generate_line_chart(args),
            "generatepiechart" => self.generate_pie_chart(args),
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

        let output_type = Self::parse_type_identifier(&type_identifier, runtime.ir())?;
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

        Ok(Self::baml_value_to_value(result))
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
        Ok(Self::baml_value_to_value(result))
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
        let output_type = Self::parse_type_identifier(&type_identifier, runtime.ir())?;

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
        Ok(Self::baml_value_to_value(result))
    }

    // Helper function to convert BamlValue to our Value type
    fn baml_value_to_value(baml_value: BamlValue) -> Value {
        match baml_value {
            BamlValue::String(s) => Value::String(s),
            BamlValue::Int(i) => Value::Int(i),
            BamlValue::Float(f) => Value::Float(f),
            BamlValue::Bool(b) => Value::Bool(b),
            BamlValue::Null => Value::Null,
            BamlValue::List(items) => {
                Value::List(items.into_iter().map(Self::baml_value_to_value).collect())
            }
            BamlValue::Map(map) => Value::Map(
                map.into_iter()
                    .map(|(k, v)| (k, Self::baml_value_to_value(v)))
                    .collect(),
            ),
        }
    }

    // Parse a type identifier string (like "Person", "[string]", etc.) into a FieldType
    fn parse_type_identifier(type_id: &str, ir: &IR) -> Result<FieldType> {
        // Check if it's a list type like "[string]"
        if type_id.starts_with('[') && type_id.ends_with(']') {
            let inner_type = type_id[1..type_id.len() - 1].trim();
            let inner_field_type = Self::parse_type_identifier(inner_type, ir)?;
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

    fn render_markdown(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "RenderMarkdown() requires exactly 1 argument"
            ));
        }

        match &args[0] {
            Value::String(s) => Ok(Value::Markdown(s.clone())),
            Value::Markdown(s) => Ok(Value::Markdown(s.clone())),
            _ => Err(anyhow::anyhow!(
                "RenderMarkdown() requires a string, got {}",
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

    /// par() - Parallel execution function
    /// Takes multiple arguments and returns them as a list
    /// This replaces the old || operator syntax
    fn par(&self, args: Vec<Value>) -> Value {
        // Always return a list, even for single argument
        // This ensures destructuring works consistently
        Value::List(args)
    }

    /// not() - Logical NOT function
    /// Takes a boolean and returns its negation
    fn not(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("not() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(anyhow::anyhow!(
                "not() requires a boolean, got {}",
                args[0].type_name()
            )),
        }
    }

    /// generateBarChart() - Generate a bar chart image
    /// Takes a list of {label: String, value: Number} and returns an Image value
    fn generate_bar_chart(&self, args: Vec<Value>) -> Result<Value> {
        use plotters::prelude::*;

        if args.is_empty() {
            return Err(anyhow::anyhow!(
                "generateBarChart() requires at least 1 argument (data)"
            ));
        }

        // Parse data: expect list of maps with "label" and "value"
        let data = match &args[0] {
            Value::List(items) => {
                let mut chart_data = Vec::new();
                for item in items {
                    match item {
                        Value::Map(m) => {
                            let label = m.get("label")
                                .and_then(|v| match v {
                                    Value::String(s) => Some(s.clone()),
                                    _ => None,
                                })
                                .ok_or_else(|| anyhow::anyhow!("Each item must have a 'label' string field"))?;

                            let value = m.get("value")
                                .and_then(|v| match v {
                                    Value::Int(n) => Some(*n as f64),
                                    Value::Float(f) => Some(*f),
                                    _ => None,
                                })
                                .ok_or_else(|| anyhow::anyhow!("Each item must have a 'value' numeric field"))?;

                            chart_data.push((label, value));
                        }
                        _ => return Err(anyhow::anyhow!("generateBarChart() expects a list of maps")),
                    }
                }
                chart_data
            }
            _ => return Err(anyhow::anyhow!("generateBarChart() requires a list as first argument")),
        };

        if data.is_empty() {
            return Err(anyhow::anyhow!("generateBarChart() requires non-empty data"));
        }

        // Create temporary file
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let file_path = temp_dir.join(format!("chart_bar_{}.png", timestamp));

        // Generate chart
        let root = BitMapBackend::new(&file_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_value = data.iter().map(|(_, v)| *v).fold(0.0f64, f64::max);
        let y_max = (max_value * 1.2).max(1.0);

        let mut chart = ChartBuilder::on(&root)
            .caption("Bar Chart", ("sans-serif", 40))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                (0usize..data.len()).into_segmented(),
                0f64..y_max,
            )?;

        chart.configure_mesh()
            .x_labels(data.len())
            .x_label_formatter(&|x| {
                if let SegmentValue::CenterOf(idx) = x {
                    data.get(*idx).map(|(label, _)| label.clone()).unwrap_or_default()
                } else {
                    String::new()
                }
            })
            .draw()?;

        chart.draw_series(
            data.iter().enumerate().map(|(i, (_, value))| {
                let x = SegmentValue::CenterOf(i);
                let mut bar = Rectangle::new([(x.clone(), 0.0), (x, *value)], BLUE.filled());
                bar.set_margin(0, 0, 5, 5);
                bar
            })
        )?;

        root.present()?;

        Ok(Value::Image(file_path.to_string_lossy().to_string()))
    }

    /// generateLineChart() - Generate a line chart image
    /// Takes a list of numbers and returns an Image value
    fn generate_line_chart(&self, args: Vec<Value>) -> Result<Value> {
        use plotters::prelude::*;

        if args.is_empty() {
            return Err(anyhow::anyhow!(
                "generateLineChart() requires at least 1 argument (data)"
            ));
        }

        // Parse data: expect list of numbers
        let data = match &args[0] {
            Value::List(items) => {
                let mut chart_data = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    let value = match item {
                        Value::Int(n) => *n as f64,
                        Value::Float(f) => *f,
                        _ => return Err(anyhow::anyhow!("generateLineChart() expects a list of numbers")),
                    };
                    chart_data.push((i as f64, value));
                }
                chart_data
            }
            _ => return Err(anyhow::anyhow!("generateLineChart() requires a list as first argument")),
        };

        if data.is_empty() {
            return Err(anyhow::anyhow!("generateLineChart() requires non-empty data"));
        }

        // Create temporary file
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let file_path = temp_dir.join(format!("chart_line_{}.png", timestamp));

        // Generate chart
        let root = BitMapBackend::new(&file_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_value = data.iter().map(|(_, v)| *v).fold(f64::NEG_INFINITY, f64::max);
        let min_value = data.iter().map(|(_, v)| *v).fold(f64::INFINITY, f64::min);
        let y_range = max_value - min_value;
        let y_min = (min_value - y_range * 0.1).min(0.0);
        let y_max = (max_value + y_range * 0.1).max(1.0);

        let mut chart = ChartBuilder::on(&root)
            .caption("Line Chart", ("sans-serif", 40))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0f64..(data.len() - 1) as f64, y_min..y_max)?;

        chart.configure_mesh().draw()?;

        chart.draw_series(LineSeries::new(data.clone(), &BLUE))?;

        chart.draw_series(PointSeries::of_element(
            data,
            5,
            &BLUE,
            &|coord, size, style| {
                EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled())
            },
        ))?;

        root.present()?;

        Ok(Value::Image(file_path.to_string_lossy().to_string()))
    }

    /// generatePieChart() - Generate a pie chart image
    /// Takes a list of {label: String, value: Number} and returns an Image value
    fn generate_pie_chart(&self, args: Vec<Value>) -> Result<Value> {
        use plotters::prelude::*;
        use std::f64::consts::PI;

        if args.is_empty() {
            return Err(anyhow::anyhow!(
                "generatePieChart() requires at least 1 argument (data)"
            ));
        }

        // Parse data: expect list of maps with "label" and "value"
        let data = match &args[0] {
            Value::List(items) => {
                let mut chart_data = Vec::new();
                for item in items {
                    match item {
                        Value::Map(m) => {
                            let label = m.get("label")
                                .and_then(|v| match v {
                                    Value::String(s) => Some(s.clone()),
                                    _ => None,
                                })
                                .ok_or_else(|| anyhow::anyhow!("Each item must have a 'label' string field"))?;

                            let value = m.get("value")
                                .and_then(|v| match v {
                                    Value::Int(n) => Some(*n as f64),
                                    Value::Float(f) => Some(*f),
                                    _ => None,
                                })
                                .ok_or_else(|| anyhow::anyhow!("Each item must have a 'value' numeric field"))?;

                            chart_data.push((label, value));
                        }
                        _ => return Err(anyhow::anyhow!("generatePieChart() expects a list of maps")),
                    }
                }
                chart_data
            }
            _ => return Err(anyhow::anyhow!("generatePieChart() requires a list as first argument")),
        };

        if data.is_empty() {
            return Err(anyhow::anyhow!("generatePieChart() requires non-empty data"));
        }

        // Create temporary file
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let file_path = temp_dir.join(format!("chart_pie_{}.png", timestamp));

        // Generate chart
        let root = BitMapBackend::new(&file_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let total: f64 = data.iter().map(|(_, v)| v).sum();

        let colors = vec![
            RGBColor(31, 119, 180),
            RGBColor(255, 127, 14),
            RGBColor(44, 160, 44),
            RGBColor(214, 39, 40),
            RGBColor(148, 103, 189),
            RGBColor(140, 86, 75),
            RGBColor(227, 119, 194),
            RGBColor(127, 127, 127),
        ];

        let center = (400, 300);
        let radius = 200.0;

        let mut current_angle = -PI / 2.0; // Start from top

        for (i, (label, value)) in data.iter().enumerate() {
            let angle = (value / total) * 2.0 * PI;
            let end_angle = current_angle + angle;

            let color = colors[i % colors.len()];

            // Draw pie slice
            let mut points = vec![center];
            let steps = 50;
            for step in 0..=steps {
                let a = current_angle + (angle * step as f64 / steps as f64);
                let x = center.0 as f64 + radius * a.cos();
                let y = center.1 as f64 + radius * a.sin();
                points.push((x as i32, y as i32));
            }

            root.draw(&Polygon::new(points, color.filled()))?;

            // Draw label
            let mid_angle = current_angle + angle / 2.0;
            let label_radius = radius * 0.7;
            let label_x = center.0 as f64 + label_radius * mid_angle.cos();
            let label_y = center.1 as f64 + label_radius * mid_angle.sin();

            let percentage = (value / total * 100.0).round();
            let text = format!("{}\n{:.0}%", label, percentage);

            root.draw_text(
                &text,
                &TextStyle::from(("sans-serif", 15).into_font()).color(&BLACK),
                (label_x as i32, label_y as i32),
            )?;

            current_angle = end_angle;
        }

        root.present()?;

        Ok(Value::Image(file_path.to_string_lossy().to_string()))
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
