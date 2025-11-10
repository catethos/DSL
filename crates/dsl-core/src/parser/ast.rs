use dsl_types::{Class, Enum, FieldType};
use std::collections::HashMap;

/// Type alias for HTTP configuration: (method, url, params, headers, body)
pub type HttpConfig = (
    String,
    String,
    Option<HashMap<String, String>>,
    Option<HashMap<String, String>>,
    Option<String>,
);

/// Parsed expression AST
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Expr {
    /// String literal
    String(String),
    /// Template string with interpolation
    TemplateString(Vec<TemplateSegment>),
    /// Integer literal
    Int(i64),
    /// Float literal
    Float(f64),
    /// Boolean literal
    Bool(bool),
    /// List literal
    List(Vec<Expr>),
    /// Map literal
    Map(Vec<(String, Expr)>),
    /// Variable reference
    Variable(String),
    /// Function call: name(args)
    FunctionCall { name: String, args: Vec<Expr> },
    /// Type instantiation: TypeName { field: value, ... }
    TypeInstantiation {
        type_name: String,
        fields: Vec<(String, Expr)>,
    },
    /// Field access: expr.field
    FieldAccess { base: Box<Expr>, field: String },
    /// Index access: expr[index]
    IndexAccess { base: Box<Expr>, index: Box<Expr> },
    /// Binary operation: left op right
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    /// Conditional: condition ? then_expr : else_expr
    Conditional {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    /// Sequential composition: left |> right
    Sequential {
        left: Box<Expr>,
        right: Box<Expr>,
        binding: Option<Binding>,
    },
    /// Parallel composition: left || right
    Parallel {
        exprs: Vec<Expr>,
        binding: Option<Binding>,
    },
    /// Match expression: match value { pattern => expr, ... }
    Match {
        scrutinee: Box<Expr>,
        cases: Vec<MatchCase>,
    },
    /// Block expression with multiple statements and a final result
    Block {
        statements: Vec<Expr>, // Executed for side effects (let bindings, etc.)
        result: Box<Expr>,     // Final expression that produces the value
    },
}

/// Template string segments
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateSegment {
    /// Plain text
    Text(String),
    /// Interpolated expression ${expr}
    Interpolation(String), // Store as string to be parsed later
}

/// Variable binding patterns
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Binding {
    /// Single variable: as name
    Single(String),
    /// Destructuring: as [a, b, c]
    List(Vec<String>),
}

/// Pattern matching patterns
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Wildcard pattern: _
    Any,
    /// Literal pattern: 0, "hello", true
    Literal(Box<Expr>),
    /// Variable binding: x, name
    Variable(String),
    /// Binding with nested pattern: x @ pattern
    Binding(String, Box<Pattern>),
    /// Type pattern with constructor: Int(x), Person(p)
    Type {
        type_name: String,
        inner: Option<Box<Pattern>>,
    },
    /// List pattern: [], [a], [a, b], [head, ...tail]
    List {
        patterns: Vec<Pattern>,
        rest: Option<String>, // For ...tail
    },
    /// Map pattern: {}, {x}, {x, y}, {name, age}
    Map {
        fields: Vec<(String, Pattern)>,
        strict: bool, // For now always false (can have extra fields)
    },
    /// Tuple pattern: (a, b, c)
    Tuple(Vec<Pattern>),
}

/// Match case with pattern, optional guard, and body
#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// Function execution modes
#[derive(Debug, Clone)]
pub enum FunctionExecution {
    /// Regular expression-based execution (arrow functions and blocks)
    Expression { body: Box<Expr> },
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
}

/// Function definition (traditional with LLM/HTTP/SQL execution)
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub return_type: Option<FieldType>,
    pub properties: HashMap<String, PropertyValue>,
    pub execution: FunctionExecution,
}

/// Pattern-based function clause (for overloaded functions)
#[derive(Debug, Clone)]
pub struct PatternFunctionClause {
    pub param_patterns: Vec<Pattern>,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// Pattern-based function group (overloaded function)
#[derive(Debug, Clone)]
pub struct PatternFunctionDef {
    pub name: String,
    pub clauses: Vec<PatternFunctionClause>,
    pub return_type: Option<FieldType>,
}

/// A declaration in the program
#[derive(Debug, Clone)]
pub enum Declaration {
    Type(Class),
    Enum(Enum),
    Function(Box<FunctionDef>),
    PatternFunction(PatternFunctionDef),
}

/// Complete program with all declarations
#[derive(Debug, Clone)]
pub struct Program {
    pub types: Vec<Class>,
    pub enums: Vec<Enum>,
    pub functions: Vec<FunctionDef>,
    pub pattern_functions: Vec<PatternFunctionDef>,
    pub entry_expr: Option<Expr>,
}

/// Property values in function definitions
#[derive(Debug, Clone)]
pub enum PropertyValue {
    String(String),
    Template(Vec<TemplateSegment>),
    Int(i64),
    Float(f64),
    Bool(bool),
}

/// Helper enum for parsing function declarations
#[derive(Debug, Clone)]
pub(super) enum FunctionOrClause {
    Function(FunctionDef),
    Clause {
        name: String,
        clause: PatternFunctionClause,
        #[allow(dead_code)]
        return_type: Option<FieldType>,
    },
}
