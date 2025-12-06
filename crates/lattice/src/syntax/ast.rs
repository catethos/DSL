//! Abstract Syntax Tree (AST) definitions for Lattice
//!
//! This module defines the AST nodes that represent a parsed Lattice program.


/// Source location for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// Create a span that encompasses both spans
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            column: if self.line <= other.line {
                self.column
            } else {
                other.column
            },
        }
    }
}

/// A node with source location information
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

// ============================================================================
// Program Structure
// ============================================================================

/// A complete Lattice program
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

/// A top-level item in a program
#[derive(Debug, Clone)]
pub enum Item {
    TypeDef(TypeDef),
    EnumDef(EnumDef),
    LlmConfigDecl(LlmConfigDecl),
    FunctionDef(FunctionDef),
    Statement(Stmt),
}

/// OpenRouter provider configuration for routing preferences
#[derive(Debug, Clone, Default)]
pub struct ProviderConfig {
    /// List of provider slugs to try in order (e.g., ["anthropic", "openai"])
    pub order: Option<Vec<String>>,
    /// Only allow these providers
    pub only: Option<Vec<String>>,
    /// Skip these providers
    pub ignore: Option<Vec<String>>,
    /// Whether backup providers activate when primary unavailable (default: true)
    pub allow_fallbacks: Option<bool>,
    /// Route only to providers supporting all request parameters
    pub require_parameters: Option<bool>,
    /// Filter by data retention policies: "allow" or "deny"
    pub data_collection: Option<String>,
    /// Restrict routing to only Zero Data Retention endpoints
    pub zdr: Option<bool>,
    /// Prioritize by "price", "throughput", or "latency"
    pub sort: Option<String>,
    /// Filter by quantization levels (int4, int8, fp8, etc.)
    pub quantizations: Option<Vec<String>>,
}

/// An LLM config declaration (reusable config block)
#[derive(Debug, Clone)]
pub struct LlmConfigDecl {
    pub name: Spanned<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key_env: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<usize>,
    /// OpenRouter provider routing configuration
    pub provider: Option<ProviderConfig>,
    pub span: Span,
}

// ============================================================================
// Type Definitions
// ============================================================================

/// A type (struct) definition
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: Spanned<String>,
    pub fields: Vec<FieldDef>,
    pub span: Span,
}

/// A field in a type definition
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: Spanned<String>,
    pub ty: TypeAnnotation,
    pub description: Option<String>,
    pub span: Span,
}

/// An enum definition
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: Spanned<String>,
    pub variants: Vec<Spanned<String>>,
    pub span: Span,
}

// ============================================================================
// Type Annotations
// ============================================================================

/// A type annotation (with optional `?` for optional types)
#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    pub ty: TypeExpr,
    pub optional: bool,
    pub span: Span,
}

/// A type expression
#[derive(Debug, Clone)]
pub enum TypeExpr {
    /// Primitive types: String, Int, Float, Bool, Null
    Primitive(PrimitiveType),
    /// Named type (class or enum reference)
    Named(String),
    /// List type: [T]
    List(Box<TypeAnnotation>),
    /// Map type: Map<K, V>
    Map(Box<TypeAnnotation>, Box<TypeAnnotation>),
    /// Result type: Result<T, E>
    Result(Box<TypeAnnotation>, Box<TypeAnnotation>),
}

/// Primitive types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    String,
    Int,
    Float,
    Bool,
    Null,
    Path,
}

// ============================================================================
// Function Definitions
// ============================================================================

/// A function definition (regular or LLM)
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: Spanned<String>,
    pub params: Vec<ParamDef>,
    pub return_type: Option<TypeAnnotation>,
    pub body: FunctionBody,
    pub span: Span,
}

/// A function parameter
#[derive(Debug, Clone)]
pub struct ParamDef {
    pub name: Spanned<String>,
    pub ty: TypeAnnotation,
    pub span: Span,
}

/// Function body: either a regular block or LLM config
#[derive(Debug, Clone)]
pub enum FunctionBody {
    /// Regular function with statements/expression
    Block(Block),
    /// LLM function with configuration (boxed to reduce enum size)
    LlmConfig(Box<LlmConfig>),
}

/// LLM function configuration
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Reference to an llm_config declaration (use: config_name)
    pub use_config: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key_env: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<usize>,
    /// OpenRouter provider routing configuration
    pub provider: Option<ProviderConfig>,
    /// Prompt expression - can be a string literal, raw string, or f-string
    pub prompt: Expr,
    pub span: Span,
}

// ============================================================================
// Statements
// ============================================================================

/// A statement
#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

/// Statement kinds
#[derive(Debug, Clone)]
pub enum StmtKind {
    /// Let binding: let x = expr or let x: Type = expr
    Let {
        name: Spanned<String>,
        ty: Option<TypeAnnotation>,
        value: Expr,
    },
    /// Assignment: target = expr
    Assign { target: AssignTarget, value: Expr },
    /// If statement
    If {
        condition: Expr,
        then_branch: Block,
        else_branch: Option<ElseClause>,
    },
    /// While loop
    While { condition: Expr, body: Block },
    /// For loop: for x in collection { }
    For {
        var: Spanned<String>,
        iterable: Expr,
        body: Block,
    },
    /// Return statement
    Return { value: Option<Expr> },
    /// Expression statement
    Expr { expr: Expr },
}

/// Assignment target
#[derive(Debug, Clone)]
pub struct AssignTarget {
    pub base: Spanned<String>,
    pub accessors: Vec<Accessor>,
    pub span: Span,
}

/// Accessor for assignment target
#[derive(Debug, Clone)]
pub enum Accessor {
    Field(String),
    Index(Expr),
}

/// Else clause for if statement
#[derive(Debug, Clone)]
pub enum ElseClause {
    ElseIf(Box<Stmt>),
    Else(Block),
}

/// Else clause for if expression
#[derive(Debug, Clone)]
pub enum IfExprElse {
    ElseIf(Box<Expr>),
    Else(Block),
}

/// A block of statements with optional trailing expression
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Box<Expr>>,
    pub span: Span,
}

// ============================================================================
// Expressions
// ============================================================================

/// An expression
#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

/// Expression kinds
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// Literal value
    Literal(Literal),
    /// Variable reference
    Var(String),
    /// Enum variant constructor: EnumName::Variant
    EnumVariant { enum_name: String, variant: String },
    /// Binary operation
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    /// Unary operation
    Unary { op: UnaryOp, operand: Box<Expr> },
    /// Field access: expr.field
    Field { object: Box<Expr>, field: String },
    /// Index access: expr[index]
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    /// Function call: func(args)
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    /// List literal: [1, 2, 3]
    List(Vec<Expr>),
    /// Map literal: {"key": value}
    Map(Vec<(MapKey, Expr)>),
    /// Struct literal: TypeName { field: value }
    Struct {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    /// If expression: if cond { expr } else { expr }
    If {
        condition: Box<Expr>,
        then_branch: Block,
        else_branch: Option<IfExprElse>,
    },
    /// Match expression
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    /// Lambda: |x, y| expr
    Lambda {
        params: Vec<String>,
        body: Box<LambdaBody>,
    },
    /// Parallel block: parallel { expr1, expr2 }
    Parallel(Vec<Expr>),
    /// Parallel map: parallel_map(collection, |x| expr)
    ParallelMap {
        collection: Box<Expr>,
        mapper: Box<Expr>,
    },
    /// Map column: map_column(table, "input_col", "output_col", |val| expr)
    MapColumn {
        table: Box<Expr>,
        input_col: Box<Expr>,
        output_col: Box<Expr>,
        mapper: Box<Expr>,
    },
    /// Map row: map_row(table, "output_col", |row| expr) - lambda receives entire row
    MapRow {
        table: Box<Expr>,
        output_col: Box<Expr>,
        mapper: Box<Expr>,
    },
    /// Explode: explode(table, "column") or explode(table, "column", "prefix")
    /// Expands nested map keys into separate columns
    Explode {
        table: Box<Expr>,
        column: Box<Expr>,
        prefix: Option<Box<Expr>>,
    },
    /// SQL query: SQL("query") or SQL<Type>("query")
    Sql {
        ty: Option<TypeAnnotation>,
        query: Box<Expr>,
    },
    /// Grouped expression: (expr)
    Grouped(Box<Expr>),
    /// Block expression: { stmts; expr }
    Block(Block),
    /// F-string (interpolated string): f"hello {name}"
    FString(Vec<FStringPart>),
    /// Dollar field access: $field or $["field"]
    /// Represents an implicit row accessor that becomes a lambda when desugared
    DollarField(Box<Expr>),
}

/// Map key (can be string literal or identifier)
#[derive(Debug, Clone)]
pub enum MapKey {
    String(String),
    Ident(String),
}

/// Lambda body
#[derive(Debug, Clone)]
pub enum LambdaBody {
    Expr(Expr),
    Block(Block),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
}

impl BinaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl UnaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
        }
    }
}

// ============================================================================
// Match Arms and Patterns
// ============================================================================

/// A match arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: MatchArmBody,
    pub span: Span,
}

/// Match arm body
#[derive(Debug, Clone)]
pub enum MatchArmBody {
    Expr(Expr),
    Block(Block),
}

/// A pattern in a match arm
#[derive(Debug, Clone)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

/// Pattern kinds
#[derive(Debug, Clone)]
pub enum PatternKind {
    /// Ok(x) or Err(e)
    Result { is_ok: bool, binding: String },
    /// EnumName::Variant
    Enum { enum_name: String, variant: String },
    /// Literal value
    Literal(Literal),
    /// Wildcard: _
    Wildcard,
    /// Variable binding
    Binding(String),
}

// ============================================================================
// Literals
// ============================================================================

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

/// Part of an f-string (interpolated string)
#[derive(Debug, Clone)]
pub enum FStringPart {
    /// Literal text portion
    Text(String),
    /// Interpolated expression: {expr}
    Expr(Expr),
}

impl Literal {
    pub fn type_name(&self) -> &'static str {
        match self {
            Literal::Int(_) => "Int",
            Literal::Float(_) => "Float",
            Literal::String(_) => "String",
            Literal::Bool(_) => "Bool",
            Literal::Null => "Null",
        }
    }
}
