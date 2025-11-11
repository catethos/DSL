use serde::{Deserialize, Serialize};
use dsl_types::{Class, Enum, FieldType};
use std::collections::HashMap;

/// Effect kind for tracking side-effecting operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EffectKind {
    /// LLM API call (e.g., OpenAI, Anthropic)
    LLM,
    /// HTTP request (GET, POST, etc.)
    HTTP,
    /// SQL query execution
    SQL,
    /// Pure computation (no side effects)
    Pure,
}

/// Source span for error reporting and debugging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

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
    ///
    /// Effect kind tracks the side effects of this call (LLM, HTTP, SQL, Pure).
    /// Source span enables better error messages by linking back to the original source.
    FunctionCall {
        name: String,
        args: Vec<IRNode>,
        effect_kind: Option<EffectKind>,
        source_span: Option<Span>,
    },
    /// Type instantiation: TypeName { field: value, ... }
    TypeInstantiation {
        type_name: String,
        fields: Vec<(String, IRNode)>,
    },
    /// Field access: expr.field
    FieldAccess { base: Box<IRNode>, field: String },
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
    /// Block expression with multiple statements and a final result
    Block {
        statements: Vec<IRNode>, // Executed for side effects (let bindings, etc.)
        result: Box<IRNode>,     // Final expression that produces the value
    },

    // Pattern matching (Phase 10B)
    /// Match expression: match value { pattern => expr, ... }
    Match {
        scrutinee: Box<IRNode>,
        cases: Vec<IRMatchCase>,
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
    ReceiveMessage { pattern: IRPattern },
    /// Broadcast message to multiple agents
    Broadcast {
        targets: Vec<String>,
        message: Box<IRNode>,
    },

    // Control flow (Phase 2)
    /// Infinite loop
    Loop { body: Box<IRNode> },
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
    Break { value: Option<Box<IRNode>> },
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
    Throw { error: Box<IRNode> },

    // Higher-order functions (Phase 11)
    /// Inline lambda expression: fn x => expr end
    /// Can be passed as argument to higher-order functions like map/filter
    Lambda(LambdaIR),
}

/// Template string segments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRTemplateSegment {
    /// Plain text
    Text(String),
    /// Interpolated expression ${expr}
    /// Now stores the compiled IR node instead of a string to avoid re-parsing
    Interpolation(Box<IRNode>),
}

/// Variable binding patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRBinding {
    /// Single variable: as name
    Single(String),
    /// Destructuring: as [a, b, c]
    List(Vec<String>),
}

/// Lambda (anonymous function) IR
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LambdaIR {
    /// Parameter names
    pub params: Vec<String>,
    /// Function body expression
    pub body: Box<IRNode>,
}

/// Match case with pattern, optional guard, and body
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IRMatchCase {
    pub pattern: IRPattern,
    pub guard: Option<Box<IRNode>>, // Optional: if condition
    pub body: Box<IRNode>,
}

/// Function execution modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRExecution {
    /// Expression-based execution (general-purpose functions)
    Expression { body: Box<IRNode> },
    /// LLM-based execution with prompt
    LLM {
        prompt: Box<IRNode>,  // Changed from String to IRNode to support templates
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

/// Function with multiple clauses (overloading/pattern matching)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IRFunctionGroup {
    pub name: String,
    pub clauses: Vec<IRFunctionClause>,
    pub return_type: Option<FieldType>,
}

/// Single clause in an overloaded function
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IRFunctionClause {
    pub param_patterns: Vec<IRPattern>,
    pub guard: Option<Box<IRNode>>,
    pub body: Box<IRNode>,
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
    /// Overloaded functions (multiple clauses with pattern matching)
    pub function_groups: Vec<IRFunctionGroup>,
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

/// Pattern matching - general language feature
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRPattern {
    /// Wildcard pattern: _
    Any,

    /// Literal pattern: 0, "hello", true
    Literal(Box<IRNode>),

    /// Variable binding: x, name
    Variable(String),

    /// Binding with nested pattern: x @ pattern
    Binding(String, Box<IRPattern>),

    /// Type pattern: String, Int(x), Person(p)
    Type {
        type_name: String,
        inner: Option<Box<IRPattern>>,
    },

    /// List pattern: [], [a], [a, b], [head, ...tail]
    List {
        patterns: Vec<IRPattern>,
        rest: Option<String>, // For ...tail
    },

    /// Map pattern: {}, {x}, {x, y}, {name, age}
    Map {
        fields: Vec<(String, IRPattern)>,
        strict: bool, // true = must match exactly, false = can have extra fields
    },

    /// Tuple pattern: (a, b, c)
    Tuple(Vec<IRPattern>),
}

/// Context store for shared agent state (Phase 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRContextStore {
    pub name: String,
    pub schema: Class,
    pub read_permissions: Vec<String>,
    pub write_permissions: Vec<String>,
}
