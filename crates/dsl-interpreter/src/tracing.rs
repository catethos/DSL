use crate::error::InterpreterError;
use crate::interpreter::Interpreter;
use crate::pattern::PatternMatcher;
use anyhow::Result;
use dsl_ir::{IRNode, Value, IR};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// A single trace event recording an execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Sequential step number
    pub step: usize,
    /// Type of IR node being executed
    pub node_type: String,
    /// Brief description of the operation
    pub description: String,
    /// Input values (if applicable)
    pub inputs: Vec<Value>,
    /// Output value from this step
    pub output: Option<Value>,
    /// Error message if step failed
    pub error: Option<String>,
    /// Execution time in microseconds
    pub duration_micros: u128,
    /// Call stack depth
    pub depth: usize,
    /// Variable bindings at this step (optional, for detailed traces)
    pub variables: Option<Vec<(String, Value)>>,
    /// LLM prompt sent (for LLM function calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_prompt: Option<String>,
    /// LLM response received (for LLM function calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_response: Option<String>,
    /// Child events (nested hierarchy)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TraceEvent>,
}

/// Configuration for trace collection
#[derive(Debug, Clone)]
pub struct TraceConfig {
    /// Maximum number of events to keep (0 = unlimited)
    pub max_events: usize,
    /// Include variable snapshots in traces
    pub capture_variables: bool,
    /// Only trace nodes matching this filter (empty = trace all)
    pub node_filter: Vec<String>,
    /// Minimum duration (microseconds) to record (0 = record all)
    pub min_duration_micros: u128,
    /// Enable recursive tracing of all child nodes
    pub recursive: bool,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            max_events: 10000,
            capture_variables: false,
            node_filter: vec![],
            min_duration_micros: 0,
            recursive: true, // Enable recursive tracing by default
        }
    }
}

/// Collects and manages trace events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceCollector {
    pub events: Vec<TraceEvent>,
    #[serde(skip)]
    config: TraceConfig,
}

impl TraceCollector {
    pub fn new(config: TraceConfig) -> Self {
        Self {
            events: Vec::new(),
            config,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(TraceConfig::default())
    }

    /// Add a trace event
    pub fn record(&mut self, event: TraceEvent) {
        // Check node filter
        if !self.config.node_filter.is_empty()
            && !self.config.node_filter.contains(&event.node_type)
        {
            return;
        }

        // Check minimum duration
        if event.duration_micros < self.config.min_duration_micros {
            return;
        }

        // Add event
        self.events.push(event);

        // Enforce max events limit
        if self.config.max_events > 0 && self.events.len() > self.config.max_events {
            self.events.remove(0);
        }
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get total execution time
    pub fn total_duration(&self) -> Duration {
        let micros: u128 = self.events.iter().map(|e| e.duration_micros).sum();
        Duration::from_micros(micros as u64)
    }

    /// Get statistics by node type
    pub fn stats_by_node_type(&self) -> Vec<(String, usize, Duration)> {
        let mut stats: std::collections::HashMap<String, (usize, u128)> =
            std::collections::HashMap::new();

        for event in &self.events {
            let entry = stats.entry(event.node_type.clone()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += event.duration_micros;
        }

        let mut result: Vec<_> = stats
            .into_iter()
            .map(|(name, (count, micros))| (name, count, Duration::from_micros(micros as u64)))
            .collect();

        result.sort_by(|a, b| b.2.cmp(&a.2)); // Sort by duration descending
        result
    }

    /// Export trace as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export trace as JSON to file
    pub fn to_json_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load trace from JSON file
    pub fn from_json_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let mut collector: TraceCollector = serde_json::from_str(&json)?;
        collector.config = TraceConfig::default();
        Ok(collector)
    }

    /// Get a summary report
    pub fn summary(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Trace Summary ===\n");
        report.push_str(&format!("Total events: {}\n", self.events.len()));
        report.push_str(&format!("Total duration: {:?}\n", self.total_duration()));
        report.push_str("\nBreakdown by node type:\n");

        for (node_type, count, duration) in self.stats_by_node_type() {
            report.push_str(&format!(
                "  {:20} {:6} calls  {:?}\n",
                node_type, count, duration
            ));
        }

        report
    }

    /// Get events that took longer than threshold
    pub fn slow_events(&self, threshold_micros: u128) -> Vec<&TraceEvent> {
        self.events
            .iter()
            .filter(|e| e.duration_micros >= threshold_micros)
            .collect()
    }

    /// Get all errors
    pub fn errors(&self) -> Vec<&TraceEvent> {
        self.events.iter().filter(|e| e.error.is_some()).collect()
    }
}

/// Tracing interpreter that records execution steps
pub struct TracingInterpreter {
    pub interpreter: Interpreter,
    pub trace: TraceCollector,
    current_step: usize,
    /// Captured inputs for the current node (set during eval_recursive)
    current_inputs: Vec<Value>,
    /// Captured LLM prompt (for LLM function calls)
    current_llm_prompt: Option<String>,
    /// Captured LLM response (for LLM function calls)
    current_llm_response: Option<String>,
    /// Stack of parent event indices for nested hierarchy
    parent_stack: Vec<usize>,
}

impl TracingInterpreter {
    /// Create a new tracing interpreter with default config
    pub fn new() -> Result<Self> {
        Ok(Self {
            interpreter: Interpreter::new()?,
            trace: TraceCollector::with_default_config(),
            current_step: 0,
            current_inputs: Vec::new(),
            current_llm_prompt: None,
            current_llm_response: None,
            parent_stack: Vec::new(),
        })
    }

    /// Create with custom trace configuration
    pub fn with_config(config: TraceConfig) -> Result<Self> {
        Ok(Self {
            interpreter: Interpreter::new()?,
            trace: TraceCollector::new(config),
            current_step: 0,
            current_inputs: Vec::new(),
            current_llm_prompt: None,
            current_llm_response: None,
            parent_stack: Vec::new(),
        })
    }

    /// Create from IR with default config
    pub fn from_ir(ir: &IR) -> Result<Self> {
        Ok(Self {
            interpreter: Interpreter::from_ir(ir)?,
            trace: TraceCollector::with_default_config(),
            current_step: 0,
            current_inputs: Vec::new(),
            current_llm_prompt: None,
            current_llm_response: None,
            parent_stack: Vec::new(),
        })
    }

    /// Create from IR with custom config
    pub fn from_ir_with_config(ir: &IR, config: TraceConfig) -> Result<Self> {
        Ok(Self {
            interpreter: Interpreter::from_ir(ir)?,
            trace: TraceCollector::new(config),
            current_step: 0,
            current_inputs: Vec::new(),
            current_llm_prompt: None,
            current_llm_response: None,
            parent_stack: Vec::new(),
        })
    }

    /// Clear trace history
    pub fn clear_trace(&mut self) {
        self.trace.clear();
        self.current_step = 0;
        self.current_inputs.clear();
        self.current_llm_prompt = None;
        self.current_llm_response = None;
        self.parent_stack.clear();
    }

    /// Evaluate an IR node with tracing
    pub async fn eval(&mut self, node: &IRNode) -> Result<Value, InterpreterError> {
        let start = Instant::now();
        let depth = self.interpreter.runtime.scopes.len();
        let step = self.current_step;
        self.current_step += 1;

        // Get node type and description
        let (node_type, description) = describe_node(node);

        // Clear current inputs before evaluation
        self.current_inputs.clear();

        // Record the trace event BEFORE evaluation so parent nodes appear before children
        // We'll update it with results after evaluation
        let event_index = self.trace.events.len();
        let placeholder = TraceEvent {
            step,
            node_type: node_type.clone(),
            description: description.clone(),
            inputs: Vec::new(),
            output: None,
            error: None,
            duration_micros: 0,
            depth,
            variables: None,
            llm_prompt: None,
            llm_response: None,
            children: Vec::new(),
        };
        self.trace.events.push(placeholder);

        // Push this event as the parent for any child evaluations
        self.parent_stack.push(event_index);

        // Execute the node (with recursive tracing if enabled)
        let result = if self.trace.config.recursive {
            self.eval_recursive(node).await
        } else {
            self.interpreter.eval(node).await
        };

        // Pop this event from the parent stack
        self.parent_stack.pop();

        // Get captured inputs (may have been set during eval_recursive)
        let inputs = self.current_inputs.clone();

        // Update the trace event with actual results
        let duration = start.elapsed();
        if let Some(event) = self.trace.events.get_mut(event_index) {
            event.inputs = inputs;
            event.output = result.as_ref().ok().cloned();
            event.error = result.as_ref().err().map(|e| e.to_string());
            event.duration_micros = duration.as_micros();
            event.llm_prompt = self.current_llm_prompt.take();
            event.llm_response = self.current_llm_response.take();
            if self.trace.config.capture_variables {
                event.variables = Some(capture_variables(&self.interpreter));
            }
        }

        // Move this event to parent's children if it has a parent
        if !self.parent_stack.is_empty() {
            if let Some(parent_idx) = self.parent_stack.last().copied() {
                // Pop the child event from the flat list
                if let Some(child) = self.trace.events.pop() {
                    // Add it to parent's children
                    if let Some(parent) = self.trace.events.get_mut(parent_idx) {
                        parent.children.push(child);
                    }
                }
            }
        }

        result
    }

    /// Recursively evaluate with tracing for child nodes
    async fn eval_recursive(&mut self, node: &IRNode) -> Result<Value, InterpreterError> {
        use dsl_ir::IRNode::*;

        match node {
            // Nodes with children that should be traced
            Sequential {
                left,
                right,
                binding,
            } => {
                // Trace left child
                let left_val = Box::pin(self.eval(left)).await?;

                // Store result as _
                self.interpreter
                    .runtime
                    .set_var("_".to_string(), left_val.clone());

                // If there's a binding, apply it
                if let Some(bind) = binding {
                    self.interpreter.apply_binding(bind, &left_val)?;
                }

                // Trace right child
                Box::pin(self.eval(right)).await
            }

            FunctionCall { name, args, .. } => {
                // Trace each argument
                let mut traced_args = Vec::new();
                for arg in args {
                    traced_args.push(Box::pin(self.eval(arg)).await?);
                }

                // Store the argument values as inputs for this function call
                self.current_inputs = traced_args.clone();

                // Check for overloaded function first (function groups / pattern functions)
                if let Some(func_group) =
                    self.interpreter.runtime.function_groups.get(name).cloned()
                {
                    // Create IR nodes from traced values for pattern matching
                    let arg_nodes: Vec<IRNode> = traced_args
                        .iter()
                        .map(|v| {
                            // Convert value back to IRNode for pattern matching
                            match v {
                                Value::Int(i) => IRNode::Int(*i),
                                Value::Float(f) => IRNode::Float(*f),
                                Value::Bool(b) => IRNode::Bool(*b),
                                Value::String(s) => IRNode::String(s.clone()),
                                Value::List(_) => IRNode::Variable("_".to_string()), // Simplified
                                _ => IRNode::Variable("_".to_string()),
                            }
                        })
                        .collect();

                    // Use interpreter's method with actual values
                    self.interpreter
                        .call_overloaded_function(&func_group, &arg_nodes)
                        .await
                } else if let Some(func_def) = self.interpreter.runtime.functions.get(name).cloned()
                {
                    // Regular user-defined function - create IR nodes from values
                    let arg_nodes: Vec<IRNode> = traced_args
                        .iter()
                        .map(|v| match v {
                            Value::Int(i) => IRNode::Int(*i),
                            Value::Float(f) => IRNode::Float(*f),
                            Value::Bool(b) => IRNode::Bool(*b),
                            Value::String(s) => IRNode::String(s.clone()),
                            Value::List(_) => IRNode::Variable("_".to_string()),
                            _ => IRNode::Variable("_".to_string()),
                        })
                        .collect();

                    let result = self
                        .interpreter
                        .call_user_function(&func_def, &arg_nodes)
                        .await;

                    // Check if this user function makes LLM calls
                    // User functions with IRExecution::LLM or IRExecution::HTTPWithLLM will have set last_prompt/last_response
                    if self.interpreter.builtins.last_prompt.is_some() {
                        self.current_llm_prompt = self.interpreter.builtins.last_prompt.clone();
                        self.current_llm_response = self.interpreter.builtins.last_response.clone();
                    }

                    result
                } else {
                    // Call builtin function with evaluated args
                    let result = self
                        .interpreter
                        .builtins
                        .call(name, traced_args)
                        .await
                        .map_err(Into::into);

                    // If this is an LLM function, capture the prompt and response
                    let lname = name.to_lowercase();
                    if lname == "ask" || lname == "extractas" || lname == "extractperson" {
                        self.current_llm_prompt = self.interpreter.builtins.last_prompt.clone();
                        self.current_llm_response = self.interpreter.builtins.last_response.clone();
                    }

                    result
                }
            }

            BinaryOp { left, op, right } => {
                // Trace operands
                let left_val = Box::pin(self.eval(left)).await?;
                let right_val = Box::pin(self.eval(right)).await?;

                // Apply operation (don't trace)
                self.interpreter.apply_binary_op(op, left_val, right_val)
            }

            List(items) => {
                // Trace each list item
                let mut values = Vec::new();
                for item in items {
                    values.push(Box::pin(self.eval(item)).await?);
                }
                Ok(Value::List(values))
            }

            Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                // Trace condition
                let cond_val = Box::pin(self.eval(condition)).await?;
                let is_true = match cond_val {
                    Value::Bool(b) => b,
                    _ => return Err(InterpreterError::TypeError {
                        message: "Condition must be a boolean".to_string(),
                        expected: "Bool".to_string(),
                        got: cond_val.type_name().to_string(),
                        source_span: None,
                    }),
                };

                // Trace appropriate branch
                if is_true {
                    Box::pin(self.eval(then_expr)).await
                } else {
                    Box::pin(self.eval(else_expr)).await
                }
            }

            Match { scrutinee, cases } => {
                // Trace the scrutinee (the value being matched)
                let scrutinee_val = Box::pin(self.eval(scrutinee)).await?;

                // Try each case in order
                for case in cases {
                    // Check if pattern matches
                    if PatternMatcher::matches(&case.pattern, &scrutinee_val) {
                        // Push new scope for this case
                        self.interpreter.runtime.push_scope();

                        // Extract bindings and add to scope
                        let bindings =
                            PatternMatcher::extract_bindings(&case.pattern, &scrutinee_val)
                                .map_err(|e| InterpreterError::RuntimeError {
                                    message: format!("Pattern binding error: {}", e),
                                    source_span: None,
                                })?;

                        for (name, val) in &bindings {
                            self.interpreter.runtime.set_var(name.clone(), val.clone());
                        }

                        // Check guard if present
                        if let Some(guard) = &case.guard {
                            // Trace guard evaluation
                            let guard_result = Box::pin(self.eval(guard)).await?;

                            // If guard fails, pop scope and try next case
                            if !matches!(guard_result, Value::Bool(true)) {
                                self.interpreter.runtime.pop_scope();
                                continue;
                            }
                        }

                        // Trace body evaluation
                        let result = Box::pin(self.eval(&case.body)).await;

                        self.interpreter.runtime.pop_scope();
                        return result;
                    }
                }

                Err(InterpreterError::RuntimeError {
                    message: "No pattern matched in match expression".to_string(),
                    source_span: None,
                })
            }

            // For all other nodes, delegate to interpreter
            _ => self.interpreter.eval(node).await,
        }
    }

    /// Get the trace collector
    pub fn get_trace(&self) -> &TraceCollector {
        &self.trace
    }

    /// Get mutable trace collector
    pub fn get_trace_mut(&mut self) -> &mut TraceCollector {
        &mut self.trace
    }

    /// Export trace to JSON file
    pub fn export_trace(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.trace.to_json_file(path)
    }

    /// Print trace summary
    pub fn print_summary(&self) {
        println!("{}", self.trace.summary());
    }
}

impl Default for TracingInterpreter {
    fn default() -> Self {
        Self::new().expect("Failed to create tracing interpreter")
    }
}

/// Describe an IR node for tracing
fn describe_node(node: &IRNode) -> (String, String) {
    match node {
        IRNode::String(s) => (
            "String".to_string(),
            format!(
                "String literal: {:?}",
                s.chars().take(50).collect::<String>()
            ),
        ),
        IRNode::Int(i) => ("Int".to_string(), format!("Integer: {}", i)),
        IRNode::Float(f) => ("Float".to_string(), format!("Float: {}", f)),
        IRNode::Bool(b) => ("Bool".to_string(), format!("Boolean: {}", b)),
        IRNode::List(_) => ("List".to_string(), "List construction".to_string()),
        IRNode::Map(_) => ("Map".to_string(), "Map construction".to_string()),
        IRNode::Variable(name) => ("Variable".to_string(), format!("Variable access: {}", name)),
        IRNode::FunctionCall { name, args, .. } => (
            "FunctionCall".to_string(),
            format!("Call {}(...) with {} args", name, args.len()),
        ),
        IRNode::BinaryOp { op, .. } => {
            ("BinaryOp".to_string(), format!("Binary operation: {}", op))
        }
        IRNode::Conditional { .. } => (
            "Conditional".to_string(),
            "Conditional expression".to_string(),
        ),
        IRNode::Sequential { binding, .. } => (
            "Sequential".to_string(),
            if let Some(bind) = binding {
                format!("Sequential composition with binding: {:?}", bind)
            } else {
                "Sequential composition".to_string()
            },
        ),
        IRNode::Parallel { exprs, .. } => (
            "Parallel".to_string(),
            format!("Parallel composition ({} exprs)", exprs.len()),
        ),
        IRNode::Match { .. } => ("Match".to_string(), "Pattern matching".to_string()),
        IRNode::Block { statements, .. } => (
            "Block".to_string(),
            format!("Block with {} statements", statements.len()),
        ),
        IRNode::FieldAccess { field, .. } => (
            "FieldAccess".to_string(),
            format!("Field access: .{}", field),
        ),
        IRNode::IndexAccess { .. } => {
            ("IndexAccess".to_string(), "Index access: [...]".to_string())
        }
        IRNode::TemplateString(segments) => (
            "TemplateString".to_string(),
            format!("Template string ({} segments)", segments.len()),
        ),
        IRNode::TypeInstantiation { type_name, .. } => (
            "TypeInstantiation".to_string(),
            format!("Type instantiation: {}", type_name),
        ),
        _ => ("Unknown".to_string(), "Unknown node type".to_string()),
    }
}

/// Capture current variable bindings
fn capture_variables(_interpreter: &Interpreter) -> Vec<(String, Value)> {
    // Capture variables from all scopes (top scope = global)
    // We can only capture from the current (top) scope
    // since runtime.scopes is not public
    // This is a simplified version - in production you might want
    // to add a method to Runtime to export all variables

    // For now, return empty vec - could be enhanced by adding
    // a public method to Runtime
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsl_ir::IRNode;

    #[tokio::test]
    async fn test_basic_tracing() {
        let mut interp = TracingInterpreter::new().unwrap();

        // Simple expression: 2 + 3
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::Int(2)),
            op: "+".to_string(),
            right: Box::new(IRNode::Int(3)),
        };

        let result = interp.eval(&node).await.unwrap();
        assert_eq!(result, Value::Int(5));

        // Should have trace events for the binary op and its operands
        assert!(!interp.trace.events.is_empty());

        // The BinaryOp should be in the trace
        let has_binop = interp
            .trace
            .events
            .iter()
            .any(|e| e.node_type == "BinaryOp");
        assert!(has_binop);
    }

    #[tokio::test]
    async fn test_trace_filtering() {
        let config = TraceConfig {
            node_filter: vec!["BinaryOp".to_string()],
            ..Default::default()
        };

        let mut interp = TracingInterpreter::with_config(config).unwrap();

        // Expression with multiple node types
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::Int(10)),
            op: "*".to_string(),
            right: Box::new(IRNode::Int(5)),
        };

        interp.eval(&node).await.unwrap();

        // Should only have BinaryOp events
        for event in &interp.trace.events {
            assert_eq!(event.node_type, "BinaryOp");
        }
    }

    #[tokio::test]
    async fn test_trace_export() {
        let mut interp = TracingInterpreter::new().unwrap();

        let node = IRNode::Int(42);
        interp.eval(&node).await.unwrap();

        // Export to JSON
        let json = interp.trace.to_json().unwrap();
        assert!(json.contains("\"node_type\""));
        assert!(json.contains("\"step\""));
    }

    #[tokio::test]
    async fn test_trace_summary() {
        let mut interp = TracingInterpreter::new().unwrap();

        // Execute several operations
        for i in 0..5 {
            let node = IRNode::Int(i);
            interp.eval(&node).await.unwrap();
        }

        let summary = interp.trace.summary();
        assert!(summary.contains("Total events:"));
        assert!(summary.contains("Int"));
    }

    #[tokio::test]
    async fn test_clear_trace() {
        let mut interp = TracingInterpreter::new().unwrap();

        let node = IRNode::Int(42);
        interp.eval(&node).await.unwrap();
        assert!(!interp.trace.events.is_empty());

        interp.clear_trace();
        assert!(interp.trace.events.is_empty());
        assert_eq!(interp.current_step, 0);
    }
}
