//! Bytecode definitions for the Lattice VM
//!
//! This module defines the instruction set for the stack-based virtual machine.

use crate::types::{FieldType, Value};
use serde::Serialize;
use std::collections::HashMap;

/// Bytecode instructions for the Lattice VM
///
/// The VM is stack-based: most operations pop operands from the stack
/// and push results back.
///
/// All variants use only Copy types (usize) for zero-cost passing.
/// String references use indices into the Chunk's string intern table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    // ========================================================================
    // Stack Operations
    // ========================================================================
    /// Push a constant from the constant pool onto the stack
    Const(usize),
    /// Pop and discard the top of the stack
    Pop,
    /// Pop N values from below the top of stack, preserving the top value
    /// Used for cleaning up scope locals while keeping return value
    PopBelow(usize),
    /// Duplicate the top of the stack
    Dup,

    // ========================================================================
    // Variables
    // ========================================================================
    /// Get a local variable by slot index
    GetLocal(usize),
    /// Set a local variable by slot index (pops value from stack)
    SetLocal(usize),
    /// Get a global variable by name (index into string intern table)
    GetGlobal(usize),
    /// Set a global variable by name (index into string intern table)
    SetGlobal(usize),

    // ========================================================================
    // Arithmetic
    // ========================================================================
    /// Add: pop two values, push their sum
    Add,
    /// Subtract: pop two values, push (a - b) where b is top of stack
    Sub,
    /// Multiply: pop two values, push their product
    Mul,
    /// Divide: pop two values, push (a / b) where b is top of stack
    Div,
    /// Modulo: pop two values, push (a % b) where b is top of stack
    Mod,
    /// Negate: pop one value, push its negation
    Neg,

    // ========================================================================
    // Comparison
    // ========================================================================
    /// Equal: pop two values, push true if equal
    Eq,
    /// Not equal: pop two values, push true if not equal
    Ne,
    /// Less than: pop two values, push true if a < b
    Lt,
    /// Less than or equal: pop two values, push true if a <= b
    Le,
    /// Greater than: pop two values, push true if a > b
    Gt,
    /// Greater than or equal: pop two values, push true if a >= b
    Ge,

    // ========================================================================
    // Logic
    // ========================================================================
    /// Logical not: pop one value, push its logical negation
    Not,
    /// Logical and: pop two values, push their logical and
    And,
    /// Logical or: pop two values, push their logical or
    Or,

    // ========================================================================
    // Control Flow
    // ========================================================================
    /// Unconditional jump to instruction at offset
    Jump(usize),
    /// Jump to instruction at offset if top of stack is false (pops condition)
    JumpIfFalse(usize),
    /// Jump to instruction at offset if top of stack is true (pops condition)
    JumpIfTrue(usize),

    // ========================================================================
    // Functions
    // ========================================================================
    /// Call a function with N arguments (args are on stack, function ref on top)
    Call(usize),
    /// Call a built-in/native function by name (string index, arg count)
    CallNative(usize, usize),
    /// Call a user-defined function by name (string index, arg count)
    CallUser(usize, usize),
    /// Return from the current function (pops return value from stack)
    Return,

    // ========================================================================
    // Collections
    // ========================================================================
    /// Create a list from the top N stack items
    MakeList(usize),
    /// Create a map from the top N*2 stack items (key, value pairs)
    MakeMap(usize),
    /// Index into a collection: pop index and collection, push element
    Index,
    /// Set index in a collection: pop value, index, and collection
    IndexSet,

    // ========================================================================
    // Structs/Objects
    // ========================================================================
    /// Create a struct instance: pop N field values, push struct
    /// First usize is type name index, second is number of fields
    MakeStruct(usize, usize),
    /// Get a field from a struct: pop struct, push field value (string index)
    GetField(usize),
    /// Set a field in a struct: pop value and struct, push modified struct (string index)
    SetField(usize),

    // ========================================================================
    // Async / Special Operations
    // ========================================================================
    /// Call an LLM function by name (string index)
    /// Pops N arguments from the stack (based on function's parameter count)
    /// Pushes Result<Value, LLMError> onto the stack
    LlmCall(usize),
    /// Execute a SQL query via DuckDB
    /// Pops query string from the stack
    /// Pushes List<Map<String, Value>> (rows) onto the stack
    SqlQuery,
    /// Execute a SQL query with expected return type (string index for type name)
    /// Pops query string from the stack
    /// Pushes List<T> where T is the specified type
    SqlQueryTyped(usize),
    /// Begin parallel execution block - pops N async values, executes in parallel
    /// Pushes List of results onto the stack
    Parallel(usize),
    /// Parallel map over a collection
    /// Pops function reference and collection from stack
    /// Pushes List of mapped results
    ParallelMap,
    /// Specialized parallel map for LLM functions (compiler optimization)
    /// Pops collection from stack, calls LLM function on each item in parallel
    /// String index is the LLM function name
    /// Pushes List of results onto the stack
    ParallelLlmMap(usize),
    /// Map a column: apply a function to each row's input column value,
    /// adding the result as a new output column
    /// Pops: mapper function name, output_col, input_col, table
    /// Pushes: new table with output column added to each row
    MapColumn,
    /// Specialized map column for LLM functions (compiler optimization)
    /// String index is the LLM function name
    /// Pops: output_col, input_col, table from stack
    /// Pushes: new table with output column added (LLM results in parallel)
    MapColumnLlm(usize),
    /// Map a row: apply a function to each entire row,
    /// adding the result as a new output column
    /// Pops: mapper function, output_col, table
    /// Pushes: new table with output column added to each row
    MapRow,
    /// Specialized map row for LLM functions (compiler optimization)
    /// First usize is the LLM function name string index
    /// Second usize is the column mappings string index (comma-separated column names)
    /// The column mappings list the row keys to use for each function parameter, in order
    /// Pops: output_col, table from stack
    /// Pushes: new table with output column added (LLM results in parallel)
    MapRowLlm(usize, usize),
    /// Explode nested map column into separate columns
    /// Pops: prefix (or null), column_name, table from stack
    /// Pushes: new table with nested map keys as separate columns
    Explode,
    /// Await an async/future value
    /// Pops async value from stack, pushes resolved value
    Await,

    // ========================================================================
    // Misc Special Operations
    // ========================================================================
    /// No operation (useful for patching jumps)
    Nop,
    /// Print the top of the stack (for debugging / REPL)
    Print,
    /// Convert the top of stack to a string
    /// Used for f-string interpolation
    Stringify,
}

/// A chunk of bytecode with its associated constant pool and string intern table
#[derive(Debug, Clone, Default)]
pub struct Chunk {
    /// The bytecode instructions
    pub code: Vec<OpCode>,
    /// Constant pool for literal values
    pub constants: Vec<Value>,
    /// Source line numbers for each instruction (for error reporting)
    pub lines: Vec<usize>,
    /// Interned strings table (for global names, field names, function names, etc.)
    pub strings: Vec<String>,
    /// Reverse lookup: string -> index (for deduplication during compilation)
    #[allow(clippy::type_complexity)]
    string_indices: HashMap<String, usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    /// Write an instruction to the chunk
    pub fn write(&mut self, op: OpCode, line: usize) {
        self.code.push(op);
        self.lines.push(line);
    }

    /// Add a constant to the pool and return its index
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    /// Intern a string and return its index
    /// If the string already exists, returns the existing index
    pub fn intern_string(&mut self, s: &str) -> usize {
        if let Some(&idx) = self.string_indices.get(s) {
            return idx;
        }
        let idx = self.strings.len();
        self.strings.push(s.to_string());
        self.string_indices.insert(s.to_string(), idx);
        idx
    }

    /// Get a string by index
    pub fn get_string(&self, idx: usize) -> Option<&str> {
        self.strings.get(idx).map(|s| s.as_str())
    }

    /// Get the current instruction count
    pub fn len(&self) -> usize {
        self.code.len()
    }

    /// Check if the chunk is empty
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    /// Patch a jump instruction at the given offset to jump to the current position
    pub fn patch_jump(&mut self, offset: usize) {
        let jump_target = self.code.len();
        match &mut self.code[offset] {
            OpCode::Jump(target) | OpCode::JumpIfFalse(target) | OpCode::JumpIfTrue(target) => {
                *target = jump_target;
            }
            _ => panic!("Tried to patch a non-jump instruction"),
        }
    }
}

/// A compiled function
#[derive(Debug, Clone)]
pub struct CompiledFunction {
    /// Function name
    pub name: String,
    /// Number of parameters
    pub arity: usize,
    /// Number of local variable slots needed
    pub local_count: usize,
    /// The function's bytecode
    pub chunk: Chunk,
}

impl CompiledFunction {
    pub fn new(name: String, arity: usize) -> Self {
        Self {
            name,
            arity,
            local_count: 0,
            chunk: Chunk::new(),
        }
    }
}

/// OpenRouter provider configuration for routing preferences
/// This is serialized directly to the API request body
#[derive(Debug, Clone, Default, Serialize)]
pub struct ProviderConfig {
    /// List of provider slugs to try in order (e.g., ["anthropic", "openai"])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
    /// Only allow these providers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<Vec<String>>,
    /// Skip these providers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,
    /// Whether backup providers activate when primary unavailable (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    /// Route only to providers supporting all request parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,
    /// Filter by data retention policies: "allow" or "deny"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<String>,
    /// Restrict routing to only Zero Data Retention endpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zdr: Option<bool>,
    /// Prioritize by "price", "throughput", or "latency"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    /// Filter by quantization levels (int4, int8, fp8, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantizations: Option<Vec<String>>,
}

/// A compiled LLM function definition
///
/// LLM functions are defined with special syntax:
/// ```lattice
/// def AnalyzeText(text: String) -> Result<Analysis, LLMError> {
///   base_url: "https://openrouter.ai/api/v1"
///   model: "anthropic/claude-3.5-sonnet"
///   api_key_env: "OPENROUTER_API_KEY"
///   temperature: 0.7
///   provider: {
///     order: ["anthropic", "openai"],
///     allow_fallbacks: false
///   }
///   prompt: """
///     Analyze this text: ${text}
///   """
/// }
/// ```
#[derive(Debug, Clone)]
pub struct LlmFunction {
    /// Function name (e.g., "AnalyzeText")
    pub name: String,
    /// Base URL for the LLM API (e.g., "https://openrouter.ai/api/v1")
    pub base_url: String,
    /// Model identifier (e.g., "anthropic/claude-3.5-sonnet")
    pub model: String,
    /// Environment variable name containing the API key
    pub api_key_env: String,
    /// Temperature for sampling (0.0 - 2.0, default varies by provider)
    pub temperature: Option<f64>,
    /// Maximum tokens to generate (optional)
    pub max_tokens: Option<usize>,
    /// OpenRouter provider routing configuration
    pub provider: Option<ProviderConfig>,
    /// The prompt template with ${variable} interpolation (legacy, kept for simple templates)
    pub prompt_template: String,
    /// Compiled bytecode for prompt generation (used when prompt contains complex expressions)
    /// When present, this is executed to generate the prompt string at runtime.
    /// The bytecode should leave a single String value on the stack.
    pub prompt_chunk: Option<Chunk>,
    /// The return type - used for parsing/coercing LLM response
    /// Can be a primitive, Class("MyType"), Enum("MyEnum"), etc.
    pub return_type: FieldType,
    /// Function parameters: (name, type) pairs for template interpolation
    pub parameters: Vec<(String, FieldType)>,
}

impl LlmFunction {
    /// Create a new LLM function with required fields
    pub fn new(
        name: String,
        base_url: String,
        model: String,
        api_key_env: String,
        prompt_template: String,
        return_type: FieldType,
        parameters: Vec<(String, FieldType)>,
    ) -> Self {
        Self {
            name,
            base_url,
            model,
            api_key_env,
            temperature: None,
            max_tokens: None,
            provider: None,
            prompt_template,
            prompt_chunk: None,
            return_type,
            parameters,
        }
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the max tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set the provider configuration
    pub fn with_provider(mut self, provider: ProviderConfig) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the prompt chunk (compiled bytecode for dynamic prompt generation)
    pub fn with_prompt_chunk(mut self, chunk: Chunk) -> Self {
        self.prompt_chunk = Some(chunk);
        self
    }

    /// Check if this function has a compiled prompt chunk
    pub fn has_prompt_chunk(&self) -> bool {
        self.prompt_chunk.is_some()
    }

    /// Get the number of parameters (arity)
    pub fn arity(&self) -> usize {
        self.parameters.len()
    }

    /// Get the parameter names
    pub fn input_names(&self) -> Vec<String> {
        self.parameters.iter().map(|(name, _)| name.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_write() {
        let mut chunk = Chunk::new();
        chunk.write(OpCode::Const(0), 1);
        chunk.write(OpCode::Const(1), 1);
        chunk.write(OpCode::Add, 1);

        assert_eq!(chunk.len(), 3);
        assert_eq!(chunk.code[0], OpCode::Const(0));
        assert_eq!(chunk.code[1], OpCode::Const(1));
        assert_eq!(chunk.code[2], OpCode::Add);
    }

    #[test]
    fn test_chunk_constants() {
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(42));
        let idx2 = chunk.add_constant(Value::string("hello"));

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(chunk.constants.len(), 2);
    }

    #[test]
    fn test_patch_jump() {
        let mut chunk = Chunk::new();
        chunk.write(OpCode::Const(0), 1);
        chunk.write(OpCode::JumpIfFalse(0), 1); // placeholder
        let jump_offset = chunk.len() - 1;
        chunk.write(OpCode::Const(1), 2);
        chunk.write(OpCode::Add, 2);
        chunk.patch_jump(jump_offset);

        assert_eq!(chunk.code[1], OpCode::JumpIfFalse(4));
    }
}
