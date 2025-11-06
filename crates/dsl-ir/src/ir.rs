use serde::{Deserialize, Serialize};
use simplify_baml::{Class, Enum, FieldType};
use std::collections::HashMap;

/// Intermediate Representation Node - all expression types in the DSL
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRNode {
    /// String literal
    String(String),
    /// Template string with interpolation
    TemplateString(Vec<IRTemplateSegment>),
    /// Integer literal
    Int(i64),
    /// Float literal
    Float(f64),
    /// Boolean literal
    Bool(bool),
    /// List literal
    List(Vec<IRNode>),
    /// Map literal (order-preserving)
    Map(Vec<(String, IRNode)>),
    /// Variable reference
    Variable(String),
    /// Function call: name(args)
    FunctionCall {
        name: String,
        args: Vec<IRNode>,
    },
    /// Type instantiation: TypeName { field: value, ... }
    TypeInstantiation {
        type_name: String,
        fields: Vec<(String, IRNode)>,
    },
    /// Field access: expr.field
    FieldAccess {
        base: Box<IRNode>,
        field: String,
    },
    /// Index access: expr[index]
    IndexAccess {
        base: Box<IRNode>,
        index: Box<IRNode>,
    },
    /// Binary operation: left op right
    BinaryOp {
        left: Box<IRNode>,
        op: String,
        right: Box<IRNode>,
    },
    /// Conditional: condition ? then_expr : else_expr
    Conditional {
        condition: Box<IRNode>,
        then_expr: Box<IRNode>,
        else_expr: Box<IRNode>,
    },
    /// Sequential composition: left |> right
    Sequential {
        left: Box<IRNode>,
        right: Box<IRNode>,
        binding: Option<IRBinding>,
    },
    /// Parallel composition: left || right
    Parallel {
        exprs: Vec<IRNode>,
        binding: Option<IRBinding>,
    },

    // Agent primitives (Phase 2)
    /// Spawn a new agent instance
    SpawnAgent {
        agent_type: String,
        init_state: Box<IRNode>,
    },
    /// Send a message to an agent (fire-and-forget)
    SendMessage {
        target: String,
        message: Box<IRNode>,
    },
    /// Call an agent and wait for reply
    CallAgent {
        target: String,
        message: Box<IRNode>,
        timeout_ms: Option<u32>,
    },
    /// Receive a message matching a pattern
    ReceiveMessage {
        pattern: IRPattern,
    },
    /// Broadcast message to multiple agents
    Broadcast {
        targets: Vec<String>,
        message: Box<IRNode>,
    },

    // Control flow (Phase 2)
    /// Infinite loop
    Loop {
        body: Box<IRNode>,
    },
    /// While loop with condition
    While {
        condition: Box<IRNode>,
        body: Box<IRNode>,
    },
    /// For loop over iterable
    For {
        var: String,
        iterable: Box<IRNode>,
        body: Box<IRNode>,
    },
    /// Break from loop with optional value
    Break {
        value: Option<Box<IRNode>>,
    },
    /// Continue to next iteration
    Continue,

    // Error handling (Phase 2)
    /// Try-catch block
    TryBlock {
        body: Box<IRNode>,
        catch_var: String,
        catch_body: Box<IRNode>,
    },
    /// Throw an error
    Throw {
        error: Box<IRNode>,
    },
}

/// Template string segments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRTemplateSegment {
    /// Plain text
    Text(String),
    /// Interpolated expression ${expr}
    Interpolation(String), // Store as string to be parsed later
}

/// Variable binding patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRBinding {
    /// Single variable: as name
    Single(String),
    /// Destructuring: as [a, b, c]
    List(Vec<String>),
}

/// Function execution modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRExecution {
    /// LLM-based execution with prompt
    LLM {
        prompt: String,
        model: Option<String>,
        base_url: Option<String>,
        api_key_env: Option<String>,
        temperature: Option<f64>,
    },
    /// HTTP request execution
    HTTP {
        method: String,
        url: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
    },
    /// SQL query execution
    SQL { query: String },
    /// Hybrid: HTTP then LLM processing
    HTTPWithLLM {
        http_method: String,
        http_url: String,
        http_params: Option<HashMap<String, String>>,
        http_headers: Option<HashMap<String, String>>,
        llm_prompt: String,
        llm_model: Option<String>,
        llm_base_url: Option<String>,
        llm_api_key_env: Option<String>,
        llm_temperature: Option<f64>,
    },
}

/// Function definition in IR
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<String>,
    pub return_type: Option<FieldType>,
    pub properties: HashMap<String, IRProperty>,
    pub execution: IRExecution,
}

/// Property values in function definitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRProperty {
    String(String),
    Template(Vec<IRTemplateSegment>),
    Int(i64),
    Float(f64),
    Bool(bool),
}

/// Complete IR container for a DSL program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IR {
    /// IR format version
    pub version: String,
    /// User-defined types (classes)
    pub types: Vec<Class>,
    /// User-defined enums
    pub enums: Vec<Enum>,
    /// User-defined functions
    pub functions: Vec<IRFunction>,
    /// Agent definitions (future extension)
    pub agents: Vec<IRAgent>,
    /// Entry point expression
    pub entry_expr: IRNode,
}

/// Agent definition (Phase 2 - future)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRAgent {
    pub name: String,
    pub description: Option<String>,
    pub state_type: Class,
    pub tools: Vec<String>,
    pub handlers: Vec<IRMessageHandler>,
}

/// Message handler for agents (Phase 2 - future)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRMessageHandler {
    pub message_type: FieldType,
    pub reply_type: Option<FieldType>,
    pub body: IRNode,
}

/// Pattern matching for message reception (Phase 2)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRPattern {
    /// Match by type
    Type(FieldType),
    /// Bind to variable with pattern
    Binding(String, Box<IRPattern>),
    /// Match any message
    Any,
}

/// Context store for shared agent state (Phase 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRContextStore {
    pub name: String,
    pub schema: Class,
    pub read_permissions: Vec<String>,
    pub write_permissions: Vec<String>,
}
