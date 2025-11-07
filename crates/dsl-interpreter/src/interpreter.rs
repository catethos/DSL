use crate::builtins::BuiltinFunctions;
use crate::pattern::PatternMatcher;
use crate::runtime::Runtime;
use dsl_ir::{IRNode, IRTemplateSegment, IRBinding, IRExecution, IRFunction, Value, IR};
use indexmap::IndexMap;
use anyhow::Result;

pub struct Interpreter {
    pub runtime: Runtime,
    builtins: BuiltinFunctions,
}

impl Interpreter {
    /// Create a new interpreter with empty state
    pub fn new() -> Result<Self> {
        Ok(Self {
            runtime: Runtime::new(),
            builtins: BuiltinFunctions::new()?,
        })
    }

    /// Create interpreter and load types/functions from IR
    pub fn from_ir(ir: &IR) -> Result<Self> {
        let mut interpreter = Self::new()?;

        // Register all types
        for class in &ir.types {
            interpreter.runtime.types.register_class(class.clone());
        }
        for enum_def in &ir.enums {
            interpreter.runtime.types.register_enum(enum_def.clone());
        }

        // Register all functions
        for func in &ir.functions {
            interpreter.runtime.functions.insert(func.name.clone(), func.clone());
        }

        // Register all function groups (overloaded functions)
        for func_group in &ir.function_groups {
            interpreter.runtime.function_groups.insert(func_group.name.clone(), func_group.clone());
        }

        // Rebuild BAML runtime with new types
        interpreter.builtins.rebuild_runtime(&interpreter.runtime.types)?;

        Ok(interpreter)
    }

    /// Rebuild the BAML runtime with current types
    pub fn rebuild_runtime(&mut self) -> Result<()> {
        self.builtins.rebuild_runtime(&self.runtime.types)
    }

    /// Main evaluation method - evaluates an IRNode and returns a Value
    pub async fn eval(&mut self, node: &IRNode) -> Result<Value, String> {
        match node {
            // ===== Simple Values (5 types) =====
            IRNode::String(s) => Ok(Value::String(s.clone())),

            IRNode::Int(i) => Ok(Value::Int(*i)),

            IRNode::Float(f) => Ok(Value::Float(*f)),

            IRNode::Bool(b) => Ok(Value::Bool(*b)),

            IRNode::TemplateString(segments) => {
                let result = self.interpolate_template(segments).await?;
                Ok(Value::String(result))
            }

            // ===== Collections (2 types) =====
            IRNode::List(items) => {
                let mut values = Vec::new();
                for item in items {
                    values.push(Box::pin(self.eval(item)).await?);
                }
                Ok(Value::List(values))
            }

            IRNode::Map(entries) => {
                let mut map = IndexMap::new();
                for (key, value_expr) in entries {
                    let value = Box::pin(self.eval(value_expr)).await?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Map(map))
            }

            // ===== Variables & Access (3 types) =====
            IRNode::Variable(name) => self.runtime.get_var(name),

            IRNode::FieldAccess { base, field } => {
                let base_value = Box::pin(self.eval(base)).await?;
                self.access_field(&base_value, field)
            }

            IRNode::IndexAccess { base, index } => {
                let base_value = Box::pin(self.eval(base)).await?;
                let index_value = Box::pin(self.eval(index)).await?;
                self.access_index(&base_value, &index_value)
            }

            // ===== Operations (1 type) =====
            IRNode::BinaryOp { left, op, right } => {
                let left_val = Box::pin(self.eval(left)).await?;
                let right_val = Box::pin(self.eval(right)).await?;
                self.apply_binary_op(op, left_val, right_val)
            }

            // ===== Control Flow (2 types) =====
            IRNode::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond_val = Box::pin(self.eval(condition)).await?;
                let is_true = match cond_val {
                    Value::Bool(b) => b,
                    _ => return Err("Condition must be a boolean".to_string()),
                };

                if is_true {
                    Box::pin(self.eval(then_expr)).await
                } else {
                    Box::pin(self.eval(else_expr)).await
                }
            }

            IRNode::Sequential {
                left,
                right,
                binding,
            } => {
                // Execute left expression
                let left_result = Box::pin(self.eval(left)).await?;

                // Store result as _ for use in right expression
                self.runtime.set_var("_".to_string(), left_result.clone());

                // If there's a binding, store it as the named variable(s)
                if let Some(bind) = binding {
                    self.apply_binding(bind, &left_result)?;
                }

                // Execute right expression (which can now reference _ or bound variables)
                Box::pin(self.eval(right)).await
            }

            // ===== Parallel Composition =====
            IRNode::Parallel { exprs, binding } => {
                // Execute all expressions and collect into a list
                //
                // NOTE: Despite the || operator suggesting parallelism, we currently evaluate
                // sequentially due to fundamental architecture limitations:
                // - BuiltinFunctions contains RefCell<duckdb::Connection> which is !Sync
                // - This prevents sharing across threads even with Arc<Mutex<>>
                // - True parallelism would require refactoring all internal state to be thread-safe
                //
                // For I/O-bound workloads (LLM/HTTP calls), async concurrency within a single
                // task already provides good performance. True parallelism would mainly benefit
                // CPU-bound operations, which are rare in this DSL's use case.

                let mut results = Vec::new();
                for expr in exprs {
                    let result = Box::pin(self.eval(expr)).await?;
                    results.push(result);
                }

                // Return single value if only one expression, otherwise return list
                let result_value = if results.len() == 1 {
                    results[0].clone()
                } else {
                    Value::List(results.clone())
                };

                // Store result as _
                self.runtime.set_var("_".to_string(), result_value.clone());

                // Handle binding if present
                if let Some(bind) = binding {
                    self.apply_binding(bind, &result_value)?;
                }

                Ok(result_value)
            }

            // ===== Function Calls =====
            IRNode::FunctionCall { name, args } => {
                // Check for overloaded function first (function groups)
                if let Some(func_group) = self.runtime.function_groups.get(name).cloned() {
                    // Overloaded function - try pattern matching
                    self.call_overloaded_function(&func_group, args).await
                } else if let Some(func_def) = self.runtime.functions.get(name).cloned() {
                    // Regular user-defined function
                    self.call_user_function(&func_def, args).await
                } else {
                    // Evaluate all arguments
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(Box::pin(self.eval(arg)).await?);
                    }

                    // Call the builtin function
                    self.builtins
                        .call(name, arg_values)
                        .await
                        .map_err(|e| e.to_string())
                }
            }

            // ===== Pattern Matching (Phase 10B) =====
            IRNode::Match { scrutinee, cases } => {
                let value = Box::pin(self.eval(scrutinee)).await?;

                // Save current variable scope
                let saved_vars = self.runtime.vars.clone();

                for case in cases {
                    // Check if pattern matches
                    if PatternMatcher::matches(&case.pattern, &value) {
                        // Check guard if present
                        if let Some(guard) = &case.guard {
                            // Temporarily add bindings for guard evaluation
                            let bindings = PatternMatcher::extract_bindings(&case.pattern, &value)
                                .map_err(|e| format!("Pattern binding error: {}", e))?;

                            for (name, val) in &bindings {
                                self.runtime.set_var(name.clone(), val.clone());
                            }

                            let guard_result = Box::pin(self.eval(guard)).await?;

                            // Restore variables
                            self.runtime.vars = saved_vars.clone();

                            // If guard fails, try next case
                            if !matches!(guard_result, Value::Bool(true)) {
                                continue;
                            }
                        }

                        // Extract bindings and add to scope
                        let bindings = PatternMatcher::extract_bindings(&case.pattern, &value)
                            .map_err(|e| format!("Pattern binding error: {}", e))?;

                        for (name, val) in bindings {
                            self.runtime.set_var(name, val);
                        }

                        // Execute body
                        let result = Box::pin(self.eval(&case.body)).await;

                        // Restore variables
                        self.runtime.vars = saved_vars;

                        return result;
                    }
                }

                Err("No matching pattern in match expression".to_string())
            }

            // ===== Type Instantiation (TODO) =====
            IRNode::TypeInstantiation { type_name, fields: _ } => {
                // TODO: Type instantiation not yet implemented in interpreter
                Err(format!(
                    "Type instantiation for '{}' not yet supported",
                    type_name
                ))
            }

            // ===== Future variants (not yet in grammar) =====
            _ => Err("Unsupported IR node variant (future feature)".to_string()),
        }
    }

    // ===== Helper Methods =====

    /// Apply a binding to a value (store in variables)
    fn apply_binding(&mut self, binding: &IRBinding, value: &Value) -> Result<(), String> {
        match binding {
            IRBinding::Single(name) => {
                self.runtime.set_var(name.clone(), value.clone());
                Ok(())
            }
            IRBinding::List(names) => {
                // Destructure list into named variables
                if let Value::List(items) = value {
                    for (i, name) in names.iter().enumerate() {
                        if let Some(item) = items.get(i) {
                            self.runtime.set_var(name.clone(), item.clone());
                        } else {
                            return Err(format!(
                                "Not enough items to destructure: expected at least {}, got {}",
                                names.len(),
                                items.len()
                            ));
                        }
                    }
                    Ok(())
                } else {
                    Err(format!(
                        "Cannot destructure non-list value: got {}",
                        value.type_name()
                    ))
                }
            }
        }
    }

    /// Interpolate a template string by evaluating all interpolations
    async fn interpolate_template(
        &mut self,
        segments: &[IRTemplateSegment],
    ) -> Result<String, String> {
        let mut result = String::new();

        for segment in segments {
            match segment {
                IRTemplateSegment::Text(text) => {
                    result.push_str(text);
                }
                IRTemplateSegment::Interpolation(expr_str) => {
                    // Parse the expression string and evaluate it
                    let expr = dsl_core::parse_expr(expr_str)
                        .map_err(|e| format!("Failed to parse interpolation: {}", e))?;
                    let ir_node = dsl_core::compile_expr(&expr)
                        .map_err(|e| format!("Failed to compile interpolation: {}", e))?;
                    let value = Box::pin(self.eval(&ir_node)).await?;
                    // Convert value to string - use to_prompt_string for full data
                    let str_value = value.to_prompt_string();
                    result.push_str(&str_value);
                }
            }
        }

        Ok(result)
    }

    /// Access a field in a map value
    fn access_field(&self, value: &Value, field: &str) -> Result<Value, String> {
        match value {
            Value::Map(m) => m
                .get(field)
                .cloned()
                .ok_or_else(|| format!("Field '{}' not found", field)),
            _ => Err(format!("Cannot access field on {}", value.type_name())),
        }
    }

    /// Access an index in a list or map
    fn access_index(&self, value: &Value, index: &Value) -> Result<Value, String> {
        match (value, index) {
            (Value::List(items), Value::Int(i)) => {
                if *i < 0 {
                    return Err("Index cannot be negative".to_string());
                }
                items
                    .get(*i as usize)
                    .cloned()
                    .ok_or_else(|| format!("Index {} out of bounds", i))
            }
            (Value::Map(map), Value::String(key)) => map
                .get(key)
                .cloned()
                .ok_or_else(|| format!("Key '{}' not found", key)),
            _ => Err(format!(
                "Invalid index operation: {} indexed by {}",
                value.type_name(),
                index.type_name()
            )),
        }
    }

    /// Apply a binary operation to two values
    fn apply_binary_op(&self, op: &str, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                match op {
                    // Arithmetic
                    "+" => Ok(Value::Int(a + b)),
                    "-" => Ok(Value::Int(a - b)),
                    "*" => Ok(Value::Int(a * b)),
                    "/" => {
                        if b == 0 {
                            return Err("Division by zero".to_string());
                        }
                        Ok(Value::Int(a / b))
                    }
                    // Comparison
                    "==" => Ok(Value::Bool(a == b)),
                    "!=" => Ok(Value::Bool(a != b)),
                    "<" => Ok(Value::Bool(a < b)),
                    ">" => Ok(Value::Bool(a > b)),
                    "<=" => Ok(Value::Bool(a <= b)),
                    ">=" => Ok(Value::Bool(a >= b)),
                    _ => Err(format!("Unknown operator: {}", op)),
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                match op {
                    // Arithmetic
                    "+" => Ok(Value::Float(a + b)),
                    "-" => Ok(Value::Float(a - b)),
                    "*" => Ok(Value::Float(a * b)),
                    "/" => Ok(Value::Float(a / b)),
                    // Comparison
                    "==" => Ok(Value::Bool(a == b)),
                    "!=" => Ok(Value::Bool(a != b)),
                    "<" => Ok(Value::Bool(a < b)),
                    ">" => Ok(Value::Bool(a > b)),
                    "<=" => Ok(Value::Bool(a <= b)),
                    ">=" => Ok(Value::Bool(a >= b)),
                    _ => Err(format!("Unknown operator: {}", op)),
                }
            }
            // Mixed Int/Float operations - promote to Float
            (Value::Int(a), Value::Float(b)) => {
                match op {
                    // Arithmetic
                    "+" => Ok(Value::Float(a as f64 + b)),
                    "-" => Ok(Value::Float(a as f64 - b)),
                    "*" => Ok(Value::Float(a as f64 * b)),
                    "/" => Ok(Value::Float(a as f64 / b)),
                    // Comparison
                    "==" => Ok(Value::Bool((a as f64) == b)),
                    "!=" => Ok(Value::Bool((a as f64) != b)),
                    "<" => Ok(Value::Bool((a as f64) < b)),
                    ">" => Ok(Value::Bool((a as f64) > b)),
                    "<=" => Ok(Value::Bool((a as f64) <= b)),
                    ">=" => Ok(Value::Bool((a as f64) >= b)),
                    _ => Err(format!("Unknown operator: {}", op)),
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                match op {
                    // Arithmetic
                    "+" => Ok(Value::Float(a + b as f64)),
                    "-" => Ok(Value::Float(a - b as f64)),
                    "*" => Ok(Value::Float(a * b as f64)),
                    "/" => Ok(Value::Float(a / b as f64)),
                    // Comparison
                    "==" => Ok(Value::Bool(a == (b as f64))),
                    "!=" => Ok(Value::Bool(a != (b as f64))),
                    "<" => Ok(Value::Bool(a < (b as f64))),
                    ">" => Ok(Value::Bool(a > (b as f64))),
                    "<=" => Ok(Value::Bool(a <= (b as f64))),
                    ">=" => Ok(Value::Bool(a >= (b as f64))),
                    _ => Err(format!("Unknown operator: {}", op)),
                }
            }
            // String operations
            (Value::String(a), Value::String(b)) => {
                match op {
                    // Concatenation
                    "+" => Ok(Value::String(format!("{}{}", a, b))),
                    // Comparison
                    "==" => Ok(Value::Bool(a == b)),
                    "!=" => Ok(Value::Bool(a != b)),
                    "<" => Ok(Value::Bool(a < b)),
                    ">" => Ok(Value::Bool(a > b)),
                    "<=" => Ok(Value::Bool(a <= b)),
                    ">=" => Ok(Value::Bool(a >= b)),
                    _ => Err(format!("Operator '{}' not supported for strings", op)),
                }
            }
            // Boolean operations
            (Value::Bool(a), Value::Bool(b)) => {
                match op {
                    "==" => Ok(Value::Bool(a == b)),
                    "!=" => Ok(Value::Bool(a != b)),
                    "&&" | "and" => Ok(Value::Bool(a && b)),
                    "||" | "or" => Ok(Value::Bool(a || b)),
                    _ => Err(format!("Operator '{}' not supported for booleans", op)),
                }
            }
            (left_val, right_val) => Err(format!(
                "Type mismatch in operation: {} {} {}",
                left_val.type_name(),
                op,
                right_val.type_name()
            )),
        }
    }

    /// Call a user-defined function
    async fn call_user_function(
        &mut self,
        func: &IRFunction,
        args: &[IRNode],
    ) -> Result<Value, String> {
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(Box::pin(self.eval(arg)).await?);
        }

        // Check argument count
        if arg_values.len() != func.params.len() {
            return Err(format!(
                "Function '{}' expects {} arguments, got {}",
                func.name,
                func.params.len(),
                arg_values.len()
            ));
        }

        // Save current variable state
        let saved_vars = self.runtime.vars.clone();

        // Bind arguments to parameters
        for (param_name, arg_value) in func.params.iter().zip(arg_values.iter()) {
            self.runtime.set_var(param_name.clone(), arg_value.clone());
        }

        // Execute based on function type
        let result = match &func.execution {
            IRExecution::Expression { body } => {
                // Simple expression evaluation (for helper functions, recursion, etc.)
                Box::pin(self.eval(body)).await?
            }
            IRExecution::LLM {
                prompt,
                model,
                base_url,
                api_key_env,
                temperature: _,
            } => {
                self.execute_llm_function(
                    func,
                    prompt,
                    model.clone(),
                    base_url.clone(),
                    api_key_env.clone(),
                    &arg_values,
                )
                .await?
            }
            IRExecution::HTTP {
                method,
                url,
                params,
                headers,
                body,
            } => {
                self.execute_http_function(
                    func,
                    method,
                    url,
                    params,
                    headers,
                    body,
                    &arg_values,
                )
                .await?
            }
            IRExecution::SQL { query } => {
                self.execute_sql_function(func, query, &arg_values).await?
            }
            IRExecution::HTTPWithLLM {
                http_method,
                http_url,
                http_params,
                http_headers,
                llm_prompt,
                llm_model,
                llm_base_url,
                llm_api_key_env,
                llm_temperature: _,
            } => {
                // First execute HTTP request
                let http_result = self
                    .execute_http_function(
                        func,
                        http_method,
                        http_url,
                        http_params,
                        http_headers,
                        &None,
                        &arg_values,
                    )
                    .await?;

                // Then pass result to LLM
                self.execute_llm_with_input(
                    func,
                    llm_prompt,
                    llm_model.clone(),
                    llm_base_url.clone(),
                    llm_api_key_env.clone(),
                    &http_result,
                    &arg_values,
                )
                .await?
            }
        };

        // Restore variable state
        self.runtime.vars = saved_vars;

        Ok(result)
    }

    /// Execute an LLM-based function
    async fn execute_llm_function(
        &mut self,
        func: &IRFunction,
        prompt_template: &str,
        model: Option<String>,
        base_url: Option<String>,
        api_key_env: Option<String>,
        arg_values: &[Value],
    ) -> Result<Value, String> {
        // Interpolate template variables
        let prompt = self.interpolate_string_template(prompt_template, &func.params, arg_values)?;

        // Call the appropriate builtin function based on return type
        if let Some(return_type) = &func.return_type {
            // For structured types, use ExtractAs with the type name or serialized type
            let type_identifier = return_type.to_string();

            self.builtins
                .extract_as_with_config(prompt, type_identifier, model, base_url, api_key_env)
                .await
                .map_err(|e| e.to_string())
        } else {
            // Simple string output - use Ask
            self.builtins
                .ask_with_config(prompt, model, base_url, api_key_env)
                .await
                .map_err(|e| e.to_string())
        }
    }

    /// Execute LLM with pre-fetched input data
    #[allow(clippy::too_many_arguments)]
    async fn execute_llm_with_input(
        &mut self,
        func: &IRFunction,
        prompt_template: &str,
        model: Option<String>,
        base_url: Option<String>,
        api_key_env: Option<String>,
        input_data: &Value,
        arg_values: &[Value],
    ) -> Result<Value, String> {
        // Interpolate template with args
        let mut prompt =
            self.interpolate_string_template(prompt_template, &func.params, arg_values)?;

        // Append the input data to the prompt
        prompt.push_str("\n\nData to analyze:\n");
        prompt.push_str(&input_data.to_prompt_string());

        // Call the appropriate builtin function based on return type
        if let Some(return_type) = &func.return_type {
            let type_identifier = return_type.to_string();

            self.builtins
                .extract_as_with_config(prompt, type_identifier, model, base_url, api_key_env)
                .await
                .map_err(|e| e.to_string())
        } else {
            self.builtins
                .ask_with_config(prompt, model, base_url, api_key_env)
                .await
                .map_err(|e| e.to_string())
        }
    }

    /// Execute an HTTP request
    #[allow(clippy::too_many_arguments)]
    async fn execute_http_function(
        &mut self,
        func: &IRFunction,
        method: &str,
        url_template: &str,
        params: &Option<std::collections::HashMap<String, String>>,
        headers: &Option<std::collections::HashMap<String, String>>,
        body: &Option<String>,
        arg_values: &[Value],
    ) -> Result<Value, String> {
        // Interpolate URL template with arguments
        let url = self.interpolate_string_template(url_template, &func.params, arg_values)?;

        // Create HTTP client with User-Agent header
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; DSL-REPL/1.0)")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {:#?}", e))?;

        // Build the request
        let mut request = match method.to_uppercase().as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            "PATCH" => client.patch(&url),
            "HEAD" => client.head(&url),
            _ => return Err(format!("Unsupported HTTP method: {}", method)),
        };

        // Add query parameters if provided
        if let Some(params) = params {
            request = request.query(params);
        }

        // Add headers if provided
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        // Add body if provided (for POST, PUT, PATCH)
        if let Some(body_content) = body {
            // Interpolate body template
            let interpolated_body =
                self.interpolate_string_template(body_content, &func.params, arg_values)?;

            // Try to parse as JSON, otherwise send as plain text
            if let Ok(json_body) = serde_json::from_str::<serde_json::Value>(&interpolated_body) {
                request = request.json(&json_body);
            } else {
                request = request.body(interpolated_body);
            }
        }

        // Execute the request
        let response = request
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        // Check if the response is successful
        let status = response.status();
        if !status.is_success() {
            return Err(format!(
                "HTTP request failed with status: {} {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            ));
        }

        // Get the response body as text first
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        // Try to parse as JSON, otherwise return as string
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
            Ok(Value::from_json(json_value))
        } else {
            // If not JSON, return as string
            Ok(Value::String(response_text))
        }
    }

    /// Execute a SQL query
    async fn execute_sql_function(
        &mut self,
        func: &IRFunction,
        query_template: &str,
        arg_values: &[Value],
    ) -> Result<Value, String> {
        // Interpolate query template with arguments
        let query = self.interpolate_string_template(query_template, &func.params, arg_values)?;

        // Use existing SQL builtin
        self.builtins
            .call("sql", vec![Value::String(query)])
            .await
            .map_err(|e| e.to_string())
    }

    /// Interpolate string template with parameter values
    fn interpolate_string_template(
        &self,
        template: &str,
        params: &[String],
        arg_values: &[Value],
    ) -> Result<String, String> {
        let mut result = template.to_string();

        // Replace ${param} with actual values
        for (param_name, arg_value) in params.iter().zip(arg_values.iter()) {
            let placeholder = format!("${{{}}}", param_name);
            let value_str = arg_value.to_prompt_string();
            result = result.replace(&placeholder, &value_str);
        }

        Ok(result)
    }

    /// Call an overloaded function with pattern matching
    async fn call_overloaded_function(
        &mut self,
        func_group: &dsl_ir::IRFunctionGroup,
        args: &[IRNode],
    ) -> Result<Value, String> {
        // Evaluate all arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(Box::pin(self.eval(arg)).await?);
        }

        // Save current variable scope
        let saved_vars = self.runtime.vars.clone();

        // Try each clause in order
        for clause in &func_group.clauses {
            // Check if patterns match arguments
            if clause.param_patterns.len() != arg_values.len() {
                continue;
            }

            let mut all_match = true;
            let mut bindings = std::collections::HashMap::new();

            // Check if all patterns match
            for (pattern, value) in clause.param_patterns.iter().zip(arg_values.iter()) {
                if !PatternMatcher::matches(pattern, value) {
                    all_match = false;
                    break;
                }

                // Extract bindings from this pattern
                let pattern_bindings = PatternMatcher::extract_bindings(pattern, value)
                    .map_err(|e| format!("Pattern binding error: {}", e))?;
                bindings.extend(pattern_bindings);
            }

            if !all_match {
                continue;
            }

            // Add bindings to scope
            for (name, val) in &bindings {
                self.runtime.set_var(name.clone(), val.clone());
            }

            // Check guard if present
            if let Some(guard) = &clause.guard {
                let guard_result = Box::pin(self.eval(guard)).await?;
                if !matches!(guard_result, Value::Bool(true)) {
                    // Restore and try next clause
                    self.runtime.vars = saved_vars.clone();
                    continue;
                }
            }

            // Execute body
            let result = Box::pin(self.eval(&clause.body)).await;

            // Restore scope
            self.runtime.vars = saved_vars;

            return result;
        }

        Err(format!(
            "No matching clause for function '{}' with {} arguments",
            func_group.name,
            arg_values.len()
        ))
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new().expect("Failed to create default interpreter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_interpreter() -> Interpreter {
        Interpreter::new().expect("Failed to create interpreter")
    }

    #[tokio::test]
    async fn test_string_comparison_equal() {
        let mut interp = create_test_interpreter();
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::String("hello".to_string())),
            op: "==".to_string(),
            right: Box::new(IRNode::String("hello".to_string())),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_string_comparison_not_equal() {
        let mut interp = create_test_interpreter();
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::String("hello".to_string())),
            op: "==".to_string(),
            right: Box::new(IRNode::String("world".to_string())),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[tokio::test]
    async fn test_string_not_equal_operator() {
        let mut interp = create_test_interpreter();
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::String("hello".to_string())),
            op: "!=".to_string(),
            right: Box::new(IRNode::String("world".to_string())),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_string_less_than() {
        let mut interp = create_test_interpreter();
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::String("apple".to_string())),
            op: "<".to_string(),
            right: Box::new(IRNode::String("banana".to_string())),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_int_comparison() {
        let mut interp = create_test_interpreter();
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::Int(5)),
            op: "<".to_string(),
            right: Box::new(IRNode::Int(10)),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_int_equality() {
        let mut interp = create_test_interpreter();
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::Int(42)),
            op: "==".to_string(),
            right: Box::new(IRNode::Int(42)),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_bool_and() {
        let mut interp = create_test_interpreter();
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::Bool(true)),
            op: "&&".to_string(),
            right: Box::new(IRNode::Bool(true)),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_bool_or() {
        let mut interp = create_test_interpreter();
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::Bool(false)),
            op: "||".to_string(),
            right: Box::new(IRNode::Bool(true)),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_mixed_int_float_comparison() {
        let mut interp = create_test_interpreter();
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::Int(5)),
            op: "==".to_string(),
            right: Box::new(IRNode::Float(5.0)),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_string_concatenation() {
        let mut interp = create_test_interpreter();
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::String("hello".to_string())),
            op: "+".to_string(),
            right: Box::new(IRNode::String(" world".to_string())),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[tokio::test]
    async fn test_conditional_true_branch() {
        let mut interp = create_test_interpreter();
        let node = IRNode::Conditional {
            condition: Box::new(IRNode::Bool(true)),
            then_expr: Box::new(IRNode::String("yes".to_string())),
            else_expr: Box::new(IRNode::String("no".to_string())),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::String("yes".to_string()));
    }

    #[tokio::test]
    async fn test_conditional_false_branch() {
        let mut interp = create_test_interpreter();
        let node = IRNode::Conditional {
            condition: Box::new(IRNode::Bool(false)),
            then_expr: Box::new(IRNode::String("yes".to_string())),
            else_expr: Box::new(IRNode::String("no".to_string())),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::String("no".to_string()));
    }

    #[tokio::test]
    async fn test_variable_binding() {
        let mut interp = create_test_interpreter();
        interp.runtime.set_var("x".to_string(), Value::Int(42));
        let node = IRNode::Variable("x".to_string());
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_list_creation() {
        let mut interp = create_test_interpreter();
        let node = IRNode::List(vec![
            IRNode::Int(1),
            IRNode::Int(2),
            IRNode::Int(3),
        ]);
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(
            result,
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
    }

    #[tokio::test]
    async fn test_map_creation() {
        let mut interp = create_test_interpreter();
        let node = IRNode::Map(vec![
            ("name".to_string(), IRNode::String("Alice".to_string())),
            ("age".to_string(), IRNode::Int(30)),
        ]);
        let result = interp.eval(&node).await.unwrap();
        if let Value::Map(map) = result {
            assert_eq!(map.get("name"), Some(&Value::String("Alice".to_string())));
            assert_eq!(map.get("age"), Some(&Value::Int(30)));
        } else {
            panic!("Expected Map value");
        }
    }

    #[tokio::test]
    async fn test_field_access() {
        let mut interp = create_test_interpreter();
        let node = IRNode::FieldAccess {
            base: Box::new(IRNode::Map(vec![
                ("name".to_string(), IRNode::String("Bob".to_string())),
            ])),
            field: "name".to_string(),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::String("Bob".to_string()));
    }

    #[tokio::test]
    async fn test_index_access_list() {
        let mut interp = create_test_interpreter();
        let node = IRNode::IndexAccess {
            base: Box::new(IRNode::List(vec![
                IRNode::String("a".to_string()),
                IRNode::String("b".to_string()),
                IRNode::String("c".to_string()),
            ])),
            index: Box::new(IRNode::Int(1)),
        };
        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::String("b".to_string()));
    }

    #[tokio::test]
    async fn test_expression_execution_mode() {
        let mut interp = create_test_interpreter();

        // Define a simple function with expression body: double(x) = x * 2
        let func = IRFunction {
            name: "double".to_string(),
            params: vec!["x".to_string()],
            return_type: None,
            properties: std::collections::HashMap::new(),
            execution: IRExecution::Expression {
                body: Box::new(IRNode::BinaryOp {
                    left: Box::new(IRNode::Variable("x".to_string())),
                    op: "*".to_string(),
                    right: Box::new(IRNode::Int(2)),
                }),
            },
        };

        // Register the function
        interp.runtime.functions.insert("double".to_string(), func);

        // Call the function with argument 21
        let call_node = IRNode::FunctionCall {
            name: "double".to_string(),
            args: vec![IRNode::Int(21)],
        };

        let result = interp.eval(&call_node).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_expression_execution_recursive() {
        let mut interp = create_test_interpreter();

        // Define factorial function: fact(n) = if n == 0 { 1 } else { n * fact(n - 1) }
        let func = IRFunction {
            name: "fact".to_string(),
            params: vec!["n".to_string()],
            return_type: None,
            properties: std::collections::HashMap::new(),
            execution: IRExecution::Expression {
                body: Box::new(IRNode::Conditional {
                    condition: Box::new(IRNode::BinaryOp {
                        left: Box::new(IRNode::Variable("n".to_string())),
                        op: "==".to_string(),
                        right: Box::new(IRNode::Int(0)),
                    }),
                    then_expr: Box::new(IRNode::Int(1)),
                    else_expr: Box::new(IRNode::BinaryOp {
                        left: Box::new(IRNode::Variable("n".to_string())),
                        op: "*".to_string(),
                        right: Box::new(IRNode::FunctionCall {
                            name: "fact".to_string(),
                            args: vec![IRNode::BinaryOp {
                                left: Box::new(IRNode::Variable("n".to_string())),
                                op: "-".to_string(),
                                right: Box::new(IRNode::Int(1)),
                            }],
                        }),
                    }),
                }),
            },
        };

        // Register the function
        interp.runtime.functions.insert("fact".to_string(), func);

        // Call fact(5) should return 120
        let call_node = IRNode::FunctionCall {
            name: "fact".to_string(),
            args: vec![IRNode::Int(5)],
        };

        let result = interp.eval(&call_node).await.unwrap();
        assert_eq!(result, Value::Int(120));
    }

    #[tokio::test]
    async fn test_match_expression() {
        use dsl_ir::{IRMatchCase, IRPattern};

        let mut interp = create_test_interpreter();

        // match 42 { 0 => "zero", 42 => "answer", _ => "other" }
        let node = IRNode::Match {
            scrutinee: Box::new(IRNode::Int(42)),
            cases: vec![
                IRMatchCase {
                    pattern: IRPattern::Literal(Box::new(IRNode::Int(0))),
                    guard: None,
                    body: Box::new(IRNode::String("zero".to_string())),
                },
                IRMatchCase {
                    pattern: IRPattern::Literal(Box::new(IRNode::Int(42))),
                    guard: None,
                    body: Box::new(IRNode::String("answer".to_string())),
                },
                IRMatchCase {
                    pattern: IRPattern::Any,
                    guard: None,
                    body: Box::new(IRNode::String("other".to_string())),
                },
            ],
        };

        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::String("answer".to_string()));
    }

    #[tokio::test]
    async fn test_match_with_binding() {
        use dsl_ir::{IRMatchCase, IRPattern};

        let mut interp = create_test_interpreter();

        // match 42 { x => x * 2 }
        let node = IRNode::Match {
            scrutinee: Box::new(IRNode::Int(42)),
            cases: vec![IRMatchCase {
                pattern: IRPattern::Variable("x".to_string()),
                guard: None,
                body: Box::new(IRNode::BinaryOp {
                    left: Box::new(IRNode::Variable("x".to_string())),
                    op: "*".to_string(),
                    right: Box::new(IRNode::Int(2)),
                }),
            }],
        };

        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Int(84));
    }

    #[tokio::test]
    async fn test_match_with_guard() {
        use dsl_ir::{IRMatchCase, IRPattern};

        let mut interp = create_test_interpreter();

        // match 42 { x if x > 40 => "big", x => "small" }
        let node = IRNode::Match {
            scrutinee: Box::new(IRNode::Int(42)),
            cases: vec![
                IRMatchCase {
                    pattern: IRPattern::Variable("x".to_string()),
                    guard: Some(Box::new(IRNode::BinaryOp {
                        left: Box::new(IRNode::Variable("x".to_string())),
                        op: ">".to_string(),
                        right: Box::new(IRNode::Int(40)),
                    })),
                    body: Box::new(IRNode::String("big".to_string())),
                },
                IRMatchCase {
                    pattern: IRPattern::Variable("x".to_string()),
                    guard: None,
                    body: Box::new(IRNode::String("small".to_string())),
                },
            ],
        };

        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::String("big".to_string()));
    }

    #[tokio::test]
    async fn test_function_overloading_factorial() {
        use dsl_ir::{IRFunctionClause, IRFunctionGroup, IRPattern};

        let mut interp = create_test_interpreter();

        // Define factorial with two clauses:
        // function factorial(0) { 1 }
        // function factorial(n) { n * factorial(n - 1) }
        let func_group = IRFunctionGroup {
            name: "factorial".to_string(),
            clauses: vec![
                // Base case: factorial(0) = 1
                IRFunctionClause {
                    param_patterns: vec![IRPattern::Literal(Box::new(IRNode::Int(0)))],
                    guard: None,
                    body: Box::new(IRNode::Int(1)),
                },
                // Recursive case: factorial(n) = n * factorial(n - 1)
                IRFunctionClause {
                    param_patterns: vec![IRPattern::Variable("n".to_string())],
                    guard: None,
                    body: Box::new(IRNode::BinaryOp {
                        left: Box::new(IRNode::Variable("n".to_string())),
                        op: "*".to_string(),
                        right: Box::new(IRNode::FunctionCall {
                            name: "factorial".to_string(),
                            args: vec![IRNode::BinaryOp {
                                left: Box::new(IRNode::Variable("n".to_string())),
                                op: "-".to_string(),
                                right: Box::new(IRNode::Int(1)),
                            }],
                        }),
                    }),
                },
            ],
            return_type: None,
        };

        // Register the function group
        interp.runtime.function_groups.insert("factorial".to_string(), func_group);

        // Test factorial(0) = 1
        let call_0 = IRNode::FunctionCall {
            name: "factorial".to_string(),
            args: vec![IRNode::Int(0)],
        };
        let result_0 = interp.eval(&call_0).await.unwrap();
        assert_eq!(result_0, Value::Int(1));

        // Test factorial(5) = 120
        let call_5 = IRNode::FunctionCall {
            name: "factorial".to_string(),
            args: vec![IRNode::Int(5)],
        };
        let result_5 = interp.eval(&call_5).await.unwrap();
        assert_eq!(result_5, Value::Int(120));
    }

    #[tokio::test]
    async fn test_function_overloading_list_length() {
        use dsl_ir::{IRFunctionClause, IRFunctionGroup, IRPattern};

        let mut interp = create_test_interpreter();

        // Define length with two clauses:
        // function length([]) { 0 }
        // function length([_, ...tail]) { 1 + length(tail) }
        let func_group = IRFunctionGroup {
            name: "length".to_string(),
            clauses: vec![
                // Base case: length([]) = 0
                IRFunctionClause {
                    param_patterns: vec![IRPattern::List {
                        patterns: vec![],
                        rest: None,
                    }],
                    guard: None,
                    body: Box::new(IRNode::Int(0)),
                },
                // Recursive case: length([_, ...tail]) = 1 + length(tail)
                IRFunctionClause {
                    param_patterns: vec![IRPattern::List {
                        patterns: vec![IRPattern::Any],
                        rest: Some("tail".to_string()),
                    }],
                    guard: None,
                    body: Box::new(IRNode::BinaryOp {
                        left: Box::new(IRNode::Int(1)),
                        op: "+".to_string(),
                        right: Box::new(IRNode::FunctionCall {
                            name: "length".to_string(),
                            args: vec![IRNode::Variable("tail".to_string())],
                        }),
                    }),
                },
            ],
            return_type: None,
        };

        // Register the function group
        interp.runtime.function_groups.insert("length".to_string(), func_group);

        // Test length([]) = 0
        let call_empty = IRNode::FunctionCall {
            name: "length".to_string(),
            args: vec![IRNode::List(vec![])],
        };
        let result_empty = interp.eval(&call_empty).await.unwrap();
        assert_eq!(result_empty, Value::Int(0));

        // Test length([1, 2, 3]) = 3
        let call_list = IRNode::FunctionCall {
            name: "length".to_string(),
            args: vec![IRNode::List(vec![
                IRNode::Int(1),
                IRNode::Int(2),
                IRNode::Int(3),
            ])],
        };
        let result_list = interp.eval(&call_list).await.unwrap();
        assert_eq!(result_list, Value::Int(3));
    }

    #[tokio::test]
    async fn test_function_overloading_with_guard() {
        use dsl_ir::{IRFunctionClause, IRFunctionGroup, IRPattern};

        let mut interp = create_test_interpreter();

        // Define classify with guards:
        // function classify(n) if n < 0 { "negative" }
        // function classify(0) { "zero" }
        // function classify(n) if n > 0 { "positive" }
        let func_group = IRFunctionGroup {
            name: "classify".to_string(),
            clauses: vec![
                // classify(n) if n < 0 = "negative"
                IRFunctionClause {
                    param_patterns: vec![IRPattern::Variable("n".to_string())],
                    guard: Some(Box::new(IRNode::BinaryOp {
                        left: Box::new(IRNode::Variable("n".to_string())),
                        op: "<".to_string(),
                        right: Box::new(IRNode::Int(0)),
                    })),
                    body: Box::new(IRNode::String("negative".to_string())),
                },
                // classify(0) = "zero"
                IRFunctionClause {
                    param_patterns: vec![IRPattern::Literal(Box::new(IRNode::Int(0)))],
                    guard: None,
                    body: Box::new(IRNode::String("zero".to_string())),
                },
                // classify(n) if n > 0 = "positive"
                IRFunctionClause {
                    param_patterns: vec![IRPattern::Variable("n".to_string())],
                    guard: Some(Box::new(IRNode::BinaryOp {
                        left: Box::new(IRNode::Variable("n".to_string())),
                        op: ">".to_string(),
                        right: Box::new(IRNode::Int(0)),
                    })),
                    body: Box::new(IRNode::String("positive".to_string())),
                },
            ],
            return_type: None,
        };

        // Register the function group
        interp.runtime.function_groups.insert("classify".to_string(), func_group);

        // Test classify(-5) = "negative"
        let call_neg = IRNode::FunctionCall {
            name: "classify".to_string(),
            args: vec![IRNode::Int(-5)],
        };
        let result_neg = interp.eval(&call_neg).await.unwrap();
        assert_eq!(result_neg, Value::String("negative".to_string()));

        // Test classify(0) = "zero"
        let call_zero = IRNode::FunctionCall {
            name: "classify".to_string(),
            args: vec![IRNode::Int(0)],
        };
        let result_zero = interp.eval(&call_zero).await.unwrap();
        assert_eq!(result_zero, Value::String("zero".to_string()));

        // Test classify(42) = "positive"
        let call_pos = IRNode::FunctionCall {
            name: "classify".to_string(),
            args: vec![IRNode::Int(42)],
        };
        let result_pos = interp.eval(&call_pos).await.unwrap();
        assert_eq!(result_pos, Value::String("positive".to_string()));
    }
}
