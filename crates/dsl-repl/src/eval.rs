use crate::builtin::BuiltinFunctions;
use crate::parser::{
    is_command, parse_enum_definition, parse_expr, parse_expr_with_binding,
    parse_function_definition, parse_type_definition, Expr, FunctionDef, FunctionExecution, TemplateSegment,
};
use crate::types::TypeRegistry;
use crate::value::Value;
use indexmap::IndexMap;
use std::collections::HashMap;

pub struct Evaluator {
    pub vars: HashMap<String, Value>,
    pub types: TypeRegistry,
    pub functions: HashMap<String, FunctionDef>,
    builtins: BuiltinFunctions,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            types: TypeRegistry::new(),
            functions: HashMap::new(),
            builtins: BuiltinFunctions::new().unwrap_or_else(|_| {
                // If builtin initialization fails, we still want to work
                // (e.g., if OPENAI_API_KEY is not set)
                BuiltinFunctions::new().unwrap()
            }),
        }
    }

    pub async fn eval(&mut self, input: &str) -> Result<(Value, Option<String>), String> {
        let input = input.trim();

        // Check for special commands
        if is_command(input) {
            let full_cmd = input.trim();
            let parts: Vec<&str> = full_cmd[1..].splitn(2, ' ').collect();
            let cmd = parts[0];
            let args = if parts.len() > 1 {
                Some(parts[1])
            } else {
                None
            };
            return Ok((self.handle_command(cmd, args)?, None));
        }

        // Check for type definitions
        if input.starts_with("type ") {
            let class = parse_type_definition(input)?;
            let name = class.name.clone();
            self.types.register_class(class);

            // Rebuild the BAML runtime with the new type
            self.builtins
                .rebuild_runtime(&self.types)
                .map_err(|e| format!("Failed to rebuild runtime: {}", e))?;

            return Ok((Value::String(format!("Type '{}' registered", name)), None));
        }

        // Check for enum definitions
        if input.starts_with("enum ") {
            let enum_def = parse_enum_definition(input)?;
            let name = enum_def.name.clone();
            self.types.register_enum(enum_def);

            // Rebuild the BAML runtime with the new enum
            self.builtins
                .rebuild_runtime(&self.types)
                .map_err(|e| format!("Failed to rebuild runtime: {}", e))?;

            return Ok((Value::String(format!("Enum '{}' registered", name)), None));
        }

        // Check for function definitions
        if input.starts_with("def ") {
            let func_def = parse_function_definition(input)?;
            let name = func_def.name.clone();
            self.functions.insert(name.clone(), func_def);
            return Ok((Value::String(format!("Function '{}' defined", name)), None));
        }

        // Parse the expression using Pest parser
        // This returns both the expression and any top-level binding
        let (expr, top_level_binding) = parse_expr_with_binding(input)?;

        // Evaluate the expression
        let value = self.eval_expr(&expr).await?;

        // Store last result as '_'
        self.vars.insert("_".to_string(), value.clone());

        // Handle top-level binding if present
        let binding_name = if let Some(binding) = top_level_binding {
            match binding {
                crate::parser::Binding::Single(name) => {
                    self.vars.insert(name.clone(), value.clone());
                    Some(name)
                }
                crate::parser::Binding::List(names) => {
                    // Destructure list into named variables
                    if let Value::List(items) = &value {
                        for (i, name) in names.iter().enumerate() {
                            if let Some(item) = items.get(i) {
                                self.vars.insert(name.clone(), item.clone());
                            } else {
                                return Err(format!(
                                    "Not enough items to destructure: expected at least {}, got {}",
                                    names.len(),
                                    items.len()
                                ));
                            }
                        }
                        Some(format!("[{}]", names.join(", ")))
                    } else {
                        return Err(format!(
                            "Cannot destructure non-list value: got {}",
                            value.type_name()
                        ));
                    }
                }
            }
        } else {
            None
        };

        Ok((value, binding_name))
    }

    async fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::TemplateString(segments) => {
                let interpolated = self.interpolate_template(segments).await?;
                Ok(Value::String(interpolated))
            }
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::List(items) => {
                let mut values = Vec::new();
                for item in items {
                    values.push(Box::pin(self.eval_expr(item)).await?);
                }
                Ok(Value::List(values))
            }
            Expr::Map(entries) => {
                let mut map = IndexMap::new();
                for (key, value_expr) in entries {
                    let value = Box::pin(self.eval_expr(value_expr)).await?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Map(map))
            }
            Expr::Variable(name) => self
                .vars
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Variable '{}' not found", name)),
            Expr::FunctionCall { name, args } => {
                // Check if it's a user-defined function
                if let Some(func_def) = self.functions.get(name).cloned() {
                    // User-defined function
                    self.call_user_function(&func_def, args).await
                } else {
                    // Evaluate all arguments
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(Box::pin(self.eval_expr(arg)).await?);
                    }

                    // Call the builtin function
                    self.builtins
                        .call(name, arg_values)
                        .await
                        .map_err(|e| e.to_string())
                }
            }
            Expr::FieldAccess { base, field } => {
                let base_value = Box::pin(self.eval_expr(base)).await?;
                self.access_field(&base_value, field)
            }
            Expr::IndexAccess { base, index } => {
                let base_value = Box::pin(self.eval_expr(base)).await?;
                let index_value = Box::pin(self.eval_expr(index)).await?;

                match index_value {
                    Value::Int(idx) => {
                        if idx < 0 {
                            return Err("Index cannot be negative".to_string());
                        }
                        self.access_index(&base_value, idx as usize)
                    }
                    _ => Err(format!(
                        "Index must be an integer, got {}",
                        index_value.type_name()
                    )),
                }
            }
            Expr::BinaryOp { left, op, right } => {
                let left_val = Box::pin(self.eval_expr(left)).await?;
                let right_val = Box::pin(self.eval_expr(right)).await?;
                self.apply_op(&left_val, op, &right_val)
            }
            Expr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond_val = Box::pin(self.eval_expr(condition)).await?;
                let is_true = match cond_val {
                    Value::Bool(b) => b,
                    _ => return Err("Condition must be a boolean".to_string()),
                };

                if is_true {
                    Box::pin(self.eval_expr(then_expr)).await
                } else {
                    Box::pin(self.eval_expr(else_expr)).await
                }
            }
            Expr::Sequential {
                left,
                right,
                binding,
            } => {
                // Check if left and right are the same (pointer equality)
                // This happens with simple bindings like "expr as name"
                let is_simple_binding = std::ptr::eq(left.as_ref(), right.as_ref());

                // Execute left expression
                let left_result = Box::pin(self.eval_expr(left)).await?;

                // Store result as _ for use in right expression
                self.vars.insert("_".to_string(), left_result.clone());

                // If there's a binding, store it as the named variable(s)
                if let Some(bind) = binding {
                    match bind {
                        crate::parser::Binding::Single(name) => {
                            self.vars.insert(name.clone(), left_result.clone());
                        }
                        crate::parser::Binding::List(names) => {
                            // Destructure list into named variables
                            if let Value::List(items) = &left_result {
                                for (i, name) in names.iter().enumerate() {
                                    if let Some(item) = items.get(i) {
                                        self.vars.insert(name.clone(), item.clone());
                                    } else {
                                        return Err(format!(
                                            "Not enough items to destructure: expected at least {}, got {}",
                                            names.len(),
                                            items.len()
                                        ));
                                    }
                                }
                            } else {
                                return Err(format!(
                                    "Cannot destructure non-list value: got {}",
                                    left_result.type_name()
                                ));
                            }
                        }
                    }
                }

                // If this is a simple binding (left == right), just return the left result
                // Don't evaluate right again as it would cause infinite recursion
                if is_simple_binding {
                    return Ok(left_result);
                }

                // Execute right expression (which can now reference _ or bound variables)
                Box::pin(self.eval_expr(right)).await
            }
            Expr::Parallel { exprs, binding } => {
                // Execute all expressions and collect into a list
                // Note: Due to Rust's borrow checker and Send trait limitations,
                // we evaluate sequentially but still collect results as a list
                // This maintains the correct semantics for the || operator
                //
                // The Parallel node is also used for simple bindings like "expr as name"
                // (with a single expression) to avoid double evaluation that would happen
                // if we wrapped it in a Sequential node with left == right.

                let mut results = Vec::new();
                for expr in exprs {
                    let result = Box::pin(self.eval_expr(expr)).await?;
                    results.push(result);
                }

                // Return single value if only one expression, otherwise return list
                let result_value = if results.len() == 1 {
                    results[0].clone()
                } else {
                    Value::List(results.clone())
                };

                // Store result as _
                self.vars.insert("_".to_string(), result_value.clone());

                // Handle binding if present
                if let Some(bind) = binding {
                    match bind {
                        crate::parser::Binding::Single(name) => {
                            self.vars.insert(name.clone(), result_value.clone());
                        }
                        crate::parser::Binding::List(names) => {
                            // Destructure results into named variables
                            for (i, name) in names.iter().enumerate() {
                                if let Some(item) = results.get(i) {
                                    self.vars.insert(name.clone(), item.clone());
                                } else {
                                    return Err(format!(
                                        "Not enough results to destructure: expected at least {}, got {}",
                                        names.len(),
                                        results.len()
                                    ));
                                }
                            }
                        }
                    }
                }

                Ok(result_value)
            }
        }
    }

    fn access_field(&self, value: &Value, field: &str) -> Result<Value, String> {
        match value {
            Value::Map(m) => m
                .get(field)
                .cloned()
                .ok_or_else(|| format!("Field '{}' not found", field)),
            _ => Err(format!("Cannot access field on {}", value.type_name())),
        }
    }

    fn access_index(&self, value: &Value, index: usize) -> Result<Value, String> {
        match value {
            Value::List(items) => items
                .get(index)
                .cloned()
                .ok_or_else(|| format!("Index {} out of bounds", index)),
            _ => Err(format!("Cannot index {}", value.type_name())),
        }
    }

    fn apply_op(&self, left: &Value, op: &str, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                let result = match op {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => {
                        if *b == 0 {
                            return Err("Division by zero".to_string());
                        }
                        a / b
                    }
                    _ => return Err(format!("Unknown operator: {}", op)),
                };
                Ok(Value::Int(result))
            }
            (Value::Float(a), Value::Float(b)) => {
                let result = match op {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => a / b,
                    _ => return Err(format!("Unknown operator: {}", op)),
                };
                Ok(Value::Float(result))
            }
            // Mixed Int/Float operations - promote to Float
            (Value::Int(a), Value::Float(b)) => {
                let result = match op {
                    "+" => *a as f64 + b,
                    "-" => *a as f64 - b,
                    "*" => *a as f64 * b,
                    "/" => *a as f64 / b,
                    _ => return Err(format!("Unknown operator: {}", op)),
                };
                Ok(Value::Float(result))
            }
            (Value::Float(a), Value::Int(b)) => {
                let result = match op {
                    "+" => a + *b as f64,
                    "-" => a - *b as f64,
                    "*" => a * *b as f64,
                    "/" => a / *b as f64,
                    _ => return Err(format!("Unknown operator: {}", op)),
                };
                Ok(Value::Float(result))
            }
            // String concatenation
            (Value::String(a), Value::String(b)) if op == "+" => {
                Ok(Value::String(format!("{}{}", a, b)))
            }
            _ => Err(format!(
                "Type mismatch in operation: {} {} {}",
                left.type_name(),
                op,
                right.type_name()
            )),
        }
    }

    async fn interpolate_template(
        &mut self,
        segments: &[TemplateSegment],
    ) -> Result<String, String> {
        let mut result = String::new();

        for segment in segments {
            match segment {
                TemplateSegment::Text(text) => {
                    result.push_str(text);
                }
                TemplateSegment::Interpolation(expr_str) => {
                    // Parse and evaluate the expression
                    let expr = parse_expr(expr_str)?;
                    let value = Box::pin(self.eval_expr(&expr)).await?;

                    // Convert value to string - use to_prompt_string for full data
                    let str_value = value.to_prompt_string();

                    result.push_str(&str_value);
                }
            }
        }

        Ok(result)
    }

    async fn call_user_function(
        &mut self,
        func_def: &FunctionDef,
        args: &[Expr],
    ) -> Result<Value, String> {
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(Box::pin(self.eval_expr(arg)).await?);
        }

        // Check argument count
        if arg_values.len() != func_def.params.len() {
            return Err(format!(
                "Function '{}' expects {} arguments, got {}",
                func_def.name,
                func_def.params.len(),
                arg_values.len()
            ));
        }

        // Save current variable state
        let saved_vars = self.vars.clone();

        // Bind arguments to parameters
        for (param_name, arg_value) in func_def.params.iter().zip(arg_values.iter()) {
            self.vars.insert(param_name.clone(), arg_value.clone());
        }

        // Execute based on function type
        let result = match &func_def.execution {
            FunctionExecution::LLM { prompt, model: _, temperature: _ } => {
                self.execute_llm_function(func_def, prompt, &arg_values).await?
            }
            FunctionExecution::HTTP { method, url, params, headers, body } => {
                self.execute_http_function(func_def, method, url, params, headers, body, &arg_values).await?
            }
            FunctionExecution::SQL { query } => {
                self.execute_sql_function(func_def, query, &arg_values).await?
            }
            FunctionExecution::HTTPWithLLM {
                http_method,
                http_url,
                http_params,
                http_headers,
                llm_prompt,
                llm_model: _,
                llm_temperature: _,
            } => {
                // First execute HTTP request
                let http_result = self.execute_http_function(
                    func_def,
                    http_method,
                    http_url,
                    http_params,
                    http_headers,
                    &None,
                    &arg_values
                ).await?;

                // Then pass result to LLM
                self.execute_llm_with_input(func_def, llm_prompt, &http_result, &arg_values).await?
            }
        };

        // Restore variable state
        self.vars = saved_vars;

        Ok(result)
    }

    /// Execute an LLM-based function
    async fn execute_llm_function(
        &mut self,
        func_def: &FunctionDef,
        prompt_template: &str,
        arg_values: &[Value],
    ) -> Result<Value, String> {
        // Interpolate template variables
        let prompt = self.interpolate_string_template(prompt_template, &func_def.params, arg_values)?;

        // Call the appropriate builtin function based on return type
        if let Some(return_type) = &func_def.return_type {
            // Structured output - use ExtractAs
            let type_name = match return_type {
                simplify_baml::FieldType::Class(name) => name.clone(),
                simplify_baml::FieldType::Enum(name) => name.clone(),
                _ => return Err("Return type must be a class or enum".to_string()),
            };

            self.builtins
                .call(
                    "extractas",
                    vec![Value::String(prompt), Value::String(type_name)],
                )
                .await
                .map_err(|e| e.to_string())
        } else {
            // Simple string output - use Ask
            self.builtins
                .call("ask", vec![Value::String(prompt)])
                .await
                .map_err(|e| e.to_string())
        }
    }

    /// Execute LLM with pre-fetched input data
    async fn execute_llm_with_input(
        &mut self,
        func_def: &FunctionDef,
        prompt_template: &str,
        input_data: &Value,
        arg_values: &[Value],
    ) -> Result<Value, String> {
        // Interpolate template with both args and input data
        let mut prompt = self.interpolate_string_template(prompt_template, &func_def.params, arg_values)?;

        // Append the input data to the prompt
        prompt.push_str("\n\nData to analyze:\n");
        prompt.push_str(&input_data.to_prompt_string());

        // Call the appropriate builtin function based on return type
        if let Some(return_type) = &func_def.return_type {
            let type_name = match return_type {
                simplify_baml::FieldType::Class(name) => name.clone(),
                simplify_baml::FieldType::Enum(name) => name.clone(),
                _ => return Err("Return type must be a class or enum".to_string()),
            };

            self.builtins
                .call(
                    "extractas",
                    vec![Value::String(prompt), Value::String(type_name)],
                )
                .await
                .map_err(|e| e.to_string())
        } else {
            self.builtins
                .call("ask", vec![Value::String(prompt)])
                .await
                .map_err(|e| e.to_string())
        }
    }

    /// Execute an HTTP request
    async fn execute_http_function(
        &mut self,
        func_def: &FunctionDef,
        method: &str,
        url_template: &str,
        params: &Option<HashMap<String, String>>,
        headers: &Option<HashMap<String, String>>,
        body: &Option<String>,
        arg_values: &[Value],
    ) -> Result<Value, String> {
        // Interpolate URL template with arguments
        let url = self.interpolate_string_template(url_template, &func_def.params, arg_values)?;

        // Create HTTP client
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

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
            let interpolated_body = self.interpolate_string_template(body_content, &func_def.params, arg_values)?;

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
            return Err(format!("HTTP request failed with status: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")));
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
        func_def: &FunctionDef,
        query_template: &str,
        arg_values: &[Value],
    ) -> Result<Value, String> {
        // Interpolate query template with arguments
        let query = self.interpolate_string_template(query_template, &func_def.params, arg_values)?;

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

    fn handle_command(&mut self, cmd: &str, args: Option<&str>) -> Result<Value, String> {
        match cmd {
            "vars" => {
                let mut result = String::from("Variables:\n");
                if self.vars.is_empty() {
                    result.push_str("  (no variables defined yet)\n");
                } else {
                    for (name, value) in &self.vars {
                        result.push_str(&format!(
                            "  {} = {} : {}\n",
                            name,
                            value.display(),
                            value.type_name()
                        ));
                    }
                }
                Ok(Value::String(result))
            }
            "types" => {
                let mut result = String::from("Registered Types:\n\n");

                let classes = self.types.all_classes();
                let enums = self.types.all_enums();

                if classes.is_empty() && enums.is_empty() {
                    result.push_str("  (no types defined yet)\n");
                } else {
                    // Show classes
                    if !classes.is_empty() {
                        result.push_str("Classes:\n");
                        for class in classes {
                            result.push_str(&format!("  type {} {{\n", class.name));
                            for field in &class.fields {
                                let optional = if field.optional { "?" } else { "" };
                                result.push_str(&format!(
                                    "    {}{}: {},\n",
                                    field.name,
                                    optional,
                                    field.field_type.to_string()
                                ));
                            }
                            result.push_str("  }\n\n");
                        }
                    }

                    // Show enums
                    if !enums.is_empty() {
                        result.push_str("Enums:\n");
                        for enum_def in enums {
                            result.push_str(&format!("  enum {} {{ ", enum_def.name));
                            result.push_str(&enum_def.values.join(", "));
                            result.push_str(" }\n");
                        }
                    }
                }

                Ok(Value::String(result))
            }
            "copy" => {
                // Save the last result (_) to a file for copying
                if let Some(last_value) = self.vars.get("_") {
                    let filename = "dsl-last-result.txt";
                    let content = last_value.to_prompt_string();

                    std::fs::write(filename, &content)
                        .map_err(|e| format!("Failed to write file: {}", e))?;

                    Ok(Value::String(format!(
                        "Last result saved to '{}' ({} bytes)\nYou can now copy from this file.",
                        filename,
                        content.len()
                    )))
                } else {
                    Err("No result to copy yet. Run a command first.".to_string())
                }
            }
            "save" => {
                let filename = args.unwrap_or("session.json");

                // Create a session snapshot with just variables
                // Note: Types and functions are not saved as they contain complex structures
                let session = serde_json::json!({
                    "variables": self.vars,
                });

                let json = serde_json::to_string_pretty(&session)
                    .map_err(|e| format!("Serialization error: {}", e))?;

                std::fs::write(filename, json).map_err(|e| format!("Write error: {}", e))?;

                Ok(Value::String(format!(
                    "Session saved to '{}'\nVariables: {} saved\nNote: {} types and {} functions NOT saved (re-enter manually)",
                    filename,
                    self.vars.len(),
                    self.types.all_classes().len() + self.types.all_enums().len(),
                    self.functions.len()
                )))
            }
            "load" => {
                let filename = args.unwrap_or("session.json");

                let content =
                    std::fs::read_to_string(filename).map_err(|e| format!("Read error: {}", e))?;

                let data: serde_json::Value =
                    serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))?;

                // Restore variables
                let mut var_count = 0;
                if let Some(vars) = data.get("variables") {
                    if let Ok(vars_map) = serde_json::from_value::<
                        std::collections::HashMap<String, Value>,
                    >(vars.clone())
                    {
                        var_count = vars_map.len();
                        self.vars = vars_map;
                    }
                }

                // Note: We can't easily restore types and functions from JSON
                // since they contain complex Rust structures. This is a limitation
                // that could be improved in the future.

                Ok(Value::String(format!(
                    "Session loaded from '{}'\nRestored {} variables\nNote: Type and function definitions must be re-entered manually",
                    filename,
                    var_count
                )))
            }
            "debug" => {
                let mut result = String::from("Debug Info:\n\n");

                if let Some(prompt) = &self.builtins.last_prompt {
                    result.push_str("Last LLM Prompt:\n");
                    result.push_str(&format!("Length: {} characters\n\n", prompt.len()));

                    // Show first 500 chars
                    let preview: String = prompt.chars().take(500).collect();
                    result.push_str(&preview);

                    if prompt.len() > 500 {
                        result
                            .push_str(&format!("\n\n... ({} more characters)", prompt.len() - 500));
                    }
                } else {
                    result.push_str("No LLM calls made yet.\n");
                }

                Ok(Value::String(result))
            }
            "funcs" | "functions" => {
                let mut result = String::from("User-Defined Functions:\n\n");

                if self.functions.is_empty() {
                    result.push_str("  (no functions defined yet)\n");
                } else {
                    for (name, func_def) in &self.functions {
                        result.push_str(&format!("def {}(", name));
                        result.push_str(&func_def.params.join(", "));
                        result.push_str(")");

                        if let Some(return_type) = &func_def.return_type {
                            result.push_str(&format!(" -> {}", return_type.to_string()));
                        }

                        result.push_str(" {\n");

                        // Show properties
                        for (key, value) in &func_def.properties {
                            result.push_str(&format!("  {}: ", key));
                            match value {
                                crate::parser::PropertyValue::String(s) => {
                                    result.push_str(&format!("\"{}\"", s))
                                }
                                crate::parser::PropertyValue::Template(_) => {
                                    result.push_str("<template>")
                                }
                                crate::parser::PropertyValue::Int(n) => {
                                    result.push_str(&n.to_string())
                                }
                                crate::parser::PropertyValue::Float(f) => {
                                    result.push_str(&f.to_string())
                                }
                                crate::parser::PropertyValue::Bool(b) => {
                                    result.push_str(&b.to_string())
                                }
                            }
                            result.push_str("\n");
                        }

                        // Show execution type
                        match &func_def.execution {
                            FunctionExecution::LLM { .. } => {
                                result.push_str("  execution: LLM\n");
                            }
                            FunctionExecution::HTTP { method, url, .. } => {
                                result.push_str(&format!("  execution: HTTP {} {}\n", method, url));
                            }
                            FunctionExecution::SQL { .. } => {
                                result.push_str("  execution: SQL\n");
                            }
                            FunctionExecution::HTTPWithLLM { .. } => {
                                result.push_str("  execution: HTTP + LLM\n");
                            }
                        }

                        result.push_str("}\n\n");
                    }
                }

                Ok(Value::String(result))
            }
            _ => Err(format!("Unknown command: :{}", cmd)),
        }
    }
}
