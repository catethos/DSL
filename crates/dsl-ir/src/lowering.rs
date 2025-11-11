//! Lowering pass from High-Level IR (HIR) to Low-Level IR (LIR)
//!
//! This module transforms special execution constructs (LLM, HTTP, SQL, HTTPWithLLM)
//! into composable expressions using intrinsic builtin function calls.
//!
//! # Architecture
//!
//! - **High-Level IR (HIR)**: Contains `IRExecution::LLM`, `HTTP`, `SQL` for static analysis
//! - **Low-Level IR (LIR)**: Contains only `IRExecution::Expression` with intrinsic calls
//!
//! # Purpose
//!
//! - Preserves effect tracking and static analysis capabilities in HIR
//! - Provides composability and simpler interpreter via LIR
//! - Maintains source location information for error reporting

use crate::ir::{IR, IRFunction, IRExecution, IRNode, IRTemplateSegment, LambdaIR, Span};
use std::collections::HashMap;

/// Unique identifier for IR nodes, used to track debug information
pub type NodeId = usize;

/// Debug information table mapping lowered nodes back to their HIR origins
#[derive(Debug, Clone, Default)]
pub struct DebugInfoTable {
    entries: HashMap<NodeId, DebugInfo>,
    next_id: NodeId,
}

impl DebugInfoTable {
    /// Create a new empty debug info table
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            next_id: 0,
        }
    }

    /// Allocate a new node ID
    pub fn allocate_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Record debug information for a node
    pub fn record(&mut self, id: NodeId, info: DebugInfo) {
        self.entries.insert(id, info);
    }

    /// Retrieve debug information for a node
    pub fn get(&self, id: &NodeId) -> Option<&DebugInfo> {
        self.entries.get(id)
    }
}

/// Debug information for a lowered IR node
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// Original construct type (e.g., "LLM function", "HTTP request")
    pub original_construct: String,
    /// Source location in the DSL file
    pub source_span: Span,
    /// Function name containing this construct
    pub function_name: String,
    /// Original prompt text for LLM calls
    pub prompt_text: Option<String>,
}

/// Lowering pass that transforms HIR to LIR
pub struct Lowering {
    debug_info: DebugInfoTable,
}

impl Lowering {
    /// Create a new lowering pass
    pub fn new() -> Self {
        Self {
            debug_info: DebugInfoTable::new(),
        }
    }

    /// Lower an entire IR program from HIR to LIR
    ///
    /// Returns the lowered program and debug information table for error reporting
    pub fn lower_program(mut hir: IR) -> (IR, DebugInfoTable) {
        let mut lowering = Self::new();

        // Lower each function in the program
        for func in &mut hir.functions {
            lowering.lower_function(func);
        }

        (hir, lowering.debug_info)
    }

    /// Lower a single function's execution
    fn lower_function(&mut self, func: &mut IRFunction) {
        // Convert FieldType to String if present
        let return_type_str = func.return_type.as_ref().map(|ft| ft.to_string());
        func.execution = self.lower_execution(&func.execution, &func.name, &return_type_str);
    }

    /// Lower an execution construct
    ///
    /// This is the main entry point for lowering individual executions.
    /// Can be called directly from the interpreter.
    pub fn lower_execution_direct(&mut self, exec: &IRExecution, function_name: &str, return_type: &Option<String>) -> IRExecution {
        self.lower_execution(exec, function_name, return_type)
    }

    /// Lower an execution construct (internal)
    fn lower_execution(&mut self, exec: &IRExecution, function_name: &str, return_type: &Option<String>) -> IRExecution {
        match exec {
            // LLM execution: transform to __llm_execute intrinsic call
            IRExecution::LLM { prompt, model, base_url, api_key_env, temperature } => {
                self.lower_llm(prompt, model, base_url, api_key_env, temperature, return_type, function_name)
            }

            // HTTP execution: transform to __http intrinsic call
            IRExecution::HTTP { method, url, params, headers, body } => {
                self.lower_http(method, url, params, headers, body, function_name)
            }

            // SQL execution: transform to __sql intrinsic call
            IRExecution::SQL { query } => {
                self.lower_sql(query, function_name)
            }

            // Expression: recursively lower nested nodes
            IRExecution::Expression { body } => {
                IRExecution::Expression {
                    body: Box::new(self.lower_node(body, function_name)),
                }
            }
        }
    }

    /// Lower LLM execution to intrinsic call
    fn lower_llm(
        &mut self,
        prompt: &IRNode,
        model: &Option<String>,
        base_url: &Option<String>,
        api_key_env: &Option<String>,
        temperature: &Option<f64>,
        return_type: &Option<String>,
        function_name: &str,
    ) -> IRExecution {
        // Allocate a node ID for debug tracking
        let node_id = self.debug_info.allocate_id();

        // Record debug info for this lowered LLM call
        self.debug_info.record(
            node_id,
            DebugInfo {
                original_construct: "LLM function".to_string(),
                source_span: Span {
                    file: "<generated>".to_string(),
                    line: 0,
                    column: 0,
                },
                function_name: function_name.to_string(),
                prompt_text: None, // Could extract text from template if needed
            },
        );

        // Build config map from optional parameters
        let mut config_entries = Vec::new();
        if let Some(m) = model {
            config_entries.push(("model".to_string(), IRNode::String(m.clone())));
        }
        if let Some(url) = base_url {
            config_entries.push(("base_url".to_string(), IRNode::String(url.clone())));
        }
        if let Some(key_env) = api_key_env {
            config_entries.push(("api_key_env".to_string(), IRNode::String(key_env.clone())));
        }
        if let Some(temp) = temperature {
            config_entries.push(("temperature".to_string(), IRNode::Float(*temp)));
        }
        // Add return type to config
        if let Some(rt) = return_type {
            config_entries.push(("return_type".to_string(), IRNode::String(rt.clone())));
        }

        // Recursively lower the prompt template
        let lowered_prompt = self.lower_node(prompt, function_name);

        // Create the intrinsic call: __llm_execute(prompt, config)
        IRExecution::Expression {
            body: Box::new(IRNode::FunctionCall {
                name: "__llm_execute".to_string(),
                args: vec![
                    lowered_prompt,
                    IRNode::Map(config_entries),
                ],
                effect_kind: Some(crate::ir::EffectKind::LLM),
                source_span: Some(Span {
                    file: "<generated>".to_string(),
                    line: 0,
                    column: 0,
                }),
            }),
        }
    }

    /// Lower HTTP execution to intrinsic call
    fn lower_http(
        &mut self,
        method: &str,
        url: &str,
        params: &Option<HashMap<String, String>>,
        headers: &Option<HashMap<String, String>>,
        body: &Option<String>,
        function_name: &str,
    ) -> IRExecution {
        // Allocate a node ID for debug tracking
        let node_id = self.debug_info.allocate_id();

        // Record debug info for this lowered HTTP call
        self.debug_info.record(
            node_id,
            DebugInfo {
                original_construct: "HTTP request".to_string(),
                source_span: Span {
                    file: "<generated>".to_string(),
                    line: 0,
                    column: 0,
                },
                function_name: function_name.to_string(),
                prompt_text: None,
            },
        );

        // Convert params HashMap to IRNode::Map
        let params_node = if let Some(p) = params {
            IRNode::Map(
                p.iter()
                    .map(|(k, v)| (k.clone(), IRNode::String(v.clone())))
                    .collect(),
            )
        } else {
            IRNode::Map(Vec::new())
        };

        // Convert headers HashMap to IRNode::Map
        let headers_node = if let Some(h) = headers {
            IRNode::Map(
                h.iter()
                    .map(|(k, v)| (k.clone(), IRNode::String(v.clone())))
                    .collect(),
            )
        } else {
            IRNode::Map(Vec::new())
        };

        // Convert body to IRNode
        let body_node = if let Some(b) = body {
            IRNode::String(b.clone())
        } else {
            IRNode::String(String::new())
        };

        // Create the intrinsic call: __http(method, url, params, headers, body)
        IRExecution::Expression {
            body: Box::new(IRNode::FunctionCall {
                name: "__http".to_string(),
                args: vec![
                    IRNode::String(method.to_string()),
                    IRNode::String(url.to_string()),
                    params_node,
                    headers_node,
                    body_node,
                ],
                effect_kind: Some(crate::ir::EffectKind::HTTP),
                source_span: Some(Span {
                    file: "<generated>".to_string(),
                    line: 0,
                    column: 0,
                }),
            }),
        }
    }

    /// Lower SQL execution to intrinsic call
    fn lower_sql(
        &mut self,
        query: &str,
        function_name: &str,
    ) -> IRExecution {
        // Allocate a node ID for debug tracking
        let node_id = self.debug_info.allocate_id();

        // Record debug info for this lowered SQL call
        self.debug_info.record(
            node_id,
            DebugInfo {
                original_construct: "SQL query".to_string(),
                source_span: Span {
                    file: "<generated>".to_string(),
                    line: 0,
                    column: 0,
                },
                function_name: function_name.to_string(),
                prompt_text: None,
            },
        );

        // Create the intrinsic call: __sql(query)
        IRExecution::Expression {
            body: Box::new(IRNode::FunctionCall {
                name: "__sql".to_string(),
                args: vec![IRNode::String(query.to_string())],
                effect_kind: Some(crate::ir::EffectKind::SQL),
                source_span: Some(Span {
                    file: "<generated>".to_string(),
                    line: 0,
                    column: 0,
                }),
            }),
        }
    }

    /// Recursively lower IR nodes
    fn lower_node(&mut self, node: &IRNode, function_name: &str) -> IRNode {
        match node {
            // Recursively lower function call arguments
            IRNode::FunctionCall { name, args, effect_kind, source_span } => {
                IRNode::FunctionCall {
                    name: name.clone(),
                    args: args.iter().map(|arg| self.lower_node(arg, function_name)).collect(),
                    effect_kind: effect_kind.clone(),
                    source_span: source_span.clone(),
                }
            }

            // Recursively lower other composite nodes
            IRNode::BinaryOp { op, left, right } => {
                IRNode::BinaryOp {
                    op: op.clone(),
                    left: Box::new(self.lower_node(left, function_name)),
                    right: Box::new(self.lower_node(right, function_name)),
                }
            }

            IRNode::List(items) => {
                IRNode::List(
                    items.iter().map(|item| self.lower_node(item, function_name)).collect()
                )
            }

            IRNode::Map(entries) => {
                IRNode::Map(
                    entries
                        .iter()
                        .map(|(k, v)| (k.clone(), self.lower_node(v, function_name)))
                        .collect()
                )
            }

            IRNode::Conditional { condition, then_expr, else_expr } => {
                IRNode::Conditional {
                    condition: Box::new(self.lower_node(condition, function_name)),
                    then_expr: Box::new(self.lower_node(then_expr, function_name)),
                    else_expr: Box::new(self.lower_node(else_expr, function_name)),
                }
            }

            IRNode::Block { statements, result } => {
                IRNode::Block {
                    statements: statements.iter().map(|stmt| self.lower_node(stmt, function_name)).collect(),
                    result: Box::new(self.lower_node(result, function_name)),
                }
            }

            IRNode::Sequential { left, right, binding } => {
                IRNode::Sequential {
                    left: Box::new(self.lower_node(left, function_name)),
                    right: Box::new(self.lower_node(right, function_name)),
                    binding: binding.clone(),
                }
            }

            IRNode::Parallel { exprs, binding } => {
                IRNode::Parallel {
                    exprs: exprs.iter().map(|e| self.lower_node(e, function_name)).collect(),
                    binding: binding.clone(),
                }
            }

            IRNode::FieldAccess { base, field } => {
                IRNode::FieldAccess {
                    base: Box::new(self.lower_node(base, function_name)),
                    field: field.clone(),
                }
            }

            IRNode::IndexAccess { base, index } => {
                IRNode::IndexAccess {
                    base: Box::new(self.lower_node(base, function_name)),
                    index: Box::new(self.lower_node(index, function_name)),
                }
            }

            IRNode::TypeInstantiation { type_name, fields } => {
                IRNode::TypeInstantiation {
                    type_name: type_name.clone(),
                    fields: fields
                        .iter()
                        .map(|(k, v)| (k.clone(), self.lower_node(v, function_name)))
                        .collect(),
                }
            }

            // Template strings need to lower their interpolation expressions
            IRNode::TemplateString(segments) => {
                let lowered_segments = segments
                    .iter()
                    .map(|seg| match seg {
                        IRTemplateSegment::Text(t) => IRTemplateSegment::Text(t.clone()),
                        IRTemplateSegment::Interpolation(node) => {
                            IRTemplateSegment::Interpolation(Box::new(self.lower_node(node, function_name)))
                        }
                    })
                    .collect();
                IRNode::TemplateString(lowered_segments)
            }

            // Lambda needs to lower its body
            IRNode::Lambda(lambda_ir) => {
                IRNode::Lambda(LambdaIR {
                    params: lambda_ir.params.clone(),
                    body: Box::new(self.lower_node(&lambda_ir.body, function_name)),
                })
            }

            // Leaf nodes that don't need lowering: pass through unchanged
            IRNode::String(_) |
            IRNode::Int(_) |
            IRNode::Float(_) |
            IRNode::Bool(_) |
            IRNode::Variable(_) |
            IRNode::Match { .. } |
            IRNode::SpawnAgent { .. } |
            IRNode::SendMessage { .. } |
            IRNode::CallAgent { .. } |
            IRNode::ReceiveMessage { .. } |
            IRNode::Broadcast { .. } |
            IRNode::Loop { .. } |
            IRNode::While { .. } |
            IRNode::For { .. } |
            IRNode::Break { .. } |
            IRNode::Continue |
            IRNode::TryBlock { .. } |
            IRNode::Throw { .. } => node.clone(),
        }
    }
}

impl Default for Lowering {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_info_table() {
        let mut table = DebugInfoTable::new();

        let id1 = table.allocate_id();
        let id2 = table.allocate_id();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);

        let info = DebugInfo {
            original_construct: "LLM function".to_string(),
            source_span: Span {
                file: "test.dsl".to_string(),
                line: 10,
                column: 5,
            },
            function_name: "extract_user".to_string(),
            prompt_text: Some("Extract user info".to_string()),
        };

        table.record(id1, info.clone());

        assert!(table.get(&id1).is_some());
        assert_eq!(table.get(&id1).unwrap().function_name, "extract_user");
    }

    #[test]
    fn test_lowering_creation() {
        let lowering = Lowering::new();
        assert_eq!(lowering.debug_info.next_id, 0);
    }

    #[test]
    fn test_lower_expression_passthrough() {
        let mut lowering = Lowering::new();

        let exec = IRExecution::Expression {
            body: Box::new(IRNode::Float(42.0)),
        };

        let lowered = lowering.lower_execution(&exec, "test_func", &None);

        match lowered {
            IRExecution::Expression { body } => {
                match *body {
                    IRNode::Float(n) => assert_eq!(n, 42.0),
                    _ => panic!("Expected Float node"),
                }
            }
            _ => panic!("Expected Expression"),
        }
    }

    #[test]
    fn test_lower_llm() {
        let mut lowering = Lowering::new();

        let exec = IRExecution::LLM {
            prompt: Box::new(IRNode::String("Extract user info".to_string())),
            model: Some("gpt-4".to_string()),
            base_url: None,
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            temperature: Some(0.7),
        };

        let lowered = lowering.lower_execution(&exec, "extract_user", &Some("Person".to_string()));

        // Should be lowered to Expression with __llm_execute call
        match lowered {
            IRExecution::Expression { body } => {
                match *body {
                    IRNode::FunctionCall { name, args, effect_kind, source_span } => {
                        assert_eq!(name, "__llm_execute");
                        assert_eq!(args.len(), 2);

                        // Check prompt argument
                        match &args[0] {
                            IRNode::String(s) => assert_eq!(s, "Extract user info"),
                            _ => panic!("Expected String node for prompt"),
                        }

                        // Check config argument (should be a Map)
                        match &args[1] {
                            IRNode::Map(entries) => {
                                assert!(entries.iter().any(|(k, _)| k == "model"));
                                assert!(entries.iter().any(|(k, _)| k == "api_key_env"));
                                assert!(entries.iter().any(|(k, _)| k == "temperature"));
                            }
                            _ => panic!("Expected Map node for config"),
                        }

                        // Check effect kind
                        assert_eq!(effect_kind, Some(crate::ir::EffectKind::LLM));
                        assert!(source_span.is_some());
                    }
                    _ => panic!("Expected FunctionCall node"),
                }
            }
            _ => panic!("Expected Expression"),
        }
    }

    #[test]
    fn test_lower_http() {
        let mut lowering = Lowering::new();

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let exec = IRExecution::HTTP {
            method: "GET".to_string(),
            url: "https://api.example.com/users".to_string(),
            params: None,
            headers: Some(headers),
            body: None,
        };

        let lowered = lowering.lower_execution(&exec, "fetch_users", &None);

        match lowered {
            IRExecution::Expression { body } => {
                match *body {
                    IRNode::FunctionCall { name, args, effect_kind, .. } => {
                        assert_eq!(name, "__http");
                        assert_eq!(args.len(), 5);

                        // Check method
                        match &args[0] {
                            IRNode::String(s) => assert_eq!(s, "GET"),
                            _ => panic!("Expected String for method"),
                        }

                        // Check URL
                        match &args[1] {
                            IRNode::String(s) => assert_eq!(s, "https://api.example.com/users"),
                            _ => panic!("Expected String for URL"),
                        }

                        // Check effect kind
                        assert_eq!(effect_kind, Some(crate::ir::EffectKind::HTTP));
                    }
                    _ => panic!("Expected FunctionCall node"),
                }
            }
            _ => panic!("Expected Expression"),
        }
    }

    #[test]
    fn test_lower_sql() {
        let mut lowering = Lowering::new();

        let exec = IRExecution::SQL {
            query: "SELECT * FROM users WHERE age > 18".to_string(),
        };

        let lowered = lowering.lower_execution(&exec, "get_adult_users", &None);

        match lowered {
            IRExecution::Expression { body } => {
                match *body {
                    IRNode::FunctionCall { name, args, effect_kind, .. } => {
                        assert_eq!(name, "__sql");
                        assert_eq!(args.len(), 1);

                        // Check query
                        match &args[0] {
                            IRNode::String(s) => assert_eq!(s, "SELECT * FROM users WHERE age > 18"),
                            _ => panic!("Expected String for query"),
                        }

                        // Check effect kind
                        assert_eq!(effect_kind, Some(crate::ir::EffectKind::SQL));
                    }
                    _ => panic!("Expected FunctionCall node"),
                }
            }
            _ => panic!("Expected Expression"),
        }
    }

    #[test]
    fn test_lower_program() {
        use dsl_types::FieldType;

        let ir = crate::ir::IR {
            version: "1.0".to_string(),
            types: vec![],
            enums: vec![],
            functions: vec![
                crate::ir::IRFunction {
                    name: "test_llm".to_string(),
                    params: vec![],
                    return_type: Some(FieldType::String),
                    properties: HashMap::new(),
                    execution: IRExecution::LLM {
                        prompt: Box::new(IRNode::String("Test prompt".to_string())),
                        model: None,
                        base_url: None,
                        api_key_env: None,
                        temperature: None,
                    },
                },
                crate::ir::IRFunction {
                    name: "test_sql".to_string(),
                    params: vec![],
                    return_type: Some(FieldType::String),
                    properties: HashMap::new(),
                    execution: IRExecution::SQL {
                        query: "SELECT * FROM test".to_string(),
                    },
                },
            ],
            function_groups: vec![],
            agents: vec![],
            entry_expr: IRNode::String("test".to_string()),
        };

        let (lowered_ir, debug_info) = Lowering::lower_program(ir);

        // All functions should be lowered to Expression
        for func in &lowered_ir.functions {
            match &func.execution {
                IRExecution::Expression { .. } => {
                    // Success - all executions lowered
                }
                _ => panic!("Function {} was not lowered to Expression", func.name),
            }
        }

        // Debug info should be populated
        assert!(debug_info.next_id > 0);
    }

    #[test]
    fn test_lower_template_string() {
        let mut lowering = Lowering::new();

        // Create a template string with an interpolation containing a function call
        let template = IRNode::TemplateString(vec![
            IRTemplateSegment::Text("Hello ".to_string()),
            IRTemplateSegment::Interpolation(Box::new(IRNode::Variable("name".to_string()))),
            IRTemplateSegment::Text(", you are ".to_string()),
            IRTemplateSegment::Interpolation(Box::new(IRNode::FunctionCall {
                name: "calculate_age".to_string(),
                args: vec![IRNode::Variable("birth_year".to_string())],
                effect_kind: None,
                source_span: None,
            })),
            IRTemplateSegment::Text(" years old".to_string()),
        ]);

        let lowered = lowering.lower_node(&template, "greeting");

        // Verify the structure is preserved
        match lowered {
            IRNode::TemplateString(segments) => {
                assert_eq!(segments.len(), 5);

                // First segment: literal text
                match &segments[0] {
                    IRTemplateSegment::Text(t) => assert_eq!(t, "Hello "),
                    _ => panic!("Expected Text segment"),
                }

                // Second segment: variable interpolation (should be unchanged)
                match &segments[1] {
                    IRTemplateSegment::Interpolation(node) => {
                        match **node {
                            IRNode::Variable(ref name) => assert_eq!(name, "name"),
                            _ => panic!("Expected Variable in interpolation"),
                        }
                    }
                    _ => panic!("Expected Interpolation segment"),
                }

                // Third segment: literal text
                match &segments[2] {
                    IRTemplateSegment::Text(t) => assert_eq!(t, ", you are "),
                    _ => panic!("Expected Text segment"),
                }

                // Fourth segment: function call interpolation (should be lowered if it had effects)
                match &segments[3] {
                    IRTemplateSegment::Interpolation(node) => {
                        match **node {
                            IRNode::FunctionCall { ref name, .. } => {
                                assert_eq!(name, "calculate_age");
                            }
                            _ => panic!("Expected FunctionCall in interpolation"),
                        }
                    }
                    _ => panic!("Expected Interpolation segment"),
                }

                // Fifth segment: literal text
                match &segments[4] {
                    IRTemplateSegment::Text(t) => assert_eq!(t, " years old"),
                    _ => panic!("Expected Text segment"),
                }
            }
            _ => panic!("Expected TemplateString node"),
        }
    }
}
