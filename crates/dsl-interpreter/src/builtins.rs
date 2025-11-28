use crate::error::InterpreterError;
use crate::sql::SQLExecutor;
use anyhow::Result;
use dsl_ir::{EffectKind, Span, TypeRegistry, Value};
use simplify_baml::*;
use std::collections::HashMap;
use std::env;

pub struct BuiltinFunctions {
    runtime: Option<BamlRuntime>,
    llm_client: Option<LLMClient>,
    api_key: Option<String>,
    sql_executor: Option<SQLExecutor>,
    pub last_prompt: Option<String>,
    pub last_response: Option<String>,
    pub current_function_name: Option<String>,
}

// Helper function to compare values for equality
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => (x - y).abs() < f64::EPSILON,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::List(x), Value::List(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| values_equal(a, b))
        }
        _ => false,
    }
}

impl BuiltinFunctions {
    pub fn new() -> Result<Self> {
        // Try to initialize BAML runtime and direct LLM client
        let (runtime, llm_client, api_key) = match env::var("OPENAI_API_KEY") {
            Ok(api_key) => {
                // Build IR with structured types (will be populated later via rebuild_runtime)
                let ir = IR::new();

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
            last_response: None,
            current_function_name: None,
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

    /// Call a builtin function with scope access for variable lookups
    pub async fn call_with_scope(
        &mut self,
        name: &str,
        args: Vec<Value>,
        scope_lookup: impl Fn(&str) -> Option<Value>,
    ) -> Result<Value> {
        match name.to_lowercase().as_str() {
            // SQL functions need scope access for $variable syntax
            "sql" => self.sql_with_scope(args, scope_lookup),
            // All other functions use the regular call
            _ => self.call(name, args).await,
        }
    }

    pub async fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        match name.to_lowercase().as_str() {
            // Internal functions
            "__null__" => Ok(Value::Null),
            // String functions
            "upper" => self.upper(args),
            "lower" => self.lower(args),
            "length" => self.length(args),
            "trim" => self.trim(args),
            "split" => self.split(args),
            "replace" => self.replace(args),
            "contains" => self.contains(args),
            "startswith" => self.starts_with(args),
            "endswith" => self.ends_with(args),
            "join" => self.join(args),
            // List functions
            "reverse" => self.reverse(args),
            "sort" => self.sort(args),
            "unique" => self.unique(args),
            "take" => self.take(args),
            "skip" => self.skip(args),
            "first" => self.first(args),
            "last" => self.last(args),
            "flatten" => self.flatten(args),
            // Higher-order list functions
            "map" => self.map(args),
            "filter" => self.filter(args),
            "pluck" => self.pluck(args),
            "where" => self.where_fn(args),
            "groupby" => self.group_by(args),
            "sortby" => self.sort_by(args),
            // Math functions
            "abs" => self.abs(args),
            "min" => self.min(args),
            "max" => self.max(args),
            "sum" => self.sum(args),
            "average" => self.average(args),
            "round" => self.round(args),
            "floor" => self.floor(args),
            "ceil" => self.ceil(args),
            // Type conversion functions
            "tostring" => self.to_string(args),
            "toint" => self.to_int(args),
            "tofloat" => self.to_float(args),
            // Utility list functions
            "zip" => self.zip(args),
            "range" => self.range(args),
            "repeat" => self.repeat(args),
            "chunk" => self.chunk(args),
            // LLM functions
            "ask" => self.ask(args).await,
            "rendermarkdown" => self.render_markdown(args),
            // SQL functions
            "sql" => self.sql(args),
            "refresh_table" => self.refresh_table(args),
            // Concurrency functions
            "par" => Ok(self.par(args)),
            // Logic functions
            "not" => self.not(args),
            // Chart generation functions
            "generatebarchart" => self.generate_bar_chart(args),
            "generatelinechart" => self.generate_line_chart(args),
            "generatepiechart" => self.generate_pie_chart(args),
            // RSS feed functions
            "rss" => self.rss(args).await,
            _ => Err(anyhow::anyhow!("Unknown function: {}", name)),
        }
    }

    /// Call an intrinsic builtin function with already-evaluated arguments
    ///
    /// Intrinsic functions are generated by the lowering pass and have names starting with `__`.
    /// They correspond to special execution constructs like LLM, HTTP, and SQL.
    pub async fn call_intrinsic_with_values(
        &mut self,
        name: &str,
        args: &[Value],
        _effect_kind: Option<EffectKind>,
        span: Option<Span>,
        scope_lookup: impl Fn(&str) -> Option<Value>,
    ) -> Result<Value, InterpreterError> {
        match name {
            "__llm_execute" => {
                if args.len() != 2 {
                    return Err(InterpreterError::InvalidArguments {
                        message: format!(
                            "__llm_execute expects 2 arguments (prompt, config), got {}",
                            args.len()
                        ),
                        source_span: span,
                    });
                }
                self.intrinsic_llm_execute(&args[0], &args[1], span).await
            }
            "__http" => {
                if args.len() != 5 {
                    return Err(InterpreterError::InvalidArguments {
                        message: format!(
                            "__http expects 5 arguments (method, url, params, headers, body), got {}",
                            args.len()
                        ),
                        source_span: span.clone(),
                    });
                }
                self.intrinsic_http(&args[0], &args[1], &args[2], &args[3], &args[4], span)
                    .await
            }
            "__sql" => {
                if args.len() != 1 {
                    return Err(InterpreterError::InvalidArguments {
                        message: format!("__sql expects 1 argument (query), got {}", args.len()),
                        source_span: span.clone(),
                    });
                }
                self.intrinsic_sql(&args[0], span, scope_lookup).await
            }
            _ => Err(InterpreterError::UnknownIntrinsic {
                name: name.to_string(),
                source_span: span,
            }),
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

        self.last_response = Some(response.clone());

        Ok(Value::String(response))
    }

    /// Execute LLM with a custom prompt template and typed output
    /// This is designed for user-defined prompts in function definitions
    pub async fn execute_with_prompt_template(
        &mut self,
        prompt_template: &str,
        params: HashMap<String, Value>,
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

        // Convert params to BamlValue
        let baml_params: HashMap<String, BamlValue> = params
            .into_iter()
            .map(|(k, v)| (k, Self::value_to_baml_value(v)))
            .collect();

        // Use the user's prompt template directly with generate_prompt_from_ir
        // This will handle both variable interpolation and schema generation
        let prompt =
            generate_prompt_from_ir(runtime.ir(), prompt_template, &baml_params, &output_type)
                .map_err(|e| anyhow::anyhow!("Failed to generate prompt: {}", e))?;

        self.last_prompt = Some(prompt.clone());

        let client = self.create_client(model, base_url, api_key_env)?;
        let raw_response = client
            .call(&prompt)
            .await
            .map_err(|e| anyhow::anyhow!("LLM call failed: {}", e))?;

        self.last_response = Some(raw_response.clone());

        // Strip markdown code fences if present
        let cleaned_response = Self::strip_markdown_fences(&raw_response);

        let result = parse_llm_response_with_ir(runtime.ir(), &cleaned_response, &output_type)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse LLM response using IR\n  Error: {}\n  IR Raw response: {}",
                    e,
                    cleaned_response
                )
            })?;

        Ok(Self::baml_value_to_value(result))
    }

    /// Strip markdown code fences from LLM responses
    /// Handles both ```json ... ``` and ``` ... ``` formats
    fn strip_markdown_fences(response: &str) -> String {
        let trimmed = response.trim();

        // Check if response is wrapped in code fences
        if trimmed.starts_with("```") {
            // Find the first newline (end of opening fence)
            if let Some(start) = trimmed.find('\n') {
                // Find the closing fence
                if let Some(end) = trimmed.rfind("```") {
                    if end > start {
                        // Extract content between fences
                        return trimmed[start + 1..end].trim().to_string();
                    }
                }
            }
        }

        // No fences found, return as-is
        response.to_string()
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

    /// Get a cloneable LLM client for parallel execution
    pub fn get_llm_client_for_parallel(&self) -> Option<LLMClient> {
        self.llm_client.clone()
    }

    /// Static helper for parallel LLM calls - doesn't need &mut self
    pub async fn call_llm_static(client: LLMClient, prompt: String) -> Result<Value> {
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

        self.last_response = Some(response.clone());

        Ok(Value::String(response))
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

    fn value_to_baml_value(value: Value) -> BamlValue {
        match value {
            Value::String(s) => BamlValue::String(s),
            Value::Int(i) => BamlValue::Int(i),
            Value::Float(f) => BamlValue::Float(f),
            Value::Bool(b) => BamlValue::Bool(b),
            Value::Null => BamlValue::Null,
            Value::List(items) => {
                BamlValue::List(items.into_iter().map(Self::value_to_baml_value).collect())
            }
            Value::Map(map) => BamlValue::Map(
                map.into_iter()
                    .map(|(k, v)| (k, Self::value_to_baml_value(v)))
                    .collect(),
            ),
            Value::Markdown(s) => BamlValue::String(s),
            Value::Image(s) => BamlValue::String(s),
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

    fn sql_with_scope(
        &mut self,
        args: Vec<Value>,
        scope_lookup: impl Fn(&str) -> Option<Value>,
    ) -> Result<Value> {
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

        // Prepare query with $variable auto-registration
        let prepared_query = executor
            .prepare_query_with_variables(&query, scope_lookup)
            .map_err(|e| anyhow::anyhow!("Variable registration failed: {}", e))?;

        // Register additional tables from arguments
        for (i, arg) in args.iter().skip(1).enumerate() {
            let table_name = format!("table{}", i + 1);
            executor
                .register_table(&table_name, arg)
                .map_err(|e| anyhow::anyhow!("Failed to register table: {}", e))?;
        }

        executor
            .execute(&prepared_query, &HashMap::new())
            .map_err(|e| anyhow::anyhow!("SQL execution failed: {}", e))
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

    fn refresh_table(&mut self, args: Vec<Value>) -> Result<Value> {
        if args.is_empty() {
            return Err(anyhow::anyhow!(
                "refresh_table() requires 1 argument (table name)"
            ));
        }

        let table_name = match &args[0] {
            Value::String(s) => s.clone(),
            _ => {
                return Err(anyhow::anyhow!(
                    "refresh_table() argument must be a string (table name)"
                ))
            }
        };

        let executor = self
            .sql_executor
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("SQL executor failed to initialize"))?;

        executor
            .clear_table(&table_name)
            .map_err(|e| anyhow::anyhow!("Failed to refresh table: {}", e))?;

        Ok(Value::Null)
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
    /// Takes a list of {label: String, value: Number} and optional theme string
    /// Themes: "blue" (default), "green", "red", "purple", "orange", "dark"
    fn generate_bar_chart(&self, args: Vec<Value>) -> Result<Value> {
        use plotters::prelude::*;

        if args.is_empty() {
            return Err(anyhow::anyhow!(
                "generateBarChart() requires at least 1 argument (data)"
            ));
        }

        // Get optional theme parameter (default: "blue")
        let theme = args
            .get(1)
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("blue");

        // Parse data: expect list of maps with "label" and "value"
        let data = match &args[0] {
            Value::List(items) => {
                let mut chart_data = Vec::new();
                for item in items {
                    match item {
                        Value::Map(m) => {
                            let label = m
                                .get("label")
                                .and_then(|v| match v {
                                    Value::String(s) => Some(s.clone()),
                                    _ => None,
                                })
                                .ok_or_else(|| {
                                    anyhow::anyhow!("Each item must have a 'label' string field")
                                })?;

                            let value = m
                                .get("value")
                                .and_then(|v| match v {
                                    Value::Int(n) => Some(*n as f64),
                                    Value::Float(f) => Some(*f),
                                    _ => None,
                                })
                                .ok_or_else(|| {
                                    anyhow::anyhow!("Each item must have a 'value' numeric field")
                                })?;

                            chart_data.push((label, value));
                        }
                        _ => {
                            return Err(anyhow::anyhow!(
                                "generateBarChart() expects a list of maps"
                            ))
                        }
                    }
                }
                chart_data
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "generateBarChart() requires a list as first argument"
                ))
            }
        };

        if data.is_empty() {
            return Err(anyhow::anyhow!(
                "generateBarChart() requires non-empty data"
            ));
        }

        // Create temporary file
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let file_path = temp_dir.join(format!("chart_bar_{}.png", timestamp));

        // Select colors based on theme
        let (bg_color, bar_color) = match theme {
            "green" => (WHITE, RGBColor(76, 175, 80)),
            "red" => (WHITE, RGBColor(244, 67, 54)),
            "purple" => (WHITE, RGBColor(156, 39, 176)),
            "orange" => (WHITE, RGBColor(255, 152, 0)),
            "dark" => (RGBColor(30, 30, 30), RGBColor(66, 165, 245)),
            _ => (WHITE, BLUE), // default "blue"
        };

        // Generate chart
        let root = BitMapBackend::new(&file_path, (800, 600)).into_drawing_area();
        root.fill(&bg_color)?;

        let max_value = data.iter().map(|(_, v)| *v).fold(0.0f64, f64::max);
        let y_max = (max_value * 1.2).max(1.0);

        let mut chart = ChartBuilder::on(&root)
            .caption("Bar Chart", ("sans-serif", 40))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d((0usize..data.len()).into_segmented(), 0f64..y_max)?;

        chart
            .configure_mesh()
            .x_labels(data.len())
            .x_label_formatter(&|x| {
                if let SegmentValue::CenterOf(idx) = x {
                    data.get(*idx)
                        .map(|(label, _)| label.clone())
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            })
            .draw()?;

        chart.draw_series(data.iter().enumerate().map(|(i, (_, value))| {
            let x = SegmentValue::CenterOf(i);
            let mut bar = Rectangle::new([(x.clone(), 0.0), (x, *value)], bar_color.filled());
            bar.set_margin(0, 0, 5, 5);
            bar
        }))?;

        root.present()?;

        Ok(Value::Image(file_path.to_string_lossy().to_string()))
    }

    /// generateLineChart() - Generate a line chart image
    /// Takes a list of numbers and optional theme string
    /// Themes: "blue" (default), "green", "red", "purple", "orange", "dark"
    fn generate_line_chart(&self, args: Vec<Value>) -> Result<Value> {
        use plotters::prelude::*;

        if args.is_empty() {
            return Err(anyhow::anyhow!(
                "generateLineChart() requires at least 1 argument (data)"
            ));
        }

        // Get optional theme parameter (default: "blue")
        let theme = args
            .get(1)
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("blue");

        // Parse data: expect list of numbers
        let data = match &args[0] {
            Value::List(items) => {
                let mut chart_data = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    let value = match item {
                        Value::Int(n) => *n as f64,
                        Value::Float(f) => *f,
                        _ => {
                            return Err(anyhow::anyhow!(
                                "generateLineChart() expects a list of numbers"
                            ))
                        }
                    };
                    chart_data.push((i as f64, value));
                }
                chart_data
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "generateLineChart() requires a list as first argument"
                ))
            }
        };

        if data.is_empty() {
            return Err(anyhow::anyhow!(
                "generateLineChart() requires non-empty data"
            ));
        }

        // Create temporary file
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let file_path = temp_dir.join(format!("chart_line_{}.png", timestamp));

        // Select colors based on theme
        let (bg_color, line_color) = match theme {
            "green" => (WHITE, RGBColor(76, 175, 80)),
            "red" => (WHITE, RGBColor(244, 67, 54)),
            "purple" => (WHITE, RGBColor(156, 39, 176)),
            "orange" => (WHITE, RGBColor(255, 152, 0)),
            "dark" => (RGBColor(30, 30, 30), RGBColor(66, 165, 245)),
            _ => (WHITE, BLUE), // default "blue"
        };

        // Generate chart
        let root = BitMapBackend::new(&file_path, (800, 600)).into_drawing_area();
        root.fill(&bg_color)?;

        let max_value = data
            .iter()
            .map(|(_, v)| *v)
            .fold(f64::NEG_INFINITY, f64::max);
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

        chart.draw_series(LineSeries::new(data.clone(), &line_color))?;

        chart.draw_series(PointSeries::of_element(
            data,
            5,
            &line_color,
            &|coord, size, style| {
                EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled())
            },
        ))?;

        root.present()?;

        Ok(Value::Image(file_path.to_string_lossy().to_string()))
    }

    /// generatePieChart() - Generate a pie chart image
    /// Takes a list of {label: String, value: Number} and optional theme string
    /// Themes: "default", "green", "red", "purple", "orange", "dark"
    fn generate_pie_chart(&self, args: Vec<Value>) -> Result<Value> {
        use plotters::prelude::*;
        use std::f64::consts::PI;

        if args.is_empty() {
            return Err(anyhow::anyhow!(
                "generatePieChart() requires at least 1 argument (data)"
            ));
        }

        // Get optional theme parameter (default: "default")
        let theme = args
            .get(1)
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("default");

        // Parse data: expect list of maps with "label" and "value"
        let data = match &args[0] {
            Value::List(items) => {
                let mut chart_data = Vec::new();
                for item in items {
                    match item {
                        Value::Map(m) => {
                            let label = m
                                .get("label")
                                .and_then(|v| match v {
                                    Value::String(s) => Some(s.clone()),
                                    _ => None,
                                })
                                .ok_or_else(|| {
                                    anyhow::anyhow!("Each item must have a 'label' string field")
                                })?;

                            let value = m
                                .get("value")
                                .and_then(|v| match v {
                                    Value::Int(n) => Some(*n as f64),
                                    Value::Float(f) => Some(*f),
                                    _ => None,
                                })
                                .ok_or_else(|| {
                                    anyhow::anyhow!("Each item must have a 'value' numeric field")
                                })?;

                            chart_data.push((label, value));
                        }
                        _ => {
                            return Err(anyhow::anyhow!(
                                "generatePieChart() expects a list of maps"
                            ))
                        }
                    }
                }
                chart_data
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "generatePieChart() requires a list as first argument"
                ))
            }
        };

        if data.is_empty() {
            return Err(anyhow::anyhow!(
                "generatePieChart() requires non-empty data"
            ));
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

        // Select colors based on theme
        let (bg_color, colors) = match theme {
            "green" => (
                WHITE,
                vec![
                    RGBColor(76, 175, 80), // Green
                    RGBColor(129, 199, 132),
                    RGBColor(165, 214, 167),
                    RGBColor(200, 230, 201),
                    RGBColor(56, 142, 60),
                    RGBColor(46, 125, 50),
                    RGBColor(27, 94, 32),
                    RGBColor(139, 195, 74),
                ],
            ),
            "red" => (
                WHITE,
                vec![
                    RGBColor(244, 67, 54), // Red
                    RGBColor(239, 83, 80),
                    RGBColor(229, 115, 115),
                    RGBColor(239, 154, 154),
                    RGBColor(211, 47, 47),
                    RGBColor(198, 40, 40),
                    RGBColor(183, 28, 28),
                    RGBColor(255, 82, 82),
                ],
            ),
            "purple" => (
                WHITE,
                vec![
                    RGBColor(156, 39, 176), // Purple
                    RGBColor(171, 71, 188),
                    RGBColor(186, 104, 200),
                    RGBColor(206, 147, 216),
                    RGBColor(123, 31, 162),
                    RGBColor(106, 27, 154),
                    RGBColor(74, 20, 140),
                    RGBColor(170, 0, 255),
                ],
            ),
            "orange" => (
                WHITE,
                vec![
                    RGBColor(255, 152, 0), // Orange
                    RGBColor(255, 167, 38),
                    RGBColor(255, 183, 77),
                    RGBColor(255, 204, 128),
                    RGBColor(251, 140, 0),
                    RGBColor(245, 124, 0),
                    RGBColor(230, 81, 0),
                    RGBColor(255, 171, 64),
                ],
            ),
            "dark" => (
                RGBColor(30, 30, 30),
                vec![
                    RGBColor(66, 165, 245),  // Blue
                    RGBColor(102, 187, 106), // Green
                    RGBColor(255, 202, 40),  // Amber
                    RGBColor(255, 138, 101), // Deep Orange
                    RGBColor(171, 71, 188),  // Purple
                    RGBColor(38, 198, 218),  // Cyan
                    RGBColor(255, 112, 67),  // Deep Orange
                    RGBColor(126, 87, 194),  // Deep Purple
                ],
            ),
            _ => (
                WHITE,
                vec![
                    // default colorful palette
                    RGBColor(31, 119, 180),
                    RGBColor(255, 127, 14),
                    RGBColor(44, 160, 44),
                    RGBColor(214, 39, 40),
                    RGBColor(148, 103, 189),
                    RGBColor(140, 86, 75),
                    RGBColor(227, 119, 194),
                    RGBColor(127, 127, 127),
                ],
            ),
        };

        root.fill(&bg_color)?;

        let total: f64 = data.iter().map(|(_, v)| v).sum();

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

    // ========================================
    // RSS feed functions
    // ========================================

    /// rss() - Fetch and parse an RSS feed
    /// Takes a URL string and returns a list of feed items
    /// Each item is a map with fields: title, link, description, pub_date, author, content
    async fn rss(&self, args: Vec<Value>) -> Result<Value> {
        use rss::Channel;

        if args.is_empty() {
            return Err(anyhow::anyhow!("rss() requires at least 1 argument (url)"));
        }

        let url = match &args[0] {
            Value::String(s) => s.clone(),
            _ => {
                return Err(anyhow::anyhow!(
                    "rss() first argument must be a URL string, got {}",
                    args[0].type_name()
                ))
            }
        };

        // Optional: limit number of items (default: all items)
        let limit = args.get(1).and_then(|v| match v {
            Value::Int(n) => Some(*n as usize),
            _ => None,
        });

        // Fetch the RSS feed
        let response = reqwest::get(&url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch RSS feed from {}: {}", url, e))?;

        let content = response
            .bytes()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read RSS feed content: {}", e))?;

        // Parse the RSS feed
        let channel = Channel::read_from(&content[..])
            .map_err(|e| anyhow::anyhow!("Failed to parse RSS feed: {}", e))?;

        // Convert items to our Value format
        let items: Vec<Value> = channel
            .items()
            .iter()
            .take(limit.unwrap_or(usize::MAX))
            .map(|item| {
                let mut map = indexmap::IndexMap::new();

                // Title
                if let Some(title) = item.title() {
                    map.insert("title".to_string(), Value::String(title.to_string()));
                }

                // Link
                if let Some(link) = item.link() {
                    map.insert("link".to_string(), Value::String(link.to_string()));
                }

                // Description
                if let Some(description) = item.description() {
                    map.insert(
                        "description".to_string(),
                        Value::String(description.to_string()),
                    );
                }

                // Publication date
                if let Some(pub_date) = item.pub_date() {
                    map.insert("pub_date".to_string(), Value::String(pub_date.to_string()));
                }

                // Author
                if let Some(author) = item.author() {
                    map.insert("author".to_string(), Value::String(author.to_string()));
                }

                // Content (if available)
                if let Some(content) = item.content() {
                    map.insert("content".to_string(), Value::String(content.to_string()));
                }

                // Categories
                let categories: Vec<Value> = item
                    .categories()
                    .iter()
                    .map(|cat| Value::String(cat.name().to_string()))
                    .collect();
                if !categories.is_empty() {
                    map.insert("categories".to_string(), Value::List(categories));
                }

                // GUID
                if let Some(guid) = item.guid() {
                    map.insert("guid".to_string(), Value::String(guid.value().to_string()));
                }

                Value::Map(map)
            })
            .collect();

        // Also include channel metadata
        let mut result = indexmap::IndexMap::new();

        // Channel title
        result.insert(
            "title".to_string(),
            Value::String(channel.title().to_string()),
        );

        // Channel link
        result.insert(
            "link".to_string(),
            Value::String(channel.link().to_string()),
        );

        // Channel description
        result.insert(
            "description".to_string(),
            Value::String(channel.description().to_string()),
        );

        // Items
        result.insert("items".to_string(), Value::List(items));

        Ok(Value::Map(result))
    }

    // ========================================
    // String processing functions
    // ========================================

    fn trim(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Trim() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::String(s) => Ok(Value::String(s.trim().to_string())),
            _ => Err(anyhow::anyhow!(
                "Trim() requires a string, got {}",
                args[0].type_name()
            )),
        }
    }

    fn split(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "Split() requires exactly 2 arguments (string, separator)"
            ));
        }

        let string = match &args[0] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "Split() first argument must be a string, got {}",
                    args[0].type_name()
                ))
            }
        };

        let separator = match &args[1] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "Split() second argument must be a string, got {}",
                    args[1].type_name()
                ))
            }
        };

        let parts: Vec<Value> = string
            .split(separator.as_str())
            .map(|s| Value::String(s.to_string()))
            .collect();

        Ok(Value::List(parts))
    }

    fn replace(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 3 {
            return Err(anyhow::anyhow!(
                "Replace() requires exactly 3 arguments (string, pattern, replacement)"
            ));
        }

        let string = match &args[0] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "Replace() first argument must be a string, got {}",
                    args[0].type_name()
                ))
            }
        };

        let pattern = match &args[1] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "Replace() second argument must be a string, got {}",
                    args[1].type_name()
                ))
            }
        };

        let replacement = match &args[2] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "Replace() third argument must be a string, got {}",
                    args[2].type_name()
                ))
            }
        };

        Ok(Value::String(
            string.replace(pattern.as_str(), replacement.as_str()),
        ))
    }

    fn contains(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "Contains() requires exactly 2 arguments (string, substring)"
            ));
        }

        let string = match &args[0] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "Contains() first argument must be a string, got {}",
                    args[0].type_name()
                ))
            }
        };

        let substring = match &args[1] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "Contains() second argument must be a string, got {}",
                    args[1].type_name()
                ))
            }
        };

        Ok(Value::Bool(string.contains(substring.as_str())))
    }

    fn starts_with(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "StartsWith() requires exactly 2 arguments (string, prefix)"
            ));
        }

        let string = match &args[0] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "StartsWith() first argument must be a string, got {}",
                    args[0].type_name()
                ))
            }
        };

        let prefix = match &args[1] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "StartsWith() second argument must be a string, got {}",
                    args[1].type_name()
                ))
            }
        };

        Ok(Value::Bool(string.starts_with(prefix.as_str())))
    }

    fn ends_with(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "EndsWith() requires exactly 2 arguments (string, suffix)"
            ));
        }

        let string = match &args[0] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "EndsWith() first argument must be a string, got {}",
                    args[0].type_name()
                ))
            }
        };

        let suffix = match &args[1] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "EndsWith() second argument must be a string, got {}",
                    args[1].type_name()
                ))
            }
        };

        Ok(Value::Bool(string.ends_with(suffix.as_str())))
    }

    // ========================================
    // List processing functions
    // ========================================

    fn reverse(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Reverse() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::List(items) => {
                let mut reversed = items.clone();
                reversed.reverse();
                Ok(Value::List(reversed))
            }
            _ => Err(anyhow::anyhow!(
                "Reverse() requires a list, got {}",
                args[0].type_name()
            )),
        }
    }

    fn sort(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Sort() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::List(items) => {
                let mut sorted = items.clone();
                sorted.sort_by(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => x.cmp(y),
                    (Value::Float(x), Value::Float(y)) => {
                        x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::String(x), Value::String(y)) => x.cmp(y),
                    (Value::Int(x), Value::Float(y)) => (*x as f64)
                        .partial_cmp(y)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    (Value::Float(x), Value::Int(y)) => x
                        .partial_cmp(&(*y as f64))
                        .unwrap_or(std::cmp::Ordering::Equal),
                    _ => std::cmp::Ordering::Equal,
                });
                Ok(Value::List(sorted))
            }
            _ => Err(anyhow::anyhow!(
                "Sort() requires a list, got {}",
                args[0].type_name()
            )),
        }
    }

    fn unique(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Unique() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::List(items) => {
                let mut unique_items = Vec::new();
                for item in items {
                    if !unique_items.iter().any(|v| values_equal(v, item)) {
                        unique_items.push(item.clone());
                    }
                }
                Ok(Value::List(unique_items))
            }
            _ => Err(anyhow::anyhow!(
                "Unique() requires a list, got {}",
                args[0].type_name()
            )),
        }
    }

    fn take(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "Take() requires exactly 2 arguments (list, count)"
            ));
        }

        let items = match &args[0] {
            Value::List(items) => items,
            _ => {
                return Err(anyhow::anyhow!(
                    "Take() first argument must be a list, got {}",
                    args[0].type_name()
                ))
            }
        };

        let count = match &args[1] {
            Value::Int(n) => *n as usize,
            _ => {
                return Err(anyhow::anyhow!(
                    "Take() second argument must be an integer, got {}",
                    args[1].type_name()
                ))
            }
        };

        let taken: Vec<Value> = items.iter().take(count).cloned().collect();
        Ok(Value::List(taken))
    }

    fn skip(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "Skip() requires exactly 2 arguments (list, count)"
            ));
        }

        let items = match &args[0] {
            Value::List(items) => items,
            _ => {
                return Err(anyhow::anyhow!(
                    "Skip() first argument must be a list, got {}",
                    args[0].type_name()
                ))
            }
        };

        let count = match &args[1] {
            Value::Int(n) => *n as usize,
            _ => {
                return Err(anyhow::anyhow!(
                    "Skip() second argument must be an integer, got {}",
                    args[1].type_name()
                ))
            }
        };

        let skipped: Vec<Value> = items.iter().skip(count).cloned().collect();
        Ok(Value::List(skipped))
    }

    fn first(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("First() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::List(items) => items
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("First() called on empty list")),
            _ => Err(anyhow::anyhow!(
                "First() requires a list, got {}",
                args[0].type_name()
            )),
        }
    }

    fn last(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Last() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::List(items) => items
                .last()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Last() called on empty list")),
            _ => Err(anyhow::anyhow!(
                "Last() requires a list, got {}",
                args[0].type_name()
            )),
        }
    }

    fn flatten(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Flatten() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::List(items) => {
                let mut flattened = Vec::new();
                for item in items {
                    match item {
                        Value::List(inner) => flattened.extend(inner.clone()),
                        other => flattened.push(other.clone()),
                    }
                }
                Ok(Value::List(flattened))
            }
            _ => Err(anyhow::anyhow!(
                "Flatten() requires a list, got {}",
                args[0].type_name()
            )),
        }
    }

    // ========================================
    // Higher-order list functions
    // ========================================

    /// pluck() - Extract field from each map in a list
    /// Usage: pluck(list, "field_name")
    /// Example: pluck(items, "title") extracts all titles
    fn pluck(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "pluck() requires exactly 2 arguments (list, field_name)"
            ));
        }

        let items = match &args[0] {
            Value::List(items) => items,
            _ => {
                return Err(anyhow::anyhow!(
                    "pluck() first argument must be a list, got {}",
                    args[0].type_name()
                ))
            }
        };

        let field_name = match &args[1] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "pluck() second argument must be a field name string, got {}",
                    args[1].type_name()
                ))
            }
        };

        let result: Vec<Value> = items
            .iter()
            .filter_map(|item| match item {
                Value::Map(m) => m.get(field_name).cloned(),
                _ => None,
            })
            .collect();

        Ok(Value::List(result))
    }

    /// map() - Note: This is a placeholder that returns an error with instructions
    /// True higher-order map requires first-class functions which aren't yet implemented
    /// Use pluck() for field extraction instead
    fn map(&self, args: Vec<Value>) -> Result<Value> {
        // For now, map is not implemented as a true higher-order function
        // because the DSL doesn't yet support passing functions as values
        Err(anyhow::anyhow!(
            "map() with functions is not yet implemented. Use pluck(list, \"field\") to extract fields from maps, or define a custom function using pattern matching"
        ))
    }

    /// filter() - Filter list by field value
    /// Usage: filter(list, "field_name", value)
    /// Example: filter(items, "author", "John") keeps only items where author is "John"
    fn filter(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 3 {
            return Err(anyhow::anyhow!(
                "filter() requires exactly 3 arguments (list, field_name, value)"
            ));
        }

        let items = match &args[0] {
            Value::List(items) => items,
            _ => {
                return Err(anyhow::anyhow!(
                    "filter() first argument must be a list, got {}",
                    args[0].type_name()
                ))
            }
        };

        let field_name = match &args[1] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "filter() second argument must be a field name string, got {}",
                    args[1].type_name()
                ))
            }
        };

        let target_value = &args[2];

        let result: Vec<Value> = items
            .iter()
            .filter(|item| match item {
                Value::Map(m) => m
                    .get(field_name)
                    .map(|v| values_equal(v, target_value))
                    .unwrap_or(false),
                _ => false,
            })
            .cloned()
            .collect();

        Ok(Value::List(result))
    }

    /// where() - Filter list by field existence
    /// Usage: where(list, "field_name")
    /// Example: where(items, "author") keeps only items that have an author field
    fn where_fn(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "where() requires exactly 2 arguments (list, field_name)"
            ));
        }

        let items = match &args[0] {
            Value::List(items) => items,
            _ => {
                return Err(anyhow::anyhow!(
                    "where() first argument must be a list, got {}",
                    args[0].type_name()
                ))
            }
        };

        let field_name = match &args[1] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "where() second argument must be a field name string, got {}",
                    args[1].type_name()
                ))
            }
        };

        let result: Vec<Value> = items
            .iter()
            .filter(|item| match item {
                Value::Map(m) => m.contains_key(field_name),
                _ => false,
            })
            .cloned()
            .collect();

        Ok(Value::List(result))
    }

    /// groupby() - Group list items by field value
    /// Usage: groupby(list, "field_name")
    /// Returns a map where keys are field values and values are lists of matching items
    fn group_by(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "groupby() requires exactly 2 arguments (list, field_name)"
            ));
        }

        let items = match &args[0] {
            Value::List(items) => items,
            _ => {
                return Err(anyhow::anyhow!(
                    "groupby() first argument must be a list, got {}",
                    args[0].type_name()
                ))
            }
        };

        let field_name = match &args[1] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "groupby() second argument must be a field name string, got {}",
                    args[1].type_name()
                ))
            }
        };

        let mut groups: indexmap::IndexMap<String, Vec<Value>> = indexmap::IndexMap::new();

        for item in items {
            if let Value::Map(m) = item {
                if let Some(key_value) = m.get(field_name) {
                    let key = match key_value {
                        Value::String(s) => s.clone(),
                        Value::Int(n) => n.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => continue,
                    };

                    groups
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .push(item.clone());
                }
            }
        }

        let result: indexmap::IndexMap<String, Value> = groups
            .into_iter()
            .map(|(k, v)| (k, Value::List(v)))
            .collect();

        Ok(Value::Map(result))
    }

    /// sortby() - Sort list by field value
    /// Usage: sortby(list, "field_name")
    /// Example: sortby(items, "pub_date") sorts items by publication date
    fn sort_by(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "sortby() requires exactly 2 arguments (list, field_name)"
            ));
        }

        let items = match &args[0] {
            Value::List(items) => items,
            _ => {
                return Err(anyhow::anyhow!(
                    "sortby() first argument must be a list, got {}",
                    args[0].type_name()
                ))
            }
        };

        let field_name = match &args[1] {
            Value::String(s) => s,
            _ => {
                return Err(anyhow::anyhow!(
                    "sortby() second argument must be a field name string, got {}",
                    args[1].type_name()
                ))
            }
        };

        let mut sorted = items.clone();
        sorted.sort_by(|a, b| {
            let a_val = match a {
                Value::Map(m) => m.get(field_name),
                _ => None,
            };
            let b_val = match b {
                Value::Map(m) => m.get(field_name),
                _ => None,
            };

            match (a_val, b_val) {
                (Some(Value::Int(x)), Some(Value::Int(y))) => x.cmp(y),
                (Some(Value::Float(x)), Some(Value::Float(y))) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Some(Value::String(x)), Some(Value::String(y))) => x.cmp(y),
                (Some(Value::Int(x)), Some(Value::Float(y))) => (*x as f64)
                    .partial_cmp(y)
                    .unwrap_or(std::cmp::Ordering::Equal),
                (Some(Value::Float(x)), Some(Value::Int(y))) => x
                    .partial_cmp(&(*y as f64))
                    .unwrap_or(std::cmp::Ordering::Equal),
                _ => std::cmp::Ordering::Equal,
            }
        });

        Ok(Value::List(sorted))
    }

    // ========================================
    // Math functions
    // ========================================

    fn abs(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Abs() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::Int(n) => Ok(Value::Int(n.abs())),
            Value::Float(f) => Ok(Value::Float(f.abs())),
            _ => Err(anyhow::anyhow!(
                "Abs() requires a number, got {}",
                args[0].type_name()
            )),
        }
    }

    fn min(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Min() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::List(items) => {
                if items.is_empty() {
                    return Err(anyhow::anyhow!("Min() called on empty list"));
                }

                let mut min_val = &items[0];
                for item in items.iter().skip(1) {
                    match (min_val, item) {
                        (Value::Int(x), Value::Int(y)) if y < x => min_val = item,
                        (Value::Float(x), Value::Float(y)) if y < x => min_val = item,
                        (Value::Int(x), Value::Float(y)) if y < &(*x as f64) => min_val = item,
                        (Value::Float(x), Value::Int(y)) if (*y as f64) < *x => min_val = item,
                        _ => {}
                    }
                }
                Ok(min_val.clone())
            }
            _ => Err(anyhow::anyhow!(
                "Min() requires a list, got {}",
                args[0].type_name()
            )),
        }
    }

    fn max(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Max() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::List(items) => {
                if items.is_empty() {
                    return Err(anyhow::anyhow!("Max() called on empty list"));
                }

                let mut max_val = &items[0];
                for item in items.iter().skip(1) {
                    match (max_val, item) {
                        (Value::Int(x), Value::Int(y)) if y > x => max_val = item,
                        (Value::Float(x), Value::Float(y)) if y > x => max_val = item,
                        (Value::Int(x), Value::Float(y)) if y > &(*x as f64) => max_val = item,
                        (Value::Float(x), Value::Int(y)) if (*y as f64) > *x => max_val = item,
                        _ => {}
                    }
                }
                Ok(max_val.clone())
            }
            _ => Err(anyhow::anyhow!(
                "Max() requires a list, got {}",
                args[0].type_name()
            )),
        }
    }

    fn sum(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Sum() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::List(items) => {
                let mut int_sum = 0i64;
                let mut float_sum = 0.0f64;
                let mut has_float = false;

                for item in items {
                    match item {
                        Value::Int(n) => {
                            if has_float {
                                float_sum += *n as f64;
                            } else {
                                int_sum += n;
                            }
                        }
                        Value::Float(f) => {
                            if !has_float {
                                float_sum = int_sum as f64;
                                has_float = true;
                            }
                            float_sum += f;
                        }
                        _ => {
                            return Err(anyhow::anyhow!(
                                "Sum() requires a list of numbers, found {}",
                                item.type_name()
                            ))
                        }
                    }
                }

                if has_float {
                    Ok(Value::Float(float_sum))
                } else {
                    Ok(Value::Int(int_sum))
                }
            }
            _ => Err(anyhow::anyhow!(
                "Sum() requires a list, got {}",
                args[0].type_name()
            )),
        }
    }

    fn average(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Average() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::List(items) => {
                if items.is_empty() {
                    return Err(anyhow::anyhow!("Average() called on empty list"));
                }

                let count = items.len() as f64;
                let sum = self.sum(args)?;

                match sum {
                    Value::Int(n) => Ok(Value::Float(n as f64 / count)),
                    Value::Float(f) => Ok(Value::Float(f / count)),
                    _ => Err(anyhow::anyhow!("Sum() returned unexpected type")),
                }
            }
            _ => Err(anyhow::anyhow!(
                "Average() requires a list, got {}",
                args[0].type_name()
            )),
        }
    }

    fn round(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Round() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::Float(f) => Ok(Value::Int(f.round() as i64)),
            Value::Int(n) => Ok(Value::Int(*n)),
            _ => Err(anyhow::anyhow!(
                "Round() requires a number, got {}",
                args[0].type_name()
            )),
        }
    }

    fn floor(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Floor() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::Float(f) => Ok(Value::Int(f.floor() as i64)),
            Value::Int(n) => Ok(Value::Int(*n)),
            _ => Err(anyhow::anyhow!(
                "Floor() requires a number, got {}",
                args[0].type_name()
            )),
        }
    }

    fn ceil(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("Ceil() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::Float(f) => Ok(Value::Int(f.ceil() as i64)),
            Value::Int(n) => Ok(Value::Int(*n)),
            _ => Err(anyhow::anyhow!(
                "Ceil() requires a number, got {}",
                args[0].type_name()
            )),
        }
    }

    // ========================================
    // Type conversion functions
    // ========================================

    fn to_string(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("ToString() requires exactly 1 argument"));
        }

        let string = match &args[0] {
            Value::String(s) => s.clone(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            other => other.display(),
        };

        Ok(Value::String(string))
    }

    fn to_int(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("ToInt() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::Int(n) => Ok(Value::Int(*n)),
            Value::Float(f) => Ok(Value::Int(*f as i64)),
            Value::String(s) => s
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|_| anyhow::anyhow!("Failed to parse '{}' as integer", s)),
            _ => Err(anyhow::anyhow!(
                "ToInt() cannot convert {} to integer",
                args[0].type_name()
            )),
        }
    }

    fn to_float(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("ToFloat() requires exactly 1 argument"));
        }

        match &args[0] {
            Value::Float(f) => Ok(Value::Float(*f)),
            Value::Int(n) => Ok(Value::Float(*n as f64)),
            Value::String(s) => s
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|_| anyhow::anyhow!("Failed to parse '{}' as float", s)),
            _ => Err(anyhow::anyhow!(
                "ToFloat() cannot convert {} to float",
                args[0].type_name()
            )),
        }
    }

    // ========================================
    // Utility list functions (no function args)
    // ========================================

    fn zip(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "Zip() requires exactly 2 arguments (list1, list2)"
            ));
        }

        let list1 = match &args[0] {
            Value::List(items) => items,
            _ => {
                return Err(anyhow::anyhow!(
                    "Zip() first argument must be a list, got {}",
                    args[0].type_name()
                ))
            }
        };

        let list2 = match &args[1] {
            Value::List(items) => items,
            _ => {
                return Err(anyhow::anyhow!(
                    "Zip() second argument must be a list, got {}",
                    args[1].type_name()
                ))
            }
        };

        let zipped: Vec<Value> = list1
            .iter()
            .zip(list2.iter())
            .map(|(a, b)| Value::List(vec![a.clone(), b.clone()]))
            .collect();

        Ok(Value::List(zipped))
    }

    fn range(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "Range() requires exactly 2 arguments (start, end)"
            ));
        }

        let start = match &args[0] {
            Value::Int(n) => *n,
            _ => {
                return Err(anyhow::anyhow!(
                    "Range() first argument must be an integer, got {}",
                    args[0].type_name()
                ))
            }
        };

        let end = match &args[1] {
            Value::Int(n) => *n,
            _ => {
                return Err(anyhow::anyhow!(
                    "Range() second argument must be an integer, got {}",
                    args[1].type_name()
                ))
            }
        };

        let range: Vec<Value> = (start..end).map(Value::Int).collect();
        Ok(Value::List(range))
    }

    fn repeat(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "Repeat() requires exactly 2 arguments (value, count)"
            ));
        }

        let value = &args[0];
        let count = match &args[1] {
            Value::Int(n) => *n as usize,
            _ => {
                return Err(anyhow::anyhow!(
                    "Repeat() second argument must be an integer, got {}",
                    args[1].type_name()
                ))
            }
        };

        let repeated = vec![value.clone(); count];
        Ok(Value::List(repeated))
    }

    fn chunk(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow::anyhow!(
                "Chunk() requires exactly 2 arguments (list, size)"
            ));
        }

        let items = match &args[0] {
            Value::List(items) => items,
            _ => {
                return Err(anyhow::anyhow!(
                    "Chunk() first argument must be a list, got {}",
                    args[0].type_name()
                ))
            }
        };

        let size = match &args[1] {
            Value::Int(n) => *n as usize,
            _ => {
                return Err(anyhow::anyhow!(
                    "Chunk() second argument must be an integer, got {}",
                    args[1].type_name()
                ))
            }
        };

        if size == 0 {
            return Err(anyhow::anyhow!("Chunk() size must be greater than 0"));
        }

        let chunks: Vec<Value> = items
            .chunks(size)
            .map(|chunk| Value::List(chunk.to_vec()))
            .collect();

        Ok(Value::List(chunks))
    }

    // ===== Intrinsic Functions (Lowered from HIR) =====

    /// Helper: Convert Value to plain string (without quotes for strings)
    fn value_to_plain_string(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => value.display(),
        }
    }

    /// __llm_execute intrinsic: Execute LLM with prompt
    ///
    /// Arguments:
    /// - prompt: Value (String with the prompt text)
    /// - config: Value (Map with optional model, base_url, api_key_env, temperature)
    async fn intrinsic_llm_execute(
        &mut self,
        prompt_value: &Value,
        config_value: &Value,
        span: Option<Span>,
    ) -> Result<Value, InterpreterError> {
        // Extract prompt string
        let prompt = match prompt_value {
            Value::String(s) => s.clone(),
            _ => {
                return Err(InterpreterError::TypeError {
                    message: "__llm_execute: prompt must be String".to_string(),
                    expected: "String".to_string(),
                    got: format!("{:?}", prompt_value),
                    source_span: span,
                });
            }
        };

        // Extract config parameters
        let config_map = match config_value {
            Value::Map(m) => m,
            _ => {
                return Err(InterpreterError::TypeError {
                    message: "__llm_execute: config must be Map".to_string(),
                    expected: "Map".to_string(),
                    got: format!("{:?}", config_value),
                    source_span: span,
                });
            }
        };

        let model = config_map.get("model").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        let base_url = config_map.get("base_url").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        let api_key_env = config_map.get("api_key_env").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        // Extract return type from config (defaults to String if not specified)
        let return_type = config_map
            .get("return_type")
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "String".to_string());

        // Call existing execute_with_prompt_template method
        // Note: We use an empty params map since template interpolation is already done
        let result = self
            .execute_with_prompt_template(
                &prompt,
                std::collections::HashMap::new(),
                return_type,
                model,
                base_url,
                api_key_env,
            )
            .await
            .map_err(|e| InterpreterError::LLMError {
                message: e.to_string(),
                function_name: self.current_function_name.clone(),
                source_span: span,
                prompt: Some(prompt),
                response: self.last_response.clone(),
            })?;

        Ok(result)
    }

    /// __http intrinsic: Execute HTTP request
    ///
    /// Arguments:
    /// - method: Value (String: GET, POST, etc.)
    /// - url: Value (String)
    /// - params: Value (Map of query parameters)
    /// - headers: Value (Map of headers)
    /// - body: Value (String, optional request body)
    async fn intrinsic_http(
        &mut self,
        method_value: &Value,
        url_value: &Value,
        params_value: &Value,
        headers_value: &Value,
        body_value: &Value,
        span: Option<Span>,
    ) -> Result<Value, InterpreterError> {
        use reqwest::Client;
        use serde_json::Value as JsonValue;

        // Extract method
        let method = match method_value {
            Value::String(s) => s.clone(),
            _ => {
                return Err(InterpreterError::TypeError {
                    message: "__http: method must be String".to_string(),
                    expected: "String".to_string(),
                    got: format!("{:?}", method_value),
                    source_span: span,
                });
            }
        };

        // Extract URL
        let url = match url_value {
            Value::String(s) => s.clone(),
            _ => {
                return Err(InterpreterError::TypeError {
                    message: "__http: url must be String".to_string(),
                    expected: "String".to_string(),
                    got: format!("{:?}", url_value),
                    source_span: span,
                });
            }
        };

        // Extract params
        let params_map = match params_value {
            Value::Map(m) => {
                let mut map = std::collections::HashMap::new();
                for (key, value) in m {
                    map.insert(key.clone(), Self::value_to_plain_string(value));
                }
                map
            }
            _ => std::collections::HashMap::new(),
        };

        // Extract headers
        let headers_map = match headers_value {
            Value::Map(m) => {
                let mut map = std::collections::HashMap::new();
                for (key, value) in m {
                    map.insert(key.clone(), Self::value_to_plain_string(value));
                }
                map
            }
            _ => std::collections::HashMap::new(),
        };

        // Extract body (optional - empty string means no body)
        let body_str = match body_value {
            Value::String(s) if !s.is_empty() => Some(s.clone()),
            _ => None,
        };

        // Execute HTTP request
        let client = Client::new();
        let mut request = match method.to_uppercase().as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            "PATCH" => client.patch(&url),
            _ => {
                return Err(InterpreterError::HTTPError {
                    message: format!("Unsupported HTTP method: {}", method),
                    function_name: self.current_function_name.clone(),
                    source_span: span,
                    method: Some(method),
                    url: Some(url),
                });
            }
        };

        // Add query parameters
        for (key, value) in params_map {
            request = request.query(&[(key, value)]);
        }

        // Add headers
        for (key, value) in headers_map {
            request = request.header(key, value);
        }

        // Add body if present
        if let Some(body) = body_str {
            request = request.body(body);
        }

        // Send request
        let response = request
            .send()
            .await
            .map_err(|e| InterpreterError::HTTPError {
                message: format!("Request failed: {}", e),
                function_name: self.current_function_name.clone(),
                source_span: span.clone(),
                method: Some(method.clone()),
                url: Some(url.clone()),
            })?;

        // Get response text
        let response_text = response
            .text()
            .await
            .map_err(|e| InterpreterError::HTTPError {
                message: format!("Failed to read response: {}", e),
                function_name: self.current_function_name.clone(),
                source_span: span,
                method: Some(method),
                url: Some(url),
            })?;

        // Try to parse as JSON, fallback to string
        if let Ok(json) = serde_json::from_str::<JsonValue>(&response_text) {
            Ok(Self::json_to_value(&json))
        } else {
            Ok(Value::String(response_text))
        }
    }

    /// __sql intrinsic: Execute SQL query
    ///
    /// Arguments:
    /// - query: Value (String with SQL)
    async fn intrinsic_sql(
        &mut self,
        query_value: &Value,
        span: Option<Span>,
        scope_lookup: impl Fn(&str) -> Option<Value>,
    ) -> Result<Value, InterpreterError> {
        // Extract query string
        let query = match query_value {
            Value::String(s) => s.clone(),
            _ => {
                return Err(InterpreterError::TypeError {
                    message: "__sql: query must be String".to_string(),
                    expected: "String".to_string(),
                    got: format!("{:?}", query_value),
                    source_span: span,
                });
            }
        };

        // Get SQL executor
        let executor =
            self.sql_executor
                .as_mut()
                .ok_or_else(|| InterpreterError::RuntimeError {
                    message: "SQL executor not initialized".to_string(),
                    source_span: span.clone(),
                })?;

        // Prepare query with auto-registration of $variables
        let prepared_query = executor
            .prepare_query_with_variables(&query, scope_lookup)
            .map_err(|e| InterpreterError::SQLError {
                message: format!("Variable registration failed: {}", e),
                function_name: self.current_function_name.clone(),
                source_span: span.clone(),
                query: Some(query.clone()),
            })?;

        // Execute the prepared query
        let params = std::collections::HashMap::new();
        executor
            .execute(&prepared_query, &params)
            .map_err(|e| InterpreterError::SQLError {
                message: format!("Query failed: {}", e),
                function_name: self.current_function_name.clone(),
                source_span: span,
                query: Some(prepared_query),
            })
    }

    /// Helper: Convert serde_json::Value to dsl_ir::Value
    fn json_to_value(json: &serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Null
                }
            }
            serde_json::Value::String(s) => Value::String(s.clone()),
            serde_json::Value::Array(arr) => {
                Value::List(arr.iter().map(Self::json_to_value).collect())
            }
            serde_json::Value::Object(obj) => {
                let mut map = indexmap::IndexMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), Self::json_to_value(v));
                }
                Value::Map(map)
            }
        }
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
