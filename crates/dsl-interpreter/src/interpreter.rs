use crate::builtins::BuiltinFunctions;
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

        // Rebuild BAML runtime with new types
        interpreter.builtins.rebuild_runtime(&interpreter.runtime.types)?;

        Ok(interpreter)
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
                // Check if it's a user-defined function
                if let Some(func_def) = self.runtime.functions.get(name).cloned() {
                    // User-defined function
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
}
