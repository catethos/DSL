//! Virtual Machine implementation for Lattice
//!
//! This module defines the VM struct, call frames, and stack operations
//! for executing bytecode.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::runtime::Runtime;
use tokio::sync::Semaphore;

use crate::error::{LatticeError, Result};
use crate::runtime::providers::{
    BoxedLlmProvider, BoxedSqlProvider, DefaultLlmProvider, LlmRequest, ProviderRouting,
};
#[cfg(not(feature = "sql"))]
use crate::runtime::providers::NoSqlProvider;
#[cfg(feature = "sql")]
use crate::runtime::providers::DuckDbProvider;
#[cfg(feature = "sql")]
use crate::sql::SqlContext;
use crate::types::{Value, IR};

use super::bytecode::{Chunk, CompiledFunction, LlmFunction, OpCode};

/// Maximum stack depth to prevent runaway recursion
const MAX_STACK_SIZE: usize = 65536;
/// Maximum call frame depth
const MAX_FRAMES: usize = 256;

/// A call frame represents a single function invocation
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// The function being executed
    pub function: CompiledFunction,
    /// Instruction pointer within this function's chunk
    pub ip: usize,
    /// Base pointer: index into the VM's stack where this frame's locals start
    pub base_pointer: usize,
}

impl CallFrame {
    /// Create a new call frame for a function
    pub fn new(function: CompiledFunction, base_pointer: usize) -> Self {
        Self {
            function,
            ip: 0,
            base_pointer,
        }
    }
}

/// Debug information from an LLM call
#[derive(Debug, Clone, Default)]
pub struct LlmDebugInfo {
    /// The function name that was called
    pub function_name: String,
    /// The return type as a string
    pub return_type: String,
    /// The generated prompt sent to the LLM
    pub prompt: String,
    /// The raw response from the LLM
    pub raw_response: String,
}

/// The Lattice Virtual Machine
///
/// A stack-based bytecode interpreter that executes compiled Lattice code.
/// The VM maintains:
/// - A value stack for operands and intermediate results
/// - A call stack for function invocations
/// - Global state that persists across cell executions
/// - Type registry (IR) for user-defined types
/// - LLM function registry for LLM calls
/// - Injectable providers for LLM and SQL operations
pub struct VM {
    /// The value stack
    stack: Vec<Value>,
    /// The call frame stack
    frames: Vec<CallFrame>,
    /// Global variables (persists across cell executions)
    globals: HashMap<String, Value>,
    /// Type registry: user-defined classes, enums, and functions
    ir: IR,
    /// LLM function registry (compiled LLM function definitions)
    llm_functions: Vec<LlmFunction>,
    /// Compiled user functions (bytecode functions)
    user_functions: HashMap<String, CompiledFunction>,
    /// Injectable LLM provider for LLM calls
    llm_provider: BoxedLlmProvider,
    /// Injectable SQL provider for database queries
    sql_provider: BoxedSqlProvider,
    /// SQL execution context (DuckDB connection) - kept for backward compatibility
    #[cfg(feature = "sql")]
    sql_context: SqlContext,
    /// Debug info from the last LLM call (if any)
    last_llm_debug: Option<LlmDebugInfo>,
    /// Shared tokio runtime for async operations (LLM calls)
    runtime: Runtime,
    /// Shared HTTP client for connection pooling
    http_client: reqwest::Client,
    /// Maximum concurrent LLM calls (None = unlimited)
    max_concurrent_llm_calls: Option<usize>,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    /// Create a new VM instance with default providers
    ///
    /// This creates a VM with:
    /// - DefaultLlmProvider for LLM calls (uses HTTP)
    /// - DuckDbProvider for SQL queries (if sql feature enabled)
    /// - NoSqlProvider if sql feature disabled
    pub fn new() -> Self {
        // Create default LLM provider
        let llm_provider: BoxedLlmProvider = Arc::new(
            DefaultLlmProvider::new().expect("Failed to create default LLM provider"),
        );

        // Create default SQL provider
        #[cfg(feature = "sql")]
        let sql_provider: BoxedSqlProvider = Arc::new(
            DuckDbProvider::new().expect("Failed to create default SQL provider"),
        );
        #[cfg(not(feature = "sql"))]
        let sql_provider: BoxedSqlProvider = Arc::new(NoSqlProvider);

        Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: HashMap::new(),
            ir: IR::new(),
            llm_functions: Vec::new(),
            user_functions: HashMap::new(),
            llm_provider,
            sql_provider,
            #[cfg(feature = "sql")]
            sql_context: SqlContext::default(),
            last_llm_debug: None,
            runtime: Runtime::new().expect("Failed to create tokio runtime"),
            http_client: reqwest::Client::builder()
                .pool_max_idle_per_host(10)
                .build()
                .expect("Failed to create HTTP client"),
            max_concurrent_llm_calls: Some(20),
        }
    }

    /// Create a new VM instance with custom providers
    ///
    /// This is the preferred constructor when embedding Lattice, as it allows
    /// injecting custom LLM and SQL providers.
    pub fn with_providers(
        llm_provider: BoxedLlmProvider,
        sql_provider: BoxedSqlProvider,
    ) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: HashMap::new(),
            ir: IR::new(),
            llm_functions: Vec::new(),
            user_functions: HashMap::new(),
            llm_provider,
            sql_provider,
            #[cfg(feature = "sql")]
            sql_context: SqlContext::default(),
            last_llm_debug: None,
            runtime: Runtime::new().expect("Failed to create tokio runtime"),
            http_client: reqwest::Client::builder()
                .pool_max_idle_per_host(10)
                .build()
                .expect("Failed to create HTTP client"),
            max_concurrent_llm_calls: Some(20),
        }
    }

    /// Create a new VM instance with a file-based DuckDB database
    #[cfg(feature = "sql")]
    pub fn with_database(path: &str) -> Result<Self> {
        let sql_context = SqlContext::open(path).map_err(|e| {
            LatticeError::Runtime(format!("Failed to open database: {}", e))
        })?;

        // Create default LLM provider
        let llm_provider: BoxedLlmProvider = Arc::new(
            DefaultLlmProvider::new().map_err(|e| {
                LatticeError::Runtime(format!("Failed to create LLM provider: {}", e))
            })?,
        );

        // Create SQL provider from the file-based connection
        let sql_provider: BoxedSqlProvider = Arc::new(
            DuckDbProvider::with_path(path).map_err(|e| {
                LatticeError::Runtime(format!("Failed to create SQL provider: {}", e))
            })?,
        );

        Ok(Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: HashMap::new(),
            ir: IR::new(),
            llm_functions: Vec::new(),
            user_functions: HashMap::new(),
            llm_provider,
            sql_provider,
            sql_context,
            last_llm_debug: None,
            runtime: Runtime::new().map_err(|e| {
                LatticeError::Runtime(format!("Failed to create tokio runtime: {}", e))
            })?,
            http_client: reqwest::Client::builder()
                .pool_max_idle_per_host(10)
                .build()
                .map_err(|e| {
                    LatticeError::Runtime(format!("Failed to create HTTP client: {}", e))
                })?,
            max_concurrent_llm_calls: Some(20),
        })
    }

    /// Set the maximum number of concurrent LLM calls
    /// None means unlimited, default is 20
    pub fn set_max_concurrent_llm_calls(&mut self, limit: Option<usize>) {
        self.max_concurrent_llm_calls = limit;
    }

    /// Get the current max concurrent LLM calls setting
    pub fn max_concurrent_llm_calls(&self) -> Option<usize> {
        self.max_concurrent_llm_calls
    }

    /// Get the debug info from the last LLM call (if any)
    pub fn last_llm_debug(&self) -> Option<&LlmDebugInfo> {
        self.last_llm_debug.as_ref()
    }

    /// Take the debug info from the last LLM call, clearing it
    pub fn take_llm_debug(&mut self) -> Option<LlmDebugInfo> {
        self.last_llm_debug.take()
    }

    /// Get a reference to the SQL context
    #[cfg(feature = "sql")]
    pub fn sql_context(&self) -> &SqlContext {
        &self.sql_context
    }

    /// Get a mutable reference to the SQL context
    #[cfg(feature = "sql")]
    pub fn sql_context_mut(&mut self) -> &mut SqlContext {
        &mut self.sql_context
    }

    // ========================================================================
    // Provider Operations
    // ========================================================================

    /// Get a reference to the LLM provider
    pub fn llm_provider(&self) -> &BoxedLlmProvider {
        &self.llm_provider
    }

    /// Get a reference to the SQL provider
    pub fn sql_provider(&self) -> &BoxedSqlProvider {
        &self.sql_provider
    }

    /// Set a new LLM provider
    pub fn set_llm_provider(&mut self, provider: BoxedLlmProvider) {
        self.llm_provider = provider;
    }

    /// Set a new SQL provider
    pub fn set_sql_provider(&mut self, provider: BoxedSqlProvider) {
        self.sql_provider = provider;
    }

    // ========================================================================
    // Registry Operations
    // ========================================================================

    /// Get a reference to the type registry (IR)
    pub fn ir(&self) -> &IR {
        &self.ir
    }

    /// Get a mutable reference to the type registry (IR)
    pub fn ir_mut(&mut self) -> &mut IR {
        &mut self.ir
    }

    /// Register an LLM function and return its index
    pub fn register_llm_function(&mut self, func: LlmFunction) -> usize {
        let idx = self.llm_functions.len();
        self.llm_functions.push(func);
        idx
    }

    /// Get an LLM function by index
    pub fn get_llm_function(&self, idx: usize) -> Option<&LlmFunction> {
        self.llm_functions.get(idx)
    }

    /// Get an LLM function by name
    pub fn get_llm_function_by_name(&self, name: &str) -> Option<&LlmFunction> {
        self.llm_functions.iter().find(|f| f.name == name)
    }

    /// Register a compiled user function
    pub fn register_function(&mut self, func: CompiledFunction) {
        self.user_functions.insert(func.name.clone(), func);
    }

    /// Get a compiled user function by name
    pub fn get_function(&self, name: &str) -> Option<&CompiledFunction> {
        self.user_functions.get(name)
    }

    /// Get names of all registered user functions
    pub fn function_names(&self) -> Vec<String> {
        self.user_functions.keys().cloned().collect()
    }

    /// Get names of all registered LLM functions
    pub fn llm_function_names(&self) -> Vec<String> {
        self.llm_functions.iter().map(|f| f.name.clone()).collect()
    }

    // ========================================================================
    // Stack Operations
    // ========================================================================

    /// Push a value onto the stack
    pub fn push(&mut self, value: Value) -> Result<()> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(LatticeError::Runtime("Stack overflow".to_string()));
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop a value from the stack
    pub fn pop(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .ok_or_else(|| LatticeError::Runtime("Stack underflow".to_string()))
    }

    /// Peek at the top of the stack without removing it
    pub fn peek(&self) -> Result<&Value> {
        self.stack
            .last()
            .ok_or_else(|| LatticeError::Runtime("Stack underflow on peek".to_string()))
    }

    /// Peek at a value at a given distance from the top of the stack
    pub fn peek_at(&self, distance: usize) -> Result<&Value> {
        if distance >= self.stack.len() {
            return Err(LatticeError::Runtime(format!(
                "Stack underflow: tried to peek at distance {} with stack size {}",
                distance,
                self.stack.len()
            )));
        }
        Ok(&self.stack[self.stack.len() - 1 - distance])
    }

    /// Get the current stack size
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    // ========================================================================
    // Call Frame Operations
    // ========================================================================

    /// Push a new call frame
    pub fn push_frame(&mut self, frame: CallFrame) -> Result<()> {
        if self.frames.len() >= MAX_FRAMES {
            return Err(LatticeError::Runtime("Call stack overflow".to_string()));
        }
        self.frames.push(frame);
        Ok(())
    }

    /// Pop the current call frame
    pub fn pop_frame(&mut self) -> Result<CallFrame> {
        self.frames
            .pop()
            .ok_or_else(|| LatticeError::Runtime("Call stack underflow".to_string()))
    }

    /// Get a reference to the current call frame
    pub fn current_frame(&self) -> Result<&CallFrame> {
        self.frames
            .last()
            .ok_or_else(|| LatticeError::Runtime("No active call frame".to_string()))
    }

    /// Get a mutable reference to the current call frame
    pub fn current_frame_mut(&mut self) -> Result<&mut CallFrame> {
        self.frames
            .last_mut()
            .ok_or_else(|| LatticeError::Runtime("No active call frame".to_string()))
    }

    /// Get the current frame depth (number of active frames)
    pub fn frame_depth(&self) -> usize {
        self.frames.len()
    }

    // ========================================================================
    // Local Variable Operations
    // ========================================================================

    /// Get a local variable by slot index (relative to current frame's base pointer)
    pub fn get_local(&self, slot: usize) -> Result<&Value> {
        let frame = self.current_frame()?;
        let index = frame.base_pointer + slot;
        self.stack.get(index).ok_or_else(|| {
            LatticeError::Runtime(format!("Invalid local variable slot: {}", slot))
        })
    }

    /// Set a local variable by slot index
    pub fn set_local(&mut self, slot: usize, value: Value) -> Result<()> {
        let frame = self.current_frame()?;
        let index = frame.base_pointer + slot;
        if index >= self.stack.len() {
            return Err(LatticeError::Runtime(format!(
                "Invalid local variable slot: {}",
                slot
            )));
        }
        self.stack[index] = value;
        Ok(())
    }

    // ========================================================================
    // Global Variable Operations
    // ========================================================================

    /// Get a global variable by name
    pub fn get_global(&self, name: &str) -> Result<&Value> {
        self.globals
            .get(name)
            .ok_or_else(|| LatticeError::Runtime(format!("Undefined variable: {}", name)))
    }

    /// Set a global variable by name
    pub fn set_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    /// Check if a global variable exists
    pub fn has_global(&self, name: &str) -> bool {
        self.globals.contains_key(name)
    }

    // ========================================================================
    // State Management
    // ========================================================================

    /// Reset the VM state (clears stack and frames, but preserves globals)
    pub fn reset(&mut self) {
        self.stack.clear();
        self.frames.clear();
    }

    /// Clear all state including globals
    pub fn clear(&mut self) {
        self.stack.clear();
        self.frames.clear();
        self.globals.clear();
        self.ir = IR::new();
        self.llm_functions.clear();
        self.user_functions.clear();
    }

    /// Get all global variable names
    pub fn global_names(&self) -> impl Iterator<Item = &String> {
        self.globals.keys()
    }

    // ========================================================================
    // Execution
    // ========================================================================

    /// Run a chunk of bytecode
    ///
    /// This is the main entry point for executing code. It sets up a top-level
    /// function from the chunk and runs the fetch-decode-execute loop.
    pub fn run(&mut self, chunk: &Chunk) -> Result<Value> {
        // Create a top-level function from the chunk
        let function = CompiledFunction {
            name: "<main>".to_string(),
            arity: 0,
            local_count: 0,
            chunk: chunk.clone(),
        };

        self.run_function(function)
    }

    /// Run a compiled function
    pub fn run_function(&mut self, function: CompiledFunction) -> Result<Value> {
        let frame = CallFrame::new(function, self.stack.len());
        self.push_frame(frame)?;
        self.execute()
    }

    /// Run a compiled function with arguments already pushed to the stack.
    ///
    /// This is used for FFI calls where the caller has already pushed
    /// the arguments to the stack. The base_pointer is set to point
    /// to the first argument.
    ///
    /// # Arguments
    ///
    /// * `function` - The function to call
    /// * `arg_count` - The number of arguments already on the stack
    pub fn run_function_with_args(
        &mut self,
        function: CompiledFunction,
        arg_count: usize,
    ) -> Result<Value> {
        // Validate arity
        if function.arity != arg_count {
            return Err(LatticeError::Runtime(format!(
                "Function '{}' expects {} arguments, got {}",
                function.name, function.arity, arg_count
            )));
        }

        // Set base_pointer to point to the first argument
        let base_pointer = self.stack.len() - arg_count;
        let frame = CallFrame::new(function, base_pointer);
        self.push_frame(frame)?;
        self.execute()
    }

    /// The main fetch-decode-execute loop
    fn execute(&mut self) -> Result<Value> {
        loop {
            // Check bounds and get IP
            let frame_idx = self.frames.len().checked_sub(1).ok_or_else(|| {
                LatticeError::Runtime("No active call frame".to_string())
            })?;

            let ip = self.frames[frame_idx].ip;
            let code_len = self.frames[frame_idx].function.chunk.code.len();

            if ip >= code_len {
                // End of code reached - return Null if nothing on stack
                return if self.stack.is_empty() {
                    Ok(Value::Null)
                } else {
                    self.pop()
                };
            }

            let line = self.frames[frame_idx].function.chunk.lines.get(ip).copied().unwrap_or(0);

            // Advance IP before execution (so jumps work correctly)
            self.frames[frame_idx].ip += 1;

            // Decode and execute - copy the opcode since OpCode is now Copy
            // This avoids borrow issues and allows mutable access to self during execution
            let opcode = self.frames[frame_idx].function.chunk.code[ip];
            match opcode {
                // Stack Operations
                OpCode::Const(idx) => {
                    let value = self.frames[frame_idx]
                        .function
                        .chunk
                        .constants
                        .get(idx)
                        .cloned()
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid constant index: {}", idx))
                        })?;
                    self.push(value)?;
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::PopBelow(n) => self.op_pop_below(n)?,
                OpCode::Dup => self.op_dup()?,

                // Variables
                OpCode::GetLocal(slot) => {
                    let base_pointer = self.frames[frame_idx].base_pointer;
                    let index = base_pointer + slot;
                    let value = self.stack.get(index).cloned().ok_or_else(|| {
                        LatticeError::Runtime(format!("Invalid local variable slot: {}", slot))
                    })?;
                    self.push(value)?;
                }
                OpCode::SetLocal(slot) => {
                    let base_pointer = self.frames[frame_idx].base_pointer;
                    let index = base_pointer + slot;
                    let value = self.peek()?.clone();
                    if index >= self.stack.len() {
                        return Err(LatticeError::Runtime(format!(
                            "Invalid local variable slot: {}",
                            slot
                        )));
                    }
                    self.stack[index] = value;
                }
                OpCode::GetGlobal(name_idx) => {
                    let name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", name_idx))
                        })?;
                    let value = self.globals.get(name).cloned().ok_or_else(|| {
                        LatticeError::Runtime(format!("Undefined variable: {}", name))
                    })?;
                    self.push(value)?;
                }
                OpCode::SetGlobal(name_idx) => {
                    let name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", name_idx))
                        })?
                        .to_string();
                    let value = self.peek()?.clone();
                    self.globals.insert(name, value);
                }

                // Arithmetic
                OpCode::Add => self.op_add()?,
                OpCode::Sub => self.op_sub()?,
                OpCode::Mul => self.op_mul()?,
                OpCode::Div => self.op_div()?,
                OpCode::Mod => self.op_mod()?,
                OpCode::Neg => self.op_neg()?,

                // Comparison
                OpCode::Eq => self.op_eq()?,
                OpCode::Ne => self.op_ne()?,
                OpCode::Lt => self.op_lt()?,
                OpCode::Le => self.op_le()?,
                OpCode::Gt => self.op_gt()?,
                OpCode::Ge => self.op_ge()?,

                // Logic
                OpCode::Not => self.op_not()?,
                OpCode::And => self.op_and()?,
                OpCode::Or => self.op_or()?,

                // Control Flow
                OpCode::Jump(target) => {
                    self.frames[frame_idx].ip = target;
                }
                OpCode::JumpIfFalse(target) => {
                    let condition = self.pop()?;
                    if !is_truthy(&condition) {
                        self.frames[frame_idx].ip = target;
                    }
                }
                OpCode::JumpIfTrue(target) => {
                    let condition = self.pop()?;
                    if is_truthy(&condition) {
                        self.frames[frame_idx].ip = target;
                    }
                }

                // Functions
                OpCode::Call(arg_count) => self.op_call(arg_count)?,
                OpCode::CallNative(name_idx, arg_count) => {
                    let name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", name_idx))
                        })?
                        .to_string();
                    self.op_call_native(&name, arg_count, line)?;
                }
                OpCode::CallUser(name_idx, arg_count) => {
                    let name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", name_idx))
                        })?
                        .to_string();
                    self.op_call_user(&name, arg_count)?;
                }
                OpCode::Return => {
                    if let Some(value) = self.op_return()? {
                        return Ok(value);
                    }
                }

                // Collections
                OpCode::MakeList(count) => self.op_make_list(count)?,
                OpCode::MakeMap(count) => self.op_make_map(count)?,
                OpCode::Index => self.op_index()?,
                OpCode::IndexSet => self.op_index_set()?,

                // Structs
                OpCode::MakeStruct(type_name_idx, field_count) => {
                    let type_name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(type_name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", type_name_idx))
                        })?
                        .to_string();
                    self.op_make_struct(type_name, field_count)?;
                }
                OpCode::GetField(field_name_idx) => {
                    let field_name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(field_name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", field_name_idx))
                        })?
                        .to_string();
                    self.op_get_field(&field_name)?;
                }
                OpCode::SetField(field_name_idx) => {
                    let field_name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(field_name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", field_name_idx))
                        })?
                        .to_string();
                    self.op_set_field(field_name)?;
                }

                // Async / Special Operations
                OpCode::LlmCall(func_name_idx) => {
                    let func_name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(func_name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", func_name_idx))
                        })?
                        .to_string();
                    self.op_llm_call(&func_name)?;
                }
                OpCode::SqlQuery => self.op_sql_query()?,
                OpCode::SqlQueryTyped(type_name_idx) => {
                    let type_name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(type_name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", type_name_idx))
                        })?
                        .to_string();
                    self.op_sql_query_typed(&type_name)?;
                }
                OpCode::Parallel(count) => self.op_parallel(count)?,
                OpCode::ParallelMap => self.op_parallel_map()?,
                OpCode::ParallelLlmMap(func_name_idx) => {
                    let func_name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(func_name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", func_name_idx))
                        })?
                        .to_string();
                    self.op_parallel_llm_map(&func_name)?;
                }
                OpCode::MapColumn => self.op_map_column()?,
                OpCode::MapColumnLlm(func_name_idx) => {
                    let func_name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(func_name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", func_name_idx))
                        })?
                        .to_string();
                    self.op_map_column_llm(&func_name)?;
                }
                OpCode::MapRow => self.op_map_row()?,
                OpCode::MapRowLlm(func_name_idx, column_mappings_idx) => {
                    let func_name = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(func_name_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", func_name_idx))
                        })?
                        .to_string();
                    let column_mappings = self.frames[frame_idx]
                        .function
                        .chunk
                        .get_string(column_mappings_idx)
                        .ok_or_else(|| {
                            LatticeError::Runtime(format!("Invalid string index: {}", column_mappings_idx))
                        })?
                        .to_string();
                    self.op_map_row_llm(&func_name, &column_mappings)?;
                }
                OpCode::Explode => self.op_explode()?,
                OpCode::Await => self.op_await()?,

                // Misc Special
                OpCode::Nop => {}
                OpCode::Print => self.op_print()?,
                OpCode::Stringify => self.op_stringify()?,
            }
        }
    }

    // ========================================================================
    // Stack Operation Handlers
    // ========================================================================

    fn op_dup(&mut self) -> Result<()> {
        let value = self.peek()?.clone();
        self.push(value)
    }

    /// Pop n values from below the top of stack, preserving the top value
    fn op_pop_below(&mut self, n: usize) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        // Save the top value
        let top = self.pop()?;
        // Pop n values
        for _ in 0..n {
            self.pop()?;
        }
        // Push back the top value
        self.push(top)
    }

    // ========================================================================
    // Arithmetic Handlers
    // ========================================================================

    fn op_add(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 + b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a + *b as f64),
            (Value::String(a), Value::String(b)) => Value::string(format!("{}{}", a, b)),
            // Boolean addition: true = 1, false = 0
            (Value::Bool(a), Value::Bool(b)) => Value::Int(*a as i64 + *b as i64),
            (Value::Bool(a), Value::Int(b)) => Value::Int(*a as i64 + b),
            (Value::Int(a), Value::Bool(b)) => Value::Int(a + *b as i64),
            (Value::Bool(a), Value::Float(b)) => Value::Float(*a as i64 as f64 + b),
            (Value::Float(a), Value::Bool(b)) => Value::Float(a + *b as i64 as f64),
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "Cannot add {} and {}",
                    type_name(&a),
                    type_name(&b)
                )))
            }
        };
        self.push(result)
    }

    fn op_sub(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
            (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 - b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a - *b as f64),
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "Cannot subtract {} from {}",
                    type_name(&b),
                    type_name(&a)
                )))
            }
        };
        self.push(result)
    }

    fn op_mul(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
            (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 * b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a * *b as f64),
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "Cannot multiply {} and {}",
                    type_name(&a),
                    type_name(&b)
                )))
            }
        };
        self.push(result)
    }

    fn op_div(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (Value::Int(_), Value::Int(0)) | (Value::Float(_), Value::Int(0)) => {
                return Err(LatticeError::Runtime("Division by zero".to_string()))
            }
            (Value::Int(_), Value::Float(b)) | (Value::Float(_), Value::Float(b))
                if *b == 0.0 =>
            {
                return Err(LatticeError::Runtime("Division by zero".to_string()))
            }
            (Value::Int(a), Value::Int(b)) => Value::Int(a / b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
            (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 / b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a / *b as f64),
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "Cannot divide {} by {}",
                    type_name(&a),
                    type_name(&b)
                )))
            }
        };
        self.push(result)
    }

    fn op_mod(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (Value::Int(_), Value::Int(0)) => {
                return Err(LatticeError::Runtime("Modulo by zero".to_string()))
            }
            (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
            (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 % b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a % *b as f64),
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "Cannot compute modulo of {} and {}",
                    type_name(&a),
                    type_name(&b)
                )))
            }
        };
        self.push(result)
    }

    fn op_neg(&mut self) -> Result<()> {
        let a = self.pop()?;
        let result = match a {
            Value::Int(a) => Value::Int(-a),
            Value::Float(a) => Value::Float(-a),
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "Cannot negate {}",
                    type_name(&a)
                )))
            }
        };
        self.push(result)
    }

    // ========================================================================
    // Comparison Handlers
    // ========================================================================

    fn op_eq(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(Value::Bool(values_equal(&a, &b)))
    }

    fn op_ne(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(Value::Bool(!values_equal(&a, &b)))
    }

    fn op_lt(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = compare_values(&a, &b)?;
        self.push(Value::Bool(result < 0))
    }

    fn op_le(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = compare_values(&a, &b)?;
        self.push(Value::Bool(result <= 0))
    }

    fn op_gt(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = compare_values(&a, &b)?;
        self.push(Value::Bool(result > 0))
    }

    fn op_ge(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = compare_values(&a, &b)?;
        self.push(Value::Bool(result >= 0))
    }

    // ========================================================================
    // Logic Handlers
    // ========================================================================

    fn op_not(&mut self) -> Result<()> {
        let a = self.pop()?;
        self.push(Value::Bool(!is_truthy(&a)))
    }

    fn op_and(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(Value::Bool(is_truthy(&a) && is_truthy(&b)))
    }

    fn op_or(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(Value::Bool(is_truthy(&a) || is_truthy(&b)))
    }

    // ========================================================================
    // Function Handlers
    // ========================================================================

    fn op_call(&mut self, arg_count: usize) -> Result<()> {
        // Pop the function reference (should be a string with function name for now)
        let func_ref = self.pop()?;
        let func_name = match func_ref {
            Value::String(name) => name,
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "Cannot call non-function value: {:?}",
                    func_ref
                )))
            }
        };

        // Look up the function
        let function = self
            .user_functions
            .get(&*func_name)
            .ok_or_else(|| {
                LatticeError::Runtime(format!("Undefined function: {}", func_name))
            })?
            .clone();

        // Verify arity
        if function.arity != arg_count {
            return Err(LatticeError::Runtime(format!(
                "Function '{}' expects {} arguments, got {}",
                func_name, function.arity, arg_count
            )));
        }

        // Set up the call frame
        // Arguments are already on the stack, base_pointer should point to first arg
        let base_pointer = self.stack.len() - arg_count;
        let frame = CallFrame::new(function, base_pointer);
        self.push_frame(frame)?;

        Ok(())
    }

    fn op_call_user(&mut self, name: &str, arg_count: usize) -> Result<()> {
        // Look up the function
        let function = self
            .user_functions
            .get(name)
            .ok_or_else(|| {
                LatticeError::Runtime(format!("Undefined function: {}", name))
            })?
            .clone();

        // Verify arity
        if function.arity != arg_count {
            return Err(LatticeError::Runtime(format!(
                "Function '{}' expects {} arguments, got {}",
                name, function.arity, arg_count
            )));
        }

        // Set up the call frame
        // Arguments are already on the stack, base_pointer should point to first arg
        let base_pointer = self.stack.len() - arg_count;
        let frame = CallFrame::new(function, base_pointer);
        self.push_frame(frame)?;

        Ok(())
    }

    fn op_call_native(&mut self, name: &str, arg_count: usize, _line: usize) -> Result<()> {
        // Collect arguments from the stack
        let mut args = Vec::with_capacity(arg_count);
        for _ in 0..arg_count {
            args.push(self.pop()?);
        }
        args.reverse();

        // Call the native function
        let result = call_native_function(name, args)?;
        self.push(result)
    }

    fn op_return(&mut self) -> Result<Option<Value>> {
        let return_value = self.pop().unwrap_or(Value::Null);
        let frame = self.pop_frame()?;

        // Restore stack to before function call
        self.stack.truncate(frame.base_pointer);

        // If this was the last frame, return the value
        if self.frames.is_empty() {
            return Ok(Some(return_value));
        }

        // Otherwise, push the return value onto the stack
        self.push(return_value)?;
        Ok(None)
    }

    // ========================================================================
    // Collection Handlers
    // ========================================================================

    fn op_make_list(&mut self, count: usize) -> Result<()> {
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(self.pop()?);
        }
        items.reverse();
        self.push(Value::list(items))
    }

    fn op_make_map(&mut self, count: usize) -> Result<()> {
        let mut map = HashMap::new();
        for _ in 0..count {
            let value = self.pop()?;
            let key = self.pop()?;
            let key_str = match key {
                Value::String(s) => s.to_string(),
                _ => {
                    return Err(LatticeError::Runtime(
                        "Map keys must be strings".to_string(),
                    ))
                }
            };
            map.insert(key_str, value);
        }
        self.push(Value::map(map))
    }

    fn op_index(&mut self) -> Result<()> {
        let index = self.pop()?;
        let collection = self.pop()?;

        let result = match (&collection, &index) {
            (Value::List(list), Value::Int(i)) => {
                let idx = if *i < 0 {
                    (list.len() as i64 + *i) as usize
                } else {
                    *i as usize
                };
                list.get(idx).cloned().ok_or_else(|| {
                    LatticeError::Runtime(format!("Index {} out of bounds", i))
                })?
            }
            (Value::Map(map), Value::String(key)) => {
                map.get(&**key).cloned().ok_or_else(|| {
                    LatticeError::Runtime(format!("Key '{}' not found in map", key))
                })?
            }
            (Value::String(s), Value::Int(i)) => {
                let idx = if *i < 0 {
                    (s.len() as i64 + *i) as usize
                } else {
                    *i as usize
                };
                s.chars().nth(idx).map(|c| Value::string(c.to_string())).ok_or_else(|| {
                    LatticeError::Runtime(format!("Index {} out of bounds", i))
                })?
            }
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "Cannot index {} with {}",
                    type_name(&collection),
                    type_name(&index)
                )))
            }
        };
        self.push(result)
    }

    fn op_index_set(&mut self) -> Result<()> {
        let value = self.pop()?;
        let index = self.pop()?;
        let mut collection = self.pop()?;

        match (&mut collection, &index) {
            (Value::List(list), Value::Int(i)) => {
                let idx = if *i < 0 {
                    (list.len() as i64 + *i) as usize
                } else {
                    *i as usize
                };
                if idx >= list.len() {
                    return Err(LatticeError::Runtime(format!(
                        "Index {} out of bounds",
                        i
                    )));
                }
                std::sync::Arc::make_mut(list)[idx] = value;
            }
            (Value::Map(map), Value::String(key)) => {
                std::sync::Arc::make_mut(map).insert(key.to_string(), value);
            }
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "Cannot set index on {} with {}",
                    type_name(&collection),
                    type_name(&index)
                )))
            }
        }
        self.push(collection)
    }

    // ========================================================================
    // Struct Handlers
    // ========================================================================

    fn op_make_struct(&mut self, type_name: String, field_count: usize) -> Result<()> {
        // For simplicity, we represent structs as maps with a special __type field
        let mut fields = HashMap::new();
        fields.insert("__type".to_string(), Value::string(type_name));

        // Pop field values (in reverse order, so we need to collect and reverse)
        let mut field_values = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            let value = self.pop()?;
            let name = self.pop()?;
            match name {
                Value::String(s) => field_values.push((s.to_string(), value)),
                _ => {
                    return Err(LatticeError::Runtime(
                        "Struct field names must be strings".to_string(),
                    ))
                }
            }
        }

        for (name, value) in field_values {
            fields.insert(name, value);
        }

        self.push(Value::map(fields))
    }

    fn op_get_field(&mut self, field_name: &str) -> Result<()> {
        let obj = self.pop()?;
        match obj {
            Value::Map(map) => {
                let value = map.get(field_name).cloned().ok_or_else(|| {
                    LatticeError::Runtime(format!("Field '{}' not found", field_name))
                })?;
                self.push(value)
            }
            _ => Err(LatticeError::Runtime(format!(
                "Cannot get field '{}' from {}",
                field_name,
                type_name(&obj)
            ))),
        }
    }

    fn op_set_field(&mut self, field_name: String) -> Result<()> {
        let value = self.pop()?;
        let mut obj = self.pop()?;
        match &mut obj {
            Value::Map(ref mut map) => {
                std::sync::Arc::make_mut(map).insert(field_name, value);
                self.push(obj)
            }
            _ => Err(LatticeError::Runtime(format!(
                "Cannot set field on {}",
                type_name(&obj)
            ))),
        }
    }

    // ========================================================================
    // Special Operation Handlers
    // ========================================================================

    fn op_print(&mut self) -> Result<()> {
        let value = self.peek()?;
        println!("{}", value);
        Ok(())
    }

    /// Convert the top of stack to a string
    fn op_stringify(&mut self) -> Result<()> {
        let value = self.pop()?;
        let string_value: std::sync::Arc<str> = match value {
            Value::String(s) => s,
            Value::Int(n) => n.to_string().into(),
            Value::Float(f) => f.to_string().into(),
            Value::Bool(b) => b.to_string().into(),
            Value::Null => "null".into(),
            Value::Path(p) => p.display().to_string().into(),
            Value::List(items) => format!("{}", Value::List(items)).into(),
            Value::Map(map) => format!("{}", Value::Map(map)).into(),
        };
        self.push(Value::String(string_value))
    }

    // ========================================================================
    // Async / Special Operation Handlers
    // ========================================================================

    /// Call an LLM function by name
    ///
    /// Pops arguments from the stack, calls the LLM via the injected provider,
    /// parses the response, and pushes the result (or error) onto the stack.
    ///
    /// The LLM call is delegated to `self.llm_provider.call()`, which allows
    /// for injectable implementations (HTTP, host callbacks, mock, etc.).
    fn op_llm_call(&mut self, func_name: &str) -> Result<()> {
        use crate::llm::{extract_template_variables, generate_prompt_from_ir, parse_llm_response_with_ir, generate_schema_from_ir};

        // Get function info first (clone what we need to avoid borrow issues)
        let func = self.get_llm_function_by_name(func_name).ok_or_else(|| {
            LatticeError::Runtime(format!("Undefined LLM function: {}", func_name))
        })?.clone();

        // Collect arguments from the stack
        let mut args = Vec::with_capacity(func.arity());
        for _ in 0..func.arity() {
            args.push(self.pop()?);
        }
        args.reverse();

        // Generate the prompt - either via bytecode execution or template
        let prompt = if let Some(ref prompt_chunk) = func.prompt_chunk {
            // Execute the compiled prompt chunk to generate the prompt string
            let prompt_str = self.execute_prompt_chunk(prompt_chunk, args)?;

            // Append the schema for the return type
            let schema = generate_schema_from_ir(&self.ir, &func.return_type);
            format!("{}\n\n{}", prompt_str, schema)
        } else {
            // Use the template-based approach for simple prompts
            // Extract variable names referenced in the template for selective cloning
            let referenced_vars = extract_template_variables(&func.prompt_template);

            // Build the params HashMap from args and input_names
            // Only clone globals that are actually referenced in the template (optimization)
            let mut params: HashMap<String, Value> = HashMap::with_capacity(
                referenced_vars.len() + func.arity()
            );
            for var_name in &referenced_vars {
                if let Some(value) = self.globals.get(var_name) {
                    params.insert(var_name.clone(), value.clone());
                }
            }

            // Then add function arguments (these override globals if there's a name collision)
            let input_names = func.input_names();
            for (name, value) in input_names.iter().zip(args.into_iter()) {
                params.insert(name.clone(), value);
            }

            // Generate the prompt using template renderer
            generate_prompt_from_ir(
                &self.ir,
                &func.prompt_template,
                &params,
                &func.return_type,
            ).map_err(|e| LatticeError::Runtime(format!("Failed to render prompt: {}", e)))?
        };

        // Initialize debug info
        let mut debug_info = LlmDebugInfo {
            function_name: func_name.to_string(),
            return_type: format!("{:?}", func.return_type),
            prompt: prompt.clone(),
            raw_response: String::new(),
        };

        // Get the API key from environment variable
        let api_key = std::env::var(&func.api_key_env).map_err(|_| {
            LatticeError::Runtime(format!(
                "Environment variable '{}' not set. Please set it to your API key.",
                func.api_key_env
            ))
        })?;

        // Build LlmRequest for the provider
        let mut request = LlmRequest::new(
            func.base_url.clone(),
            func.model.clone(),
            api_key,
            prompt.clone(),
        );

        if let Some(temp) = func.temperature {
            request = request.with_temperature(temp as f32);
        }
        if let Some(max_tok) = func.max_tokens {
            request = request.with_max_tokens(max_tok as u32);
        }
        if let Some(ref provider_config) = func.provider {
            // Convert ProviderConfig to ProviderRouting
            request = request.with_provider(ProviderRouting {
                order: provider_config.order.clone(),
                allow_fallbacks: provider_config.allow_fallbacks,
                require: provider_config.only.clone(),
            });
        }

        // Call the LLM via the injected provider
        let response = self.llm_provider.call(request).map_err(|e| {
            // Store debug info even on error
            self.last_llm_debug = Some(debug_info.clone());
            LatticeError::Runtime(format!("LLM call failed: {}", e))
        })?;

        let raw_response = response.content;

        // Store the raw response in debug info
        debug_info.raw_response = raw_response.clone();
        self.last_llm_debug = Some(debug_info);

        // Parse the response
        let result = parse_llm_response_with_ir(&self.ir, &raw_response, &func.return_type)
            .map_err(|e| LatticeError::Runtime(format!(
                "Failed to parse LLM response: {}. Raw response was: {}",
                e, raw_response
            )))?;

        // Push the result onto the stack
        self.push(result)
    }

    /// Execute a prompt chunk with the given arguments to produce a prompt string.
    ///
    /// This is used when an LLM function has a compiled prompt_chunk (containing
    /// complex expressions like if-expressions) rather than a simple template string.
    ///
    /// # Arguments
    /// * `chunk` - The compiled bytecode chunk for the prompt expression
    /// * `args` - Arguments to pass as local variables (in order matching function params)
    ///
    /// # Returns
    /// The evaluated prompt string
    fn execute_prompt_chunk(&mut self, chunk: &Chunk, args: Vec<Value>) -> Result<String> {
        // Save the current stack position
        let saved_stack_len = self.stack.len();
        let saved_frames_len = self.frames.len();

        // Push arguments as locals (they will be at stack positions 0, 1, 2, ...)
        for arg in args {
            self.push(arg)?;
        }

        // Create a temporary function to hold the chunk
        let function = CompiledFunction {
            name: "<prompt>".to_string(),
            arity: 0, // Args already on stack as locals
            local_count: 0,
            chunk: chunk.clone(),
        };

        // Set up a call frame
        let base_pointer = saved_stack_len;
        let frame = CallFrame::new(function, base_pointer);
        self.push_frame(frame)?;

        // Execute until this frame returns
        let mut result = Value::Null;
        while self.frames.len() > saved_frames_len {
            let frame_idx = self.frames.len() - 1;
            let ip = self.frames[frame_idx].ip;
            let code_len = self.frames[frame_idx].function.chunk.code.len();

            if ip >= code_len {
                // End of chunk - pop the result
                result = self.pop().unwrap_or(Value::Null);
                let base = self.frames[frame_idx].base_pointer;
                self.stack.truncate(base);
                let _ = self.pop_frame();
                break;
            }

            self.frames[frame_idx].ip += 1;
            let op = self.frames[frame_idx].function.chunk.code[ip];

            match op {
                OpCode::Return => {
                    result = self.pop().unwrap_or(Value::Null);
                    let base = self.frames[frame_idx].base_pointer;
                    self.stack.truncate(base);
                    let _ = self.pop_frame();
                }
                _ => {
                    self.execute_single_op(op, frame_idx)?;
                }
            }
        }

        // Convert result to string
        match result {
            Value::String(s) => Ok(s.to_string()),
            other => Ok(format!("{}", other)),
        }
    }

    /// Execute a SQL query via the injected SQL provider
    ///
    /// Pops query string from stack, executes via the SQL provider,
    /// pushes List<Map<String, Value>> (rows) onto stack.
    ///
    /// The SQL call is delegated to `self.sql_provider.query()`, which allows
    /// for injectable implementations (DuckDB, host callbacks, mock, etc.).
    fn op_sql_query(&mut self) -> Result<()> {
        let query = self.pop()?;
        let query_str = match query {
            Value::String(s) => s,
            _ => {
                return Err(LatticeError::Runtime(
                    "SQL query must be a string".to_string(),
                ))
            }
        };

        // Execute query via the injected SQL provider
        let sql_result = self.sql_provider.query(&query_str).map_err(|e| {
            LatticeError::Runtime(format!("SQL query failed: {}", e))
        })?;

        // Convert SqlResult to internal Value (List of Maps)
        let result = sql_result.to_lattice_value().to_internal();

        // Push result onto stack
        self.push(result)
    }

    /// Execute a SQL query with typed results
    ///
    /// Pops query string from stack, executes via the SQL provider,
    /// pushes List<T> where T is the specified type.
    ///
    /// The typed form `SQL<Person>("SELECT * FROM people")` validates
    /// that the result columns match the type definition and adds
    /// a `__type` field to each row.
    fn op_sql_query_typed(&mut self, type_name: &str) -> Result<()> {
        let query = self.pop()?;
        let query_str = match query {
            Value::String(s) => s,
            _ => {
                return Err(LatticeError::Runtime(
                    "SQL query must be a string".to_string(),
                ))
            }
        };

        // Verify type exists in IR and get field info
        let class = self.ir.find_class(type_name).cloned();
        let class = class.ok_or_else(|| {
            LatticeError::Runtime(format!("Unknown type for SQL query: {}", type_name))
        })?;

        // Execute query via the injected SQL provider
        let sql_result = self.sql_provider.query(&query_str).map_err(|e| {
            LatticeError::Runtime(format!("SQL query failed: {}", e))
        })?;

        // Convert SqlResult to internal Value
        let result = sql_result.to_lattice_value().to_internal();

        // Convert result rows to typed structs
        let typed_result = match result {
            Value::List(rows) => {
                let mut typed_rows = Vec::with_capacity(rows.len());
                for row in rows.iter().cloned() {
                    if let Value::Map(map) = row {
                        // Validate fields match the type definition
                        for field in &class.fields {
                            if !field.optional && !map.contains_key(&field.name) {
                                return Err(LatticeError::Runtime(format!(
                                    "SQL result missing required field '{}' for type '{}'",
                                    field.name, type_name
                                )));
                            }
                        }
                        // Add __type field to mark as typed struct
                        let mut new_map = (*map).clone();
                        new_map.insert("__type".to_string(), Value::string(type_name.to_string()));
                        typed_rows.push(Value::map(new_map));
                    } else {
                        return Err(LatticeError::Runtime(
                            "SQL result row is not a map".to_string(),
                        ));
                    }
                }
                Value::list(typed_rows)
            }
            _ => {
                return Err(LatticeError::Runtime(
                    "SQL result is not a list".to_string(),
                ))
            }
        };

        // Push result onto stack
        self.push(typed_result)
    }

    /// Execute N expressions in parallel
    ///
    /// Pops N async/future values from stack, executes them in parallel,
    /// pushes List of results onto stack.
    fn op_parallel(&mut self, count: usize) -> Result<()> {
        let mut _items = Vec::with_capacity(count);
        for _ in 0..count {
            _items.push(self.pop()?);
        }

        // TODO: Implement parallel execution
        // 1. Execute items concurrently (tokio::join! or similar)
        // 2. Collect results
        // 3. Push List onto stack

        Err(LatticeError::Runtime(
            "Parallel execution not yet implemented".to_string(),
        ))
    }

    /// Parallel map over a collection
    ///
    /// Pops function reference and collection from stack,
    /// applies function to each element in parallel,
    /// pushes List of results onto stack.
    fn op_parallel_map(&mut self) -> Result<()> {
        let func_name_value = self.pop()?;
        let collection = self.pop()?;

        // Get function name
        let func_name = match &func_name_value {
            Value::String(s) => s.to_string(),
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "parallel_map mapper must be a function, got {:?}",
                    func_name_value
                )))
            }
        };

        // Verify collection is a List
        let items = match &collection {
            Value::List(list) => list.clone(),
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "parallel_map collection must be a list, got {:?}",
                    collection
                )))
            }
        };

        // Look up the function
        let function = self
            .user_functions
            .get(&func_name)
            .ok_or_else(|| {
                LatticeError::Runtime(format!("Undefined function: {}", func_name))
            })?
            .clone();

        // Verify arity (should be 1 for map)
        if function.arity != 1 {
            return Err(LatticeError::Runtime(format!(
                "parallel_map function '{}' must take exactly 1 argument, takes {}",
                func_name, function.arity
            )));
        }

        // Execute sequentially for now (TODO: true parallelism for suitable functions)
        // This is a fallback - LLM functions are optimized via ParallelLlmMap
        let mut results = Vec::with_capacity(items.len());

        for item in items.iter() {
            // Push the argument
            self.push(item.clone())?;

            // Set up call frame
            let base_pointer = self.stack.len() - 1;
            let frame = CallFrame::new(function.clone(), base_pointer);
            let initial_frame_count = self.frames.len();
            self.push_frame(frame)?;

            // Execute until this function returns
            // Run the inner execute loop until we're back to the original frame depth
            while self.frames.len() > initial_frame_count {
                let frame_idx = self.frames.len() - 1;
                let frame = &mut self.frames[frame_idx];
                let ip = frame.ip;
                frame.ip += 1;

                if ip >= frame.function.chunk.code.len() {
                    // Implicit return null
                    let result = self.pop().unwrap_or(Value::Null);
                    let _ = self.pop_frame();
                    if self.frames.len() > initial_frame_count {
                        // Still in nested call, push for caller
                        self.push(result)?;
                    } else {
                        // Returned to original depth, capture result
                        results.push(result);
                    }
                    continue;
                }

                let op = frame.function.chunk.code[ip];
                match op {
                    OpCode::Return => {
                        let result = self.pop().unwrap_or(Value::Null);
                        // Clean up locals
                        let base = self.frames[frame_idx].base_pointer;
                        self.stack.truncate(base);
                        let _ = self.pop_frame();

                        if self.frames.len() > initial_frame_count {
                            // Still in nested call, push result for caller
                            self.push(result)?;
                        } else {
                            // Returned to original depth, capture result
                            results.push(result);
                        }
                    }
                    // Handle other opcodes - delegate to existing handlers
                    _ => {
                        self.execute_single_op(op, frame_idx)?;
                    }
                }
            }
        }

        // Push results list
        self.push(Value::List(results.into()))?;
        Ok(())
    }

    /// Execute a single opcode (helper for parallel_map's inner loop)
    fn execute_single_op(&mut self, op: OpCode, frame_idx: usize) -> Result<()> {
        match op {
            OpCode::Const(idx) => {
                let value = self.frames[frame_idx]
                    .function
                    .chunk
                    .constants
                    .get(idx)
                    .cloned()
                    .ok_or_else(|| {
                        LatticeError::Runtime(format!("Invalid constant index: {}", idx))
                    })?;
                self.push(value)?;
            }
            OpCode::Pop => {
                self.pop()?;
            }
            OpCode::PopBelow(n) => self.op_pop_below(n)?,
            OpCode::Dup => self.op_dup()?,
            OpCode::GetLocal(slot) => {
                let base = self.frames[frame_idx].base_pointer;
                let value = self.stack.get(base + slot).cloned().ok_or_else(|| {
                    LatticeError::Runtime(format!("Invalid local slot: {}", slot))
                })?;
                self.push(value)?;
            }
            OpCode::SetLocal(slot) => {
                let base = self.frames[frame_idx].base_pointer;
                let value = self.peek()?.clone();
                if base + slot < self.stack.len() {
                    self.stack[base + slot] = value;
                } else {
                    return Err(LatticeError::Runtime(format!(
                        "Invalid local slot: {}",
                        slot
                    )));
                }
            }
            OpCode::GetGlobal(name_idx) => {
                let name = self.frames[frame_idx]
                    .function
                    .chunk
                    .get_string(name_idx)
                    .ok_or_else(|| {
                        LatticeError::Runtime(format!("Invalid string index: {}", name_idx))
                    })?;
                let value = self.globals.get(name).cloned().ok_or_else(|| {
                    LatticeError::Runtime(format!("Undefined variable: {}", name))
                })?;
                self.push(value)?;
            }
            OpCode::SetGlobal(name_idx) => {
                let name = self.frames[frame_idx]
                    .function
                    .chunk
                    .get_string(name_idx)
                    .ok_or_else(|| {
                        LatticeError::Runtime(format!("Invalid string index: {}", name_idx))
                    })?
                    .to_string();
                let value = self.peek()?.clone();
                self.globals.insert(name, value);
            }
            OpCode::CallNative(name_idx, arg_count) => {
                let name = self.frames[frame_idx]
                    .function
                    .chunk
                    .get_string(name_idx)
                    .ok_or_else(|| {
                        LatticeError::Runtime(format!("Invalid string index: {}", name_idx))
                    })?
                    .to_string();
                self.op_call_native(&name, arg_count, 0)?;
            }
            OpCode::GetField(field_idx) => {
                let field_name = self.frames[frame_idx]
                    .function
                    .chunk
                    .get_string(field_idx)
                    .ok_or_else(|| {
                        LatticeError::Runtime(format!("Invalid string index: {}", field_idx))
                    })?
                    .to_string();
                self.op_get_field(&field_name)?;
            }
            OpCode::IndexSet => self.op_index_set()?,
            OpCode::MakeStruct(type_name_idx, field_count) => {
                let type_name = self.frames[frame_idx]
                    .function
                    .chunk
                    .get_string(type_name_idx)
                    .ok_or_else(|| {
                        LatticeError::Runtime(format!("Invalid string index: {}", type_name_idx))
                    })?
                    .to_string();
                self.op_make_struct(type_name, field_count)?;
            }
            OpCode::Add => self.op_add()?,
            OpCode::Sub => self.op_sub()?,
            OpCode::Mul => self.op_mul()?,
            OpCode::Div => self.op_div()?,
            OpCode::Mod => self.op_mod()?,
            OpCode::Neg => self.op_neg()?,
            OpCode::Eq => self.op_eq()?,
            OpCode::Ne => self.op_ne()?,
            OpCode::Lt => self.op_lt()?,
            OpCode::Le => self.op_le()?,
            OpCode::Gt => self.op_gt()?,
            OpCode::Ge => self.op_ge()?,
            OpCode::Not => self.op_not()?,
            OpCode::And => self.op_and()?,
            OpCode::Or => self.op_or()?,
            OpCode::Jump(target) => {
                self.frames[frame_idx].ip = target;
            }
            OpCode::JumpIfFalse(target) => {
                let cond = self.pop()?;
                if !is_truthy(&cond) {
                    self.frames[frame_idx].ip = target;
                }
            }
            OpCode::JumpIfTrue(target) => {
                let cond = self.pop()?;
                if is_truthy(&cond) {
                    self.frames[frame_idx].ip = target;
                }
            }
            OpCode::MakeList(count) => self.op_make_list(count)?,
            OpCode::MakeMap(count) => self.op_make_map(count)?,
            OpCode::Index => self.op_index()?,
            OpCode::Print => self.op_print()?,
            OpCode::Stringify => self.op_stringify()?,
            OpCode::Nop => {}
            OpCode::CallUser(name_idx, arg_count) => {
                // Get function name
                let func_name = self.frames[frame_idx]
                    .function
                    .chunk
                    .get_string(name_idx)
                    .ok_or_else(|| {
                        LatticeError::Runtime(format!("Invalid string index: {}", name_idx))
                    })?
                    .to_string();

                // Look up the function
                let function = self.user_functions.get(&func_name).ok_or_else(|| {
                    LatticeError::Runtime(format!("Undefined function: {}", func_name))
                })?.clone();

                // Verify arity
                if function.arity != arg_count {
                    return Err(LatticeError::Runtime(format!(
                        "Function '{}' expects {} arguments, got {}",
                        func_name, function.arity, arg_count
                    )));
                }

                // Set up call frame - args are already on the stack
                let base_pointer = self.stack.len() - arg_count;
                let frame = CallFrame::new(function, base_pointer);
                self.push_frame(frame)?;
            }
            OpCode::Call(arg_count) => {
                // Dynamic call - function reference is on top of stack
                let func_ref = self.pop()?;
                match func_ref {
                    Value::String(func_name) => {
                        let function = self.user_functions.get(func_name.as_ref()).ok_or_else(|| {
                            LatticeError::Runtime(format!("Undefined function: {}", func_name))
                        })?.clone();

                        let base_pointer = self.stack.len() - arg_count;
                        let frame = CallFrame::new(function, base_pointer);
                        self.push_frame(frame)?;
                    }
                    _ => {
                        return Err(LatticeError::Runtime(
                            "Cannot call non-function value".to_string()
                        ));
                    }
                }
            }
            OpCode::Return => {
                // This should be handled by the outer loop, but just in case
                return Err(LatticeError::Runtime(
                    "Unexpected Return in execute_single_op".to_string()
                ));
            }
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "Opcode {:?} not supported in inner loop",
                    op
                )));
            }
        }
        Ok(())
    }

    /// Parallel map with LLM function (specialized optimization)
    ///
    /// Pops collection from stack, calls LLM function on each item in parallel,
    /// pushes List of results onto stack.
    ///
    /// This is a compiler optimization for the pattern: parallel_map(items, |x| llm_fn(x))
    fn op_parallel_llm_map(&mut self, func_name: &str) -> Result<()> {
        use crate::llm::{extract_template_variables, generate_prompt_from_ir, parse_llm_response_with_ir, LLMClient};
        use std::sync::Arc;

        // Pop the collection
        let collection = self.pop()?;
        let items = match collection {
            Value::List(items) => items,
            _ => {
                return Err(LatticeError::Runtime(
                    "parallel_map requires a List".to_string(),
                ))
            }
        };

        // Empty list case
        if items.is_empty() {
            self.push(Value::List(Arc::new(vec![])))?;
            return Ok(());
        }

        // Get function info (clone to avoid borrow issues)
        let func = self.get_llm_function_by_name(func_name).ok_or_else(|| {
            LatticeError::Runtime(format!("Undefined LLM function: {}", func_name))
        })?.clone();

        // Verify function has exactly 1 parameter (since we're mapping |x| f(x))
        if func.arity() != 1 {
            return Err(LatticeError::Runtime(format!(
                "parallel_map with LLM function requires function with 1 parameter, got {}",
                func.arity()
            )));
        }

        // Get the API key
        let api_key = std::env::var(&func.api_key_env).map_err(|_| {
            LatticeError::Runtime(format!(
                "Environment variable '{}' not set. Please set it to your API key.",
                func.api_key_env
            ))
        })?;

        // Create the LLM client
        let mut client = LLMClient::custom(api_key, func.base_url.clone(), func.model.clone());
        if let Some(temp) = func.temperature {
            client = client.with_temperature(temp as f32);
        }
        if let Some(max_tok) = func.max_tokens {
            client = client.with_max_tokens(max_tok as u32);
        }
        if let Some(ref provider) = func.provider {
            client = client.with_provider(provider.clone());
        }

        // Extract variable names referenced in the template
        let referenced_vars = extract_template_variables(&func.prompt_template);

        // Clone referenced globals once (they're shared across all calls)
        let mut base_params: HashMap<String, Value> = HashMap::with_capacity(referenced_vars.len());
        for var_name in &referenced_vars {
            if let Some(value) = self.globals.get(var_name) {
                base_params.insert(var_name.clone(), value.clone());
            }
        }

        // Clone IR for use in async context
        let ir = self.ir.clone();
        let param_name = func.input_names()[0].clone();

        // Generate all prompts (synchronous, fast)
        let mut prompts = Vec::with_capacity(items.len());
        for item in items.iter() {
            let mut params = base_params.clone();
            params.insert(param_name.clone(), item.clone());

            let prompt = generate_prompt_from_ir(
                &ir,
                &func.prompt_template,
                &params,
                &func.return_type,
            ).map_err(|e| LatticeError::Runtime(format!("Failed to render prompt: {}", e)))?;

            prompts.push(prompt);
        }

        // Execute all LLM calls in parallel using shared runtime and HTTP client
        let http_client = self.http_client.clone();
        let raw_responses: Vec<std::result::Result<String, anyhow::Error>> =
            if let Some(limit) = self.max_concurrent_llm_calls {
                // Use semaphore to limit concurrency
                let semaphore = Arc::new(Semaphore::new(limit));
                self.runtime.block_on(async {
                    let futures: Vec<_> = prompts.iter()
                        .map(|prompt| {
                            let sem = semaphore.clone();
                            let client = client.clone();
                            let http_client = http_client.clone();
                            let prompt = prompt.clone();
                            async move {
                                let _permit = sem.acquire().await.unwrap();
                                client.call_with_client(&prompt, &http_client).await
                            }
                        })
                        .collect();
                    futures::future::join_all(futures).await
                })
            } else {
                // Unlimited concurrency
                self.runtime.block_on(async {
                    let futures: Vec<_> = prompts.iter()
                        .map(|prompt| {
                            let client = client.clone();
                            let http_client = http_client.clone();
                            let prompt = prompt.clone();
                            async move {
                                client.call_with_client(&prompt, &http_client).await
                            }
                        })
                        .collect();
                    futures::future::join_all(futures).await
                })
            };

        // Parse all responses
        let mut results = Vec::with_capacity(raw_responses.len());
        for (i, response_result) in raw_responses.into_iter().enumerate() {
            let raw_response = response_result.map_err(|e| {
                LatticeError::Runtime(format!("LLM call {} failed: {}", i, e))
            })?;

            let result = parse_llm_response_with_ir(&ir, &raw_response, &func.return_type)
                .map_err(|e| LatticeError::Runtime(format!(
                    "Failed to parse LLM response {}: {}. Raw response was: {}",
                    i, e, raw_response
                )))?;

            results.push(result);
        }

        // Push the results list onto the stack
        self.push(Value::List(Arc::new(results)))
    }

    /// Map a column: apply a function to each row's input column value,
    /// adding the result as a new output column
    ///
    /// Stack order (top to bottom): mapper, output_col, input_col, table
    fn op_map_column(&mut self) -> Result<()> {
        use std::sync::Arc;
        let func_name_value = self.pop()?;
        let output_col = self.pop()?;
        let input_col = self.pop()?;
        let table = self.pop()?;

        // Get column names
        let input_col_name = match &input_col {
            Value::String(s) => s.to_string(),
            _ => return Err(LatticeError::Runtime(
                "map_column input column must be a string".to_string()
            )),
        };
        let output_col_name = match &output_col {
            Value::String(s) => s.to_string(),
            _ => return Err(LatticeError::Runtime(
                "map_column output column must be a string".to_string()
            )),
        };

        // Get function name
        let func_name = match &func_name_value {
            Value::String(s) => s.to_string(),
            _ => return Err(LatticeError::Runtime(
                "map_column mapper must be a function name".to_string()
            )),
        };

        // Verify table is a List
        let rows = match &table {
            Value::List(list) => list.clone(),
            _ => return Err(LatticeError::Runtime(
                "map_column requires a List (table)".to_string()
            )),
        };

        // Empty table case
        if rows.is_empty() {
            self.push(Value::List(Arc::new(vec![])))?;
            return Ok(());
        }

        // Look up the function
        let function = self.user_functions.get(&func_name).ok_or_else(|| {
            LatticeError::Runtime(format!("Undefined function: {}", func_name))
        })?.clone();

        // Verify arity (should be 1 for column mapping)
        if function.arity != 1 {
            return Err(LatticeError::Runtime(format!(
                "map_column function '{}' must take exactly 1 argument, takes {}",
                func_name, function.arity
            )));
        }

        // Process each row
        let mut result_rows = Vec::with_capacity(rows.len());
        for row in rows.iter() {
            // Verify row is a Map
            let row_map = match row {
                Value::Map(m) => m.clone(),
                _ => return Err(LatticeError::Runtime(
                    "map_column rows must be Maps".to_string()
                )),
            };

            // Get the input value from the row
            let input_value = row_map.get(&input_col_name).cloned().unwrap_or(Value::Null);

            // Push the argument
            self.push(input_value)?;

            // Set up call frame
            let base_pointer = self.stack.len() - 1;
            let frame = CallFrame::new(function.clone(), base_pointer);
            let initial_frame_count = self.frames.len();
            self.push_frame(frame)?;

            // Execute until this function returns
            let mut result = Value::Null;
            while self.frames.len() > initial_frame_count {
                let frame_idx = self.frames.len() - 1;
                let frame = &mut self.frames[frame_idx];
                let ip = frame.ip;
                frame.ip += 1;

                if ip >= frame.function.chunk.code.len() {
                    // Implicit return null
                    let ret_val = self.pop().unwrap_or(Value::Null);
                    let base = self.frames[frame_idx].base_pointer;
                    self.stack.truncate(base);
                    let _ = self.pop_frame();

                    if self.frames.len() > initial_frame_count {
                        // Still in nested call, push for caller
                        self.push(ret_val)?;
                    } else {
                        // Returned to original depth, capture result
                        result = ret_val;
                    }
                    continue;
                }

                let op = frame.function.chunk.code[ip];
                match op {
                    OpCode::Return => {
                        let ret_val = self.pop().unwrap_or(Value::Null);
                        // Clean up locals
                        let base = self.frames[frame_idx].base_pointer;
                        self.stack.truncate(base);
                        let _ = self.pop_frame();

                        if self.frames.len() > initial_frame_count {
                            // Still in nested call, push result for caller
                            self.push(ret_val)?;
                        } else {
                            // Returned to original depth, capture result
                            result = ret_val;
                        }
                    }
                    // Handle other opcodes - delegate to existing handlers
                    _ => {
                        self.execute_single_op(op, frame_idx)?;
                    }
                }
            }

            // Create new row with output column added
            let mut new_row = (*row_map).clone();
            new_row.insert(output_col_name.clone(), result);
            result_rows.push(Value::Map(Arc::new(new_row)));
        }

        self.push(Value::List(Arc::new(result_rows)))
    }

    /// Map column with LLM function (parallel execution)
    ///
    /// Stack order (top to bottom): output_col, input_col, table
    fn op_map_column_llm(&mut self, func_name: &str) -> Result<()> {
        use crate::llm::{extract_template_variables, generate_prompt_from_ir, generate_schema_from_ir, parse_llm_response_with_ir, LLMClient};
        use std::sync::Arc;

        let output_col = self.pop()?;
        let input_col = self.pop()?;
        let table = self.pop()?;

        // Get column names
        let input_col_name = match &input_col {
            Value::String(s) => s.to_string(),
            _ => return Err(LatticeError::Runtime(
                "map_column input column must be a string".to_string()
            )),
        };
        let output_col_name = match &output_col {
            Value::String(s) => s.to_string(),
            _ => return Err(LatticeError::Runtime(
                "map_column output column must be a string".to_string()
            )),
        };

        // Verify table is a List
        let rows = match &table {
            Value::List(list) => list.clone(),
            _ => return Err(LatticeError::Runtime(
                "map_column requires a List (table)".to_string()
            )),
        };

        // Empty table case
        if rows.is_empty() {
            self.push(Value::List(Arc::new(vec![])))?;
            return Ok(());
        }

        // Get function info
        let func = self.get_llm_function_by_name(func_name).ok_or_else(|| {
            LatticeError::Runtime(format!("Undefined LLM function: {}", func_name))
        })?.clone();

        // Verify function has exactly 1 parameter
        if func.arity() != 1 {
            return Err(LatticeError::Runtime(format!(
                "map_column with LLM function requires function with 1 parameter, got {}",
                func.arity()
            )));
        }

        // Get the API key
        let api_key = std::env::var(&func.api_key_env).map_err(|_| {
            LatticeError::Runtime(format!(
                "Environment variable '{}' not set. Please set it to your API key.",
                func.api_key_env
            ))
        })?;

        // Create the LLM client
        let mut client = LLMClient::custom(api_key, func.base_url.clone(), func.model.clone());
        if let Some(temp) = func.temperature {
            client = client.with_temperature(temp as f32);
        }
        if let Some(max_tok) = func.max_tokens {
            client = client.with_max_tokens(max_tok as u32);
        }
        if let Some(ref provider) = func.provider {
            client = client.with_provider(provider.clone());
        }

        // Clone IR for use in async context
        let ir = self.ir.clone();

        // Extract input values and generate prompts
        let mut prompts = Vec::with_capacity(rows.len());
        let mut row_maps: Vec<Arc<HashMap<String, Value>>> = Vec::with_capacity(rows.len());

        // Check if we need to use bytecode execution for prompts
        if let Some(ref prompt_chunk) = func.prompt_chunk {
            // Use bytecode execution for complex prompt expressions
            let schema = generate_schema_from_ir(&ir, &func.return_type);

            for row in rows.iter() {
                let row_map = match row {
                    Value::Map(m) => m.clone(),
                    _ => return Err(LatticeError::Runtime(
                        "map_column rows must be Maps".to_string()
                    )),
                };

                let input_value = row_map.get(&input_col_name).cloned().unwrap_or(Value::Null);
                row_maps.push(row_map);

                // Execute the prompt chunk with the input value
                let prompt_str = self.execute_prompt_chunk(prompt_chunk, vec![input_value])?;

                // Append the schema
                let prompt = format!("{}\n\n{}", prompt_str, schema);
                prompts.push(prompt);
            }
        } else {
            // Use the template-based approach for simple prompts
            // Extract variable names referenced in the template
            let referenced_vars = extract_template_variables(&func.prompt_template);

            // Clone referenced globals
            let mut base_params: HashMap<String, Value> = HashMap::with_capacity(referenced_vars.len());
            for var_name in &referenced_vars {
                if let Some(value) = self.globals.get(var_name) {
                    base_params.insert(var_name.clone(), value.clone());
                }
            }

            let param_name = func.input_names()[0].clone();

            for row in rows.iter() {
                let row_map = match row {
                    Value::Map(m) => m.clone(),
                    _ => return Err(LatticeError::Runtime(
                        "map_column rows must be Maps".to_string()
                    )),
                };

                let input_value = row_map.get(&input_col_name).cloned().unwrap_or(Value::Null);
                row_maps.push(row_map);

                let mut params = base_params.clone();
                params.insert(param_name.clone(), input_value);

                let prompt = generate_prompt_from_ir(
                    &ir,
                    &func.prompt_template,
                    &params,
                    &func.return_type,
                ).map_err(|e| LatticeError::Runtime(format!("Failed to render prompt: {}", e)))?;

                prompts.push(prompt);
            }
        }

        // Execute all LLM calls in parallel using shared runtime and HTTP client
        let http_client = self.http_client.clone();
        let raw_responses: Vec<std::result::Result<String, anyhow::Error>> =
            if let Some(limit) = self.max_concurrent_llm_calls {
                // Use semaphore to limit concurrency
                let semaphore = Arc::new(Semaphore::new(limit));
                self.runtime.block_on(async {
                    let futures: Vec<_> = prompts.iter()
                        .map(|prompt| {
                            let sem = semaphore.clone();
                            let client = client.clone();
                            let http_client = http_client.clone();
                            let prompt = prompt.clone();
                            async move {
                                let _permit = sem.acquire().await.unwrap();
                                client.call_with_client(&prompt, &http_client).await
                            }
                        })
                        .collect();
                    futures::future::join_all(futures).await
                })
            } else {
                // Unlimited concurrency
                self.runtime.block_on(async {
                    let futures: Vec<_> = prompts.iter()
                        .map(|prompt| {
                            let client = client.clone();
                            let http_client = http_client.clone();
                            let prompt = prompt.clone();
                            async move {
                                client.call_with_client(&prompt, &http_client).await
                            }
                        })
                        .collect();
                    futures::future::join_all(futures).await
                })
            };

        // Parse responses and create new rows with output column
        let mut result_rows = Vec::with_capacity(raw_responses.len());
        for (i, (response_result, row_map)) in raw_responses.into_iter().zip(row_maps.into_iter()).enumerate() {
            let raw_response = response_result.map_err(|e| {
                LatticeError::Runtime(format!("LLM call {} failed: {}", i, e))
            })?;

            let result = parse_llm_response_with_ir(&ir, &raw_response, &func.return_type)
                .map_err(|e| LatticeError::Runtime(format!(
                    "Failed to parse LLM response {}: {}. Raw response was: {}",
                    i, e, raw_response
                )))?;

            // Create new row with output column added
            let mut new_row = (*row_map).clone();
            new_row.insert(output_col_name.clone(), result);
            result_rows.push(Value::Map(Arc::new(new_row)));
        }

        self.push(Value::List(Arc::new(result_rows)))
    }

    /// Map row: apply a function to each entire row, adding the result as a new column
    ///
    /// Stack order (top to bottom): mapper, output_col, table
    fn op_map_row(&mut self) -> Result<()> {
        use std::sync::Arc;
        let func_name_value = self.pop()?;
        let output_col = self.pop()?;
        let table = self.pop()?;

        // Get output column name
        let output_col_name = match &output_col {
            Value::String(s) => s.to_string(),
            _ => return Err(LatticeError::Runtime(
                "map_row output column must be a string".to_string()
            )),
        };

        // Get function name
        let func_name = match &func_name_value {
            Value::String(s) => s.to_string(),
            _ => return Err(LatticeError::Runtime(
                "map_row mapper must be a function name".to_string()
            )),
        };

        // Verify table is a List
        let rows = match &table {
            Value::List(list) => list.clone(),
            _ => return Err(LatticeError::Runtime(
                "map_row requires a List (table)".to_string()
            )),
        };

        // Empty table case
        if rows.is_empty() {
            self.push(Value::List(Arc::new(vec![])))?;
            return Ok(());
        }

        // Look up the function
        let function = self.user_functions.get(&func_name).ok_or_else(|| {
            LatticeError::Runtime(format!("Undefined function: {}", func_name))
        })?.clone();

        // Verify arity (should be 1 for row mapping - receives entire row)
        if function.arity != 1 {
            return Err(LatticeError::Runtime(format!(
                "map_row function '{}' must take exactly 1 argument (the row), takes {}",
                func_name, function.arity
            )));
        }

        // Process each row
        let mut result_rows = Vec::with_capacity(rows.len());
        for row in rows.iter() {
            // Verify row is a Map
            let row_map = match row {
                Value::Map(m) => m.clone(),
                _ => return Err(LatticeError::Runtime(
                    "map_row rows must be Maps".to_string()
                )),
            };

            // Push the entire row as the argument
            self.push(Value::Map(row_map.clone()))?;

            // Set up call frame
            let base_pointer = self.stack.len() - 1;
            let frame = CallFrame::new(function.clone(), base_pointer);
            let initial_frame_count = self.frames.len();
            self.push_frame(frame)?;

            // Execute until this function returns
            let mut result = Value::Null;
            while self.frames.len() > initial_frame_count {
                let frame_idx = self.frames.len() - 1;
                let frame = &mut self.frames[frame_idx];
                let ip = frame.ip;
                frame.ip += 1;

                if ip >= frame.function.chunk.code.len() {
                    // Implicit return null
                    let ret_val = self.pop().unwrap_or(Value::Null);
                    let base = self.frames[frame_idx].base_pointer;
                    self.stack.truncate(base);
                    let _ = self.pop_frame();

                    if self.frames.len() > initial_frame_count {
                        self.push(ret_val)?;
                    } else {
                        result = ret_val;
                    }
                    continue;
                }

                let op = frame.function.chunk.code[ip];
                match op {
                    OpCode::Return => {
                        let ret_val = self.pop().unwrap_or(Value::Null);
                        let base = self.frames[frame_idx].base_pointer;
                        self.stack.truncate(base);
                        let _ = self.pop_frame();

                        if self.frames.len() > initial_frame_count {
                            self.push(ret_val)?;
                        } else {
                            result = ret_val;
                        }
                    }
                    _ => {
                        self.execute_single_op(op, frame_idx)?;
                    }
                }
            }

            // Create new row with output column added
            let mut new_row = (*row_map).clone();
            new_row.insert(output_col_name.clone(), result);
            result_rows.push(Value::Map(Arc::new(new_row)));
        }

        self.push(Value::List(Arc::new(result_rows)))
    }

    /// Map row with LLM function (parallel execution)
    ///
    /// Stack order (top to bottom): mapper (lambda), output_col, table
    /// The lambda body should call an LLM function with row fields
    ///
    /// # Arguments
    /// * `func_name` - The name of the LLM function to call
    /// * `column_mappings` - Comma-separated list of column names from the row to use as arguments.
    ///   The order matches the function's parameter order. E.g., "job_description,verbal_reasoning_score"
    ///   means row["job_description"] is passed as first arg, row["verbal_reasoning_score"] as second.
    fn op_map_row_llm(&mut self, func_name: &str, column_mappings: &str) -> Result<()> {
        use crate::llm::{extract_template_variables, generate_prompt_from_ir, generate_schema_from_ir, parse_llm_response_with_ir, LLMClient};
        use std::sync::Arc;

        let _mapper = self.pop()?; // Lambda (used for detection, not execution)
        let output_col = self.pop()?;
        let table = self.pop()?;

        // Get output column name
        let output_col_name = match &output_col {
            Value::String(s) => s.to_string(),
            _ => return Err(LatticeError::Runtime(
                "map_row output column must be a string".to_string()
            )),
        };

        // Verify table is a List
        let rows = match &table {
            Value::List(list) => list.clone(),
            _ => return Err(LatticeError::Runtime(
                "map_row requires a List (table)".to_string()
            )),
        };

        // Empty table case
        if rows.is_empty() {
            self.push(Value::List(Arc::new(vec![])))?;
            return Ok(());
        }

        // Get function info
        let func = self.get_llm_function_by_name(func_name).ok_or_else(|| {
            LatticeError::Runtime(format!("Undefined LLM function: {}", func_name))
        })?.clone();

        // Get the API key
        let api_key = std::env::var(&func.api_key_env).map_err(|_| {
            LatticeError::Runtime(format!(
                "Environment variable '{}' not set. Please set it to your API key.",
                func.api_key_env
            ))
        })?;

        // Create the LLM client
        let mut client = LLMClient::custom(api_key, func.base_url.clone(), func.model.clone());
        if let Some(temp) = func.temperature {
            client = client.with_temperature(temp as f32);
        }
        if let Some(max_tok) = func.max_tokens {
            client = client.with_max_tokens(max_tok as u32);
        }
        if let Some(ref provider) = func.provider {
            client = client.with_provider(provider.clone());
        }

        // Parse the column mappings - these are the actual row keys to use
        // The column_mappings string is comma-separated: "col1,col2,col3"
        let row_column_names: Vec<&str> = column_mappings.split(',').collect();

        // Get the function's parameter names - these are what the template uses
        let param_names = func.input_names();

        // Verify column mappings match parameter count
        if row_column_names.len() != param_names.len() {
            return Err(LatticeError::Runtime(format!(
                "map_row: column mappings count ({}) doesn't match function '{}' parameter count ({})",
                row_column_names.len(), func_name, param_names.len()
            )));
        }

        // Verify all param names can be found in rows
        let row_maps: Vec<Arc<HashMap<String, Value>>> = rows.iter().map(|row| {
            match row {
                Value::Map(m) => Ok(m.clone()),
                _ => Err(LatticeError::Runtime("map_row rows must be Maps".to_string())),
            }
        }).collect::<Result<Vec<_>>>()?;

        // Generate prompts for each row
        let mut prompts = Vec::with_capacity(rows.len());

        // Check if we need to use bytecode execution for prompts
        if let Some(ref prompt_chunk) = func.prompt_chunk {
            // Use bytecode execution for complex prompt expressions
            let schema = generate_schema_from_ir(&self.ir, &func.return_type);

            for row_map in &row_maps {
                // Build args vector from row columns in order
                let mut args = Vec::with_capacity(row_column_names.len());
                for &row_col in &row_column_names {
                    let value = if row_col.is_empty() {
                        Value::Null
                    } else {
                        row_map.get(row_col).cloned().unwrap_or(Value::Null)
                    };
                    args.push(value);
                }

                // Execute the prompt chunk to generate the prompt string
                let prompt_str = self.execute_prompt_chunk(prompt_chunk, args)?;

                // Append the schema
                let prompt = format!("{}\n\n{}", prompt_str, schema);
                prompts.push(prompt);
            }
        } else {
            // Use the template-based approach for simple prompts
            // Extract variable names referenced in the template
            let referenced_vars = extract_template_variables(&func.prompt_template);

            // Clone referenced globals
            let mut base_params: HashMap<String, Value> = HashMap::with_capacity(referenced_vars.len());
            for var_name in &referenced_vars {
                if let Some(value) = self.globals.get(var_name) {
                    base_params.insert(var_name.clone(), value.clone());
                }
            }

            for row_map in &row_maps {
                let mut params = base_params.clone();
                // Map function parameters to row fields using the column mappings
                // param_names[i] = what the template uses (e.g., "score_eval")
                // row_column_names[i] = what the row has (e.g., "verbal_reasoning_score_unweighted_eval")
                for (param_name, &row_col) in param_names.iter().zip(row_column_names.iter()) {
                    let value = if row_col.is_empty() {
                        Value::Null
                    } else {
                        row_map.get(row_col).cloned().unwrap_or(Value::Null)
                    };
                    params.insert(param_name.clone(), value);
                }

                let prompt = generate_prompt_from_ir(
                    &self.ir,
                    &func.prompt_template,
                    &params,
                    &func.return_type,
                ).map_err(|e| LatticeError::Runtime(format!("Failed to render prompt: {}", e)))?;

                prompts.push(prompt);
            }
        }

        // Execute all LLM calls in parallel using shared runtime and HTTP client
        let http_client = self.http_client.clone();
        let raw_responses: Vec<std::result::Result<String, anyhow::Error>> =
            if let Some(limit) = self.max_concurrent_llm_calls {
                // Use semaphore to limit concurrency
                let semaphore = Arc::new(Semaphore::new(limit));
                self.runtime.block_on(async {
                    let futures: Vec<_> = prompts.iter()
                        .map(|prompt| {
                            let sem = semaphore.clone();
                            let client = client.clone();
                            let http_client = http_client.clone();
                            let prompt = prompt.clone();
                            async move {
                                let _permit = sem.acquire().await.unwrap();
                                client.call_with_client(&prompt, &http_client).await
                            }
                        })
                        .collect();
                    futures::future::join_all(futures).await
                })
            } else {
                // Unlimited concurrency
                self.runtime.block_on(async {
                    let futures: Vec<_> = prompts.iter()
                        .map(|prompt| {
                            let client = client.clone();
                            let http_client = http_client.clone();
                            let prompt = prompt.clone();
                            async move {
                                client.call_with_client(&prompt, &http_client).await
                            }
                        })
                        .collect();
                    futures::future::join_all(futures).await
                })
            };

        // Parse responses and create new rows with output column
        let mut result_rows = Vec::with_capacity(raw_responses.len());
        for (i, (response_result, row_map)) in raw_responses.into_iter().zip(row_maps.into_iter()).enumerate() {
            let raw_response = response_result.map_err(|e| {
                LatticeError::Runtime(format!("LLM call {} failed: {}", i, e))
            })?;

            let result = parse_llm_response_with_ir(&self.ir, &raw_response, &func.return_type)
                .map_err(|e| LatticeError::Runtime(format!(
                    "Failed to parse LLM response {}: {}. Raw response was: {}",
                    i, e, raw_response
                )))?;

            // Create new row with output column added
            let mut new_row = (*row_map).clone();
            new_row.insert(output_col_name.clone(), result);
            result_rows.push(Value::Map(Arc::new(new_row)));
        }

        self.push(Value::List(Arc::new(result_rows)))
    }

    /// Explode nested map column into separate columns
    ///
    /// Takes a table (list of maps) and a column name containing nested maps,
    /// expands those nested map keys into new columns.
    ///
    /// Stack: [table, column_name, prefix] -> [result_table]
    fn op_explode(&mut self) -> Result<()> {
        // Pop arguments from stack
        let prefix_val = self.pop()?;
        let column_val = self.pop()?;
        let table_val = self.pop()?;

        // Extract column name
        let column_name = match &column_val {
            Value::String(s) => s.to_string(),
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "explode column name must be String, got {}",
                    type_name(&column_val)
                )))
            }
        };

        // Extract prefix (default to empty string)
        let prefix = match &prefix_val {
            Value::Null => String::new(),
            Value::String(s) => s.to_string(),
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "explode prefix must be String or null, got {}",
                    type_name(&prefix_val)
                )))
            }
        };

        // Get the table (list of maps)
        let rows = match &table_val {
            Value::List(list) => list.clone(),
            _ => {
                return Err(LatticeError::Runtime(format!(
                    "explode table must be List, got {}",
                    type_name(&table_val)
                )))
            }
        };

        // First pass: collect all keys from the nested maps
        let mut all_nested_keys: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for row in rows.iter() {
            if let Value::Map(row_map) = row {
                if let Some(Value::Map(nested_map)) = row_map.get(&column_name) {
                    for key in nested_map.keys() {
                        all_nested_keys.insert(key.clone());
                    }
                }
            }
        }

        // Sort keys for consistent column ordering
        let mut sorted_keys: Vec<_> = all_nested_keys.into_iter().collect();
        sorted_keys.sort();

        // Second pass: create new rows with exploded columns
        let mut result_rows = Vec::with_capacity(rows.len());

        for row in rows.iter() {
            let row_map = match row {
                Value::Map(m) => m,
                _ => {
                    return Err(LatticeError::Runtime(format!(
                        "explode row must be Map, got {}",
                        type_name(row)
                    )))
                }
            };

            // Start with all existing columns (keeping the original nested column too)
            let mut new_row: HashMap<String, Value> = (**row_map).clone();

            // Get the nested map (if it exists)
            let nested_map = row_map.get(&column_name).and_then(|v| {
                if let Value::Map(m) = v {
                    Some(m.clone())
                } else {
                    None
                }
            });

            // Add exploded columns
            for nested_key in &sorted_keys {
                let new_col_name = format!("{}{}", prefix, nested_key);
                let value = nested_map
                    .as_ref()
                    .and_then(|m| m.get(nested_key))
                    .cloned()
                    .unwrap_or(Value::Null);
                new_row.insert(new_col_name, value);
            }

            result_rows.push(Value::Map(Arc::new(new_row)));
        }

        self.push(Value::List(Arc::new(result_rows)))
    }

    /// Await an async/future value
    ///
    /// Pops async value from stack, blocks until resolved,
    /// pushes resolved value onto stack.
    fn op_await(&mut self) -> Result<()> {
        let _async_value = self.pop()?;

        // TODO: Implement await
        // In a sync context, this might need to block
        // In an async context, this would yield

        Err(LatticeError::Runtime(
            "Await not yet implemented".to_string(),
        ))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the type name of a value for error messages
fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::String(_) => "String",
        Value::Bool(_) => "Bool",
        Value::Path(_) => "Path",
        Value::List(_) => "List",
        Value::Map(_) => "Map",
        Value::Null => "Null",
    }
}

/// Check if a value is truthy
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Int(0) => false,
        Value::Float(f) if *f == 0.0 => false,
        Value::String(s) if s.is_empty() => false,
        Value::List(l) if l.is_empty() => false,
        Value::Map(m) if m.is_empty() => false,
        _ => true,
    }
}

/// Check if two values are equal
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
        (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Path(a), Value::Path(b)) => a == b,
        (Value::Null, Value::Null) => true,
        (Value::List(a), Value::List(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Map(a), Value::Map(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).map(|bv| values_equal(v, bv)).unwrap_or(false))
        }
        _ => false,
    }
}

/// Compare two values (for < <= > >=)
/// Returns negative if a < b, 0 if equal, positive if a > b
fn compare_values(a: &Value, b: &Value) -> Result<i32> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok(a.cmp(b) as i32),
        (Value::Float(a), Value::Float(b)) => {
            Ok(a.partial_cmp(b).map(|o| o as i32).unwrap_or(0))
        }
        (Value::Int(a), Value::Float(b)) => {
            let a = *a as f64;
            Ok(a.partial_cmp(b).map(|o| o as i32).unwrap_or(0))
        }
        (Value::Float(a), Value::Int(b)) => {
            let b = *b as f64;
            Ok(a.partial_cmp(&b).map(|o| o as i32).unwrap_or(0))
        }
        (Value::String(a), Value::String(b)) => Ok(a.cmp(b) as i32),
        (Value::Path(a), Value::Path(b)) => Ok(a.cmp(b) as i32),
        _ => Err(LatticeError::Runtime(format!(
            "Cannot compare {} and {}",
            type_name(a),
            type_name(b)
        ))),
    }
}

/// Call a native function by name
fn call_native_function(name: &str, args: Vec<Value>) -> Result<Value> {
    match name {
        "len" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "len() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::String(s) => Ok(Value::Int(s.len() as i64)),
                Value::List(l) => Ok(Value::Int(l.len() as i64)),
                Value::Map(m) => Ok(Value::Int(m.len() as i64)),
                _ => Err(LatticeError::Runtime(format!(
                    "len() not supported for {}",
                    type_name(&args[0])
                ))),
            }
        }
        "word_count" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "word_count() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::String(s) => {
                    let count = s.split_whitespace().count() as i64;
                    Ok(Value::Int(count))
                }
                _ => Err(LatticeError::Runtime(format!(
                    "word_count() requires a string, got {}",
                    type_name(&args[0])
                ))),
            }
        }
        "type" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "type() takes exactly 1 argument".to_string(),
                ));
            }
            Ok(Value::string(type_name(&args[0])))
        }
        "str" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "str() takes exactly 1 argument".to_string(),
                ));
            }
            Ok(Value::string(format!("{}", args[0])))
        }
        "int" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "int() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Int(i) => Ok(Value::Int(*i)),
                Value::Float(f) => Ok(Value::Int(*f as i64)),
                Value::String(s) => s.parse::<i64>().map(Value::Int).map_err(|_| {
                    LatticeError::Runtime(format!("Cannot convert '{}' to int", s))
                }),
                Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
                _ => Err(LatticeError::Runtime(format!(
                    "Cannot convert {} to int",
                    type_name(&args[0])
                ))),
            }
        }
        "float" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "float() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(*i as f64)),
                Value::Float(f) => Ok(Value::Float(*f)),
                Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
                    LatticeError::Runtime(format!("Cannot convert '{}' to float", s))
                }),
                _ => Err(LatticeError::Runtime(format!(
                    "Cannot convert {} to float",
                    type_name(&args[0])
                ))),
            }
        }
        "bool" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "bool() takes exactly 1 argument".to_string(),
                ));
            }
            Ok(Value::Bool(is_truthy(&args[0])))
        }
        "push" => {
            if args.len() != 2 {
                return Err(LatticeError::Runtime(
                    "push() takes exactly 2 arguments".to_string(),
                ));
            }
            match &args[0] {
                Value::List(l) => {
                    let mut list = (**l).clone();
                    list.push(args[1].clone());
                    Ok(Value::list(list))
                }
                _ => Err(LatticeError::Runtime(
                    "push() first argument must be a list".to_string(),
                )),
            }
        }
        "pop" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "pop() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::List(l) => {
                    if l.is_empty() {
                        Err(LatticeError::Runtime("Cannot pop from empty list".to_string()))
                    } else {
                        let mut list = (**l).clone();
                        let value = list.pop().unwrap();
                        Ok(value)
                    }
                }
                _ => Err(LatticeError::Runtime(
                    "pop() argument must be a list".to_string(),
                )),
            }
        }
        "keys" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "keys() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Map(m) => {
                    let keys: Vec<Value> = m.keys().map(|k| Value::string(k.clone())).collect();
                    Ok(Value::list(keys))
                }
                _ => Err(LatticeError::Runtime(
                    "keys() argument must be a map".to_string(),
                )),
            }
        }
        "values" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "values() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Map(m) => {
                    let values: Vec<Value> = m.values().cloned().collect();
                    Ok(Value::list(values))
                }
                _ => Err(LatticeError::Runtime(
                    "values() argument must be a map".to_string(),
                )),
            }
        }
        "print" => {
            if args.is_empty() {
                println!();
            } else {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    match arg {
                        Value::String(s) => print!("{}", s),
                        other => print!("{}", other),
                    }
                }
                println!();
            }
            Ok(Value::Null)
        }
        "sqrt" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "sqrt() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Int(i) => Ok(Value::Float((*i as f64).sqrt())),
                Value::Float(f) => Ok(Value::Float(f.sqrt())),
                _ => Err(LatticeError::Runtime(format!(
                    "sqrt() not supported for {}",
                    type_name(&args[0])
                ))),
            }
        }
        // Path functions
        "path" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "path() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::String(s) => Ok(Value::path(std::path::PathBuf::from(&**s))),
                Value::Path(p) => Ok(Value::Path(p.clone())),
                _ => Err(LatticeError::Runtime(format!(
                    "path() expects String, got {}",
                    type_name(&args[0])
                ))),
            }
        }
        "path_join" => {
            if args.len() != 2 {
                return Err(LatticeError::Runtime(
                    "path_join() takes exactly 2 arguments".to_string(),
                ));
            }
            let base = match &args[0] {
                Value::Path(p) => (**p).clone(),
                Value::String(s) => std::path::PathBuf::from(&**s),
                _ => return Err(LatticeError::Runtime(format!(
                    "path_join() first argument must be Path or String, got {}",
                    type_name(&args[0])
                ))),
            };
            let to_join = match &args[1] {
                Value::Path(p) => (**p).clone(),
                Value::String(s) => std::path::PathBuf::from(&**s),
                _ => return Err(LatticeError::Runtime(format!(
                    "path_join() second argument must be Path or String, got {}",
                    type_name(&args[1])
                ))),
            };
            Ok(Value::path(base.join(to_join)))
        }
        "path_parent" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "path_parent() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Path(p) => {
                    match p.parent() {
                        Some(parent) => Ok(Value::path(parent.to_path_buf())),
                        None => Ok(Value::Null),
                    }
                }
                Value::String(s) => {
                    let p = std::path::Path::new(&**s);
                    match p.parent() {
                        Some(parent) => Ok(Value::path(parent.to_path_buf())),
                        None => Ok(Value::Null),
                    }
                }
                _ => Err(LatticeError::Runtime(format!(
                    "path_parent() expects Path or String, got {}",
                    type_name(&args[0])
                ))),
            }
        }
        "path_file_name" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "path_file_name() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Path(p) => {
                    match p.file_name() {
                        Some(name) => Ok(Value::string(name.to_string_lossy().to_string())),
                        None => Ok(Value::Null),
                    }
                }
                Value::String(s) => {
                    let p = std::path::Path::new(&**s);
                    match p.file_name() {
                        Some(name) => Ok(Value::string(name.to_string_lossy().to_string())),
                        None => Ok(Value::Null),
                    }
                }
                _ => Err(LatticeError::Runtime(format!(
                    "path_file_name() expects Path or String, got {}",
                    type_name(&args[0])
                ))),
            }
        }
        "path_extension" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "path_extension() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Path(p) => {
                    match p.extension() {
                        Some(ext) => Ok(Value::string(ext.to_string_lossy().to_string())),
                        None => Ok(Value::Null),
                    }
                }
                Value::String(s) => {
                    let p = std::path::Path::new(&**s);
                    match p.extension() {
                        Some(ext) => Ok(Value::string(ext.to_string_lossy().to_string())),
                        None => Ok(Value::Null),
                    }
                }
                _ => Err(LatticeError::Runtime(format!(
                    "path_extension() expects Path or String, got {}",
                    type_name(&args[0])
                ))),
            }
        }
        "path_exists" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "path_exists() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Path(p) => Ok(Value::Bool(p.exists())),
                Value::String(s) => Ok(Value::Bool(std::path::Path::new(&**s).exists())),
                _ => Err(LatticeError::Runtime(format!(
                    "path_exists() expects Path or String, got {}",
                    type_name(&args[0])
                ))),
            }
        }
        "path_is_file" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "path_is_file() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Path(p) => Ok(Value::Bool(p.is_file())),
                Value::String(s) => Ok(Value::Bool(std::path::Path::new(&**s).is_file())),
                _ => Err(LatticeError::Runtime(format!(
                    "path_is_file() expects Path or String, got {}",
                    type_name(&args[0])
                ))),
            }
        }
        "path_is_dir" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "path_is_dir() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Path(p) => Ok(Value::Bool(p.is_dir())),
                Value::String(s) => Ok(Value::Bool(std::path::Path::new(&**s).is_dir())),
                _ => Err(LatticeError::Runtime(format!(
                    "path_is_dir() expects Path or String, got {}",
                    type_name(&args[0])
                ))),
            }
        }
        "path_to_str" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "path_to_str() takes exactly 1 argument".to_string(),
                ));
            }
            match &args[0] {
                Value::Path(p) => Ok(Value::string(p.display().to_string())),
                Value::String(s) => Ok(Value::String(s.clone())),
                _ => Err(LatticeError::Runtime(format!(
                    "path_to_str() expects Path or String, got {}",
                    type_name(&args[0])
                ))),
            }
        }
        // String functions
        "contains" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(LatticeError::Runtime(
                    "contains() takes 2-3 arguments: contains(haystack, needle, case_insensitive?)".to_string(),
                ));
            }
            let case_insensitive = if args.len() == 3 {
                match &args[2] {
                    Value::Bool(b) => *b,
                    _ => return Err(LatticeError::Runtime(
                        "contains() third argument must be a boolean".to_string(),
                    )),
                }
            } else {
                false
            };
            match (&args[0], &args[1]) {
                (Value::String(haystack), Value::String(needle)) => {
                    let result = if case_insensitive {
                        haystack.to_lowercase().contains(&needle.to_lowercase())
                    } else {
                        haystack.contains(&**needle)
                    };
                    Ok(Value::Bool(result))
                }
                _ => Err(LatticeError::Runtime(format!(
                    "contains() expects (String, String, Bool?), got ({}, {})",
                    type_name(&args[0]),
                    type_name(&args[1])
                ))),
            }
        }
        "regex_match" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(LatticeError::Runtime(
                    "regex_match() takes 2-3 arguments: regex_match(text, pattern, case_insensitive?)".to_string(),
                ));
            }
            let case_insensitive = if args.len() == 3 {
                match &args[2] {
                    Value::Bool(b) => *b,
                    _ => return Err(LatticeError::Runtime(
                        "regex_match() third argument must be a boolean".to_string(),
                    )),
                }
            } else {
                false
            };
            match (&args[0], &args[1]) {
                (Value::String(text), Value::String(pattern)) => {
                    let final_pattern = if case_insensitive {
                        format!("(?i){}", pattern)
                    } else {
                        pattern.to_string()
                    };
                    match regex::Regex::new(&final_pattern) {
                        Ok(re) => Ok(Value::Bool(re.is_match(text))),
                        Err(e) => Err(LatticeError::Runtime(format!(
                            "Invalid regex pattern '{}': {}",
                            pattern, e
                        ))),
                    }
                }
                _ => Err(LatticeError::Runtime(format!(
                    "regex_match() expects (String, String, Bool?), got ({}, {})",
                    type_name(&args[0]),
                    type_name(&args[1])
                ))),
            }
        }
        _ => Err(LatticeError::Runtime(format!(
            "Unknown native function: {}",
            name
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let vm = VM::new();
        assert_eq!(vm.stack_size(), 0);
        assert_eq!(vm.frame_depth(), 0);
    }

    #[test]
    fn test_stack_operations() {
        let mut vm = VM::new();

        // Push values
        vm.push(Value::Int(42)).unwrap();
        vm.push(Value::string("hello")).unwrap();
        assert_eq!(vm.stack_size(), 2);

        // Peek
        assert!(matches!(vm.peek().unwrap(), Value::String(ref s) if &**s == "hello"));
        assert!(matches!(vm.peek_at(1).unwrap(), Value::Int(42)));

        // Pop
        let val = vm.pop().unwrap();
        assert!(matches!(val, Value::String(ref s) if &**s == "hello"));
        assert_eq!(vm.stack_size(), 1);
    }

    #[test]
    fn test_stack_underflow() {
        let mut vm = VM::new();
        assert!(vm.pop().is_err());
        assert!(vm.peek().is_err());
    }

    #[test]
    fn test_global_variables() {
        let mut vm = VM::new();

        // Set and get
        vm.set_global("x".to_string(), Value::Int(100));
        assert!(vm.has_global("x"));
        assert!(matches!(vm.get_global("x").unwrap(), Value::Int(100)));

        // Undefined variable
        assert!(vm.get_global("undefined").is_err());

        // Globals persist after reset
        vm.reset();
        assert!(vm.has_global("x"));

        // But not after clear
        vm.clear();
        assert!(!vm.has_global("x"));
    }

    #[test]
    fn test_call_frames() {
        let mut vm = VM::new();

        let func = CompiledFunction::new("test".to_string(), 0);
        let frame = CallFrame::new(func, 0);

        vm.push_frame(frame).unwrap();
        assert_eq!(vm.frame_depth(), 1);

        let current = vm.current_frame().unwrap();
        assert_eq!(current.function.name, "test");
        assert_eq!(current.ip, 0);

        vm.pop_frame().unwrap();
        assert_eq!(vm.frame_depth(), 0);
    }

    #[test]
    fn test_local_variables() {
        let mut vm = VM::new();

        // Set up a frame with some locals on the stack
        vm.push(Value::Int(10)).unwrap(); // local 0
        vm.push(Value::Int(20)).unwrap(); // local 1
        vm.push(Value::Int(30)).unwrap(); // local 2

        let func = CompiledFunction::new("test".to_string(), 0);
        let frame = CallFrame::new(func, 0);
        vm.push_frame(frame).unwrap();

        // Get locals
        assert!(matches!(vm.get_local(0).unwrap(), Value::Int(10)));
        assert!(matches!(vm.get_local(1).unwrap(), Value::Int(20)));
        assert!(matches!(vm.get_local(2).unwrap(), Value::Int(30)));

        // Set local
        vm.set_local(1, Value::Int(200)).unwrap();
        assert!(matches!(vm.get_local(1).unwrap(), Value::Int(200)));
    }

    // ========================================================================
    // Execution Tests
    // ========================================================================

    #[test]
    fn test_run_empty_chunk() {
        let mut vm = VM::new();
        let chunk = Chunk::new();
        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Null));
    }

    #[test]
    fn test_run_const() {
        let mut vm = VM::new();
        let mut chunk = Chunk::new();
        let idx = chunk.add_constant(Value::Int(42));
        chunk.write(OpCode::Const(idx), 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn test_run_arithmetic() {
        let mut vm = VM::new();

        // Test: 10 + 20 = 30
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(10));
        let idx2 = chunk.add_constant(Value::Int(20));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Add, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(30)));

        // Test: 50 - 20 = 30
        vm.reset();
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(50));
        let idx2 = chunk.add_constant(Value::Int(20));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Sub, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(30)));

        // Test: 6 * 7 = 42
        vm.reset();
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(6));
        let idx2 = chunk.add_constant(Value::Int(7));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Mul, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(42)));

        // Test: 84 / 2 = 42
        vm.reset();
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(84));
        let idx2 = chunk.add_constant(Value::Int(2));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Div, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(42)));

        // Test: 17 % 5 = 2
        vm.reset();
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(17));
        let idx2 = chunk.add_constant(Value::Int(5));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Mod, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(2)));

        // Test: -42 = -42
        vm.reset();
        let mut chunk = Chunk::new();
        let idx = chunk.add_constant(Value::Int(42));
        chunk.write(OpCode::Const(idx), 1);
        chunk.write(OpCode::Neg, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(-42)));
    }

    #[test]
    fn test_run_float_arithmetic() {
        let mut vm = VM::new();

        // Test: 3.5 + 2.5 = 6.0
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Float(3.5));
        let idx2 = chunk.add_constant(Value::Float(2.5));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Add, 1);

        let result = vm.run(&chunk).unwrap();
        match result {
            Value::Float(f) => assert!((f - 6.0).abs() < 0.0001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_run_string_concatenation() {
        let mut vm = VM::new();

        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::string("Hello, "));
        let idx2 = chunk.add_constant(Value::string("World!"));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Add, 1);

        let result = vm.run(&chunk).unwrap();
        match result {
            Value::String(s) => assert_eq!(&*s, "Hello, World!"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_run_comparison() {
        let mut vm = VM::new();

        // Test: 5 < 10 = true
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(5));
        let idx2 = chunk.add_constant(Value::Int(10));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Lt, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(true)));

        // Test: 10 == 10 = true
        vm.reset();
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(10));
        let idx2 = chunk.add_constant(Value::Int(10));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Eq, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(true)));

        // Test: 10 != 5 = true
        vm.reset();
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(10));
        let idx2 = chunk.add_constant(Value::Int(5));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Ne, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_run_logic() {
        let mut vm = VM::new();

        // Test: true && false = false
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Bool(true));
        let idx2 = chunk.add_constant(Value::Bool(false));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::And, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(false)));

        // Test: true || false = true
        vm.reset();
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Bool(true));
        let idx2 = chunk.add_constant(Value::Bool(false));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Or, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(true)));

        // Test: !true = false
        vm.reset();
        let mut chunk = Chunk::new();
        let idx = chunk.add_constant(Value::Bool(true));
        chunk.write(OpCode::Const(idx), 1);
        chunk.write(OpCode::Not, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_run_jump() {
        let mut vm = VM::new();

        // Test unconditional jump: skip pushing 999, push 42 instead
        // Const(0) [10]
        // Jump(4)
        // Const(1) [999] <- skipped
        // Pop        <- skipped
        // Const(2) [42]
        let mut chunk = Chunk::new();
        let idx_10 = chunk.add_constant(Value::Int(10));
        let idx_999 = chunk.add_constant(Value::Int(999));
        let idx_42 = chunk.add_constant(Value::Int(42));

        chunk.write(OpCode::Const(idx_10), 1);
        chunk.write(OpCode::Jump(4), 1);
        chunk.write(OpCode::Const(idx_999), 1);
        chunk.write(OpCode::Pop, 1);
        chunk.write(OpCode::Const(idx_42), 1);

        let result = vm.run(&chunk).unwrap();
        // Stack has 10, then 42 - result is top of stack
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn test_run_jump_if_false() {
        let mut vm = VM::new();

        // If false, jump to push 42
        let mut chunk = Chunk::new();
        let idx_false = chunk.add_constant(Value::Bool(false));
        let idx_999 = chunk.add_constant(Value::Int(999));
        let idx_42 = chunk.add_constant(Value::Int(42));

        chunk.write(OpCode::Const(idx_false), 1); // 0
        chunk.write(OpCode::JumpIfFalse(4), 1);   // 1 -> jump to 4 if false
        chunk.write(OpCode::Const(idx_999), 1);  // 2 <- skipped
        chunk.write(OpCode::Jump(5), 1);         // 3 <- skipped
        chunk.write(OpCode::Const(idx_42), 1);   // 4

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn test_run_global_variables() {
        let mut vm = VM::new();

        // Set global x = 42, then get it
        let mut chunk = Chunk::new();
        let idx = chunk.add_constant(Value::Int(42));
        let x_idx = chunk.intern_string("x");
        chunk.write(OpCode::Const(idx), 1);
        chunk.write(OpCode::SetGlobal(x_idx), 1);
        chunk.write(OpCode::Pop, 1);
        chunk.write(OpCode::GetGlobal(x_idx), 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn test_run_make_list() {
        let mut vm = VM::new();

        // Create list [1, 2, 3]
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(1));
        let idx2 = chunk.add_constant(Value::Int(2));
        let idx3 = chunk.add_constant(Value::Int(3));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Const(idx3), 1);
        chunk.write(OpCode::MakeList(3), 1);

        let result = vm.run(&chunk).unwrap();
        match result {
            Value::List(l) => {
                assert_eq!(l.len(), 3);
                assert!(matches!(l[0], Value::Int(1)));
                assert!(matches!(l[1], Value::Int(2)));
                assert!(matches!(l[2], Value::Int(3)));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_run_make_map() {
        let mut vm = VM::new();

        // Create map {"a": 1, "b": 2}
        let mut chunk = Chunk::new();
        let idx_a = chunk.add_constant(Value::string("a"));
        let idx_1 = chunk.add_constant(Value::Int(1));
        let idx_b = chunk.add_constant(Value::string("b"));
        let idx_2 = chunk.add_constant(Value::Int(2));

        chunk.write(OpCode::Const(idx_a), 1);
        chunk.write(OpCode::Const(idx_1), 1);
        chunk.write(OpCode::Const(idx_b), 1);
        chunk.write(OpCode::Const(idx_2), 1);
        chunk.write(OpCode::MakeMap(2), 1);

        let result = vm.run(&chunk).unwrap();
        match result {
            Value::Map(m) => {
                assert_eq!(m.len(), 2);
                assert!(matches!(m.get("a"), Some(Value::Int(1))));
                assert!(matches!(m.get("b"), Some(Value::Int(2))));
            }
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_run_index_list() {
        let mut vm = VM::new();

        // [10, 20, 30][1] = 20
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(10));
        let idx2 = chunk.add_constant(Value::Int(20));
        let idx3 = chunk.add_constant(Value::Int(30));
        let idx_i = chunk.add_constant(Value::Int(1));

        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Const(idx3), 1);
        chunk.write(OpCode::MakeList(3), 1);
        chunk.write(OpCode::Const(idx_i), 1);
        chunk.write(OpCode::Index, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(20)));
    }

    #[test]
    fn test_run_index_map() {
        let mut vm = VM::new();

        // {"x": 42}["x"] = 42
        let mut chunk = Chunk::new();
        let idx_x = chunk.add_constant(Value::string("x"));
        let idx_42 = chunk.add_constant(Value::Int(42));

        chunk.write(OpCode::Const(idx_x), 1);
        chunk.write(OpCode::Const(idx_42), 1);
        chunk.write(OpCode::MakeMap(1), 1);
        chunk.write(OpCode::Const(idx_x), 1);
        chunk.write(OpCode::Index, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn test_run_dup() {
        let mut vm = VM::new();

        // Dup 42 and add: 42 + 42 = 84
        let mut chunk = Chunk::new();
        let idx = chunk.add_constant(Value::Int(42));
        chunk.write(OpCode::Const(idx), 1);
        chunk.write(OpCode::Dup, 1);
        chunk.write(OpCode::Add, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(84)));
    }

    #[test]
    fn test_run_native_len() {
        let mut vm = VM::new();

        // len([1, 2, 3]) = 3
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(1));
        let idx2 = chunk.add_constant(Value::Int(2));
        let idx3 = chunk.add_constant(Value::Int(3));
        let len_idx = chunk.intern_string("len");

        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Const(idx3), 1);
        chunk.write(OpCode::MakeList(3), 1);
        chunk.write(OpCode::CallNative(len_idx, 1), 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(3)));
    }

    #[test]
    fn test_run_return() {
        let mut vm = VM::new();

        // Return 42 early
        let mut chunk = Chunk::new();
        let idx_42 = chunk.add_constant(Value::Int(42));
        let idx_999 = chunk.add_constant(Value::Int(999));

        chunk.write(OpCode::Const(idx_42), 1);
        chunk.write(OpCode::Return, 1);
        chunk.write(OpCode::Const(idx_999), 1); // never reached

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn test_run_get_set_field() {
        let mut vm = VM::new();

        // Create map, set field, get field
        let mut chunk = Chunk::new();
        let idx_x = chunk.add_constant(Value::string("x"));
        let idx_10 = chunk.add_constant(Value::Int(10));
        let idx_42 = chunk.add_constant(Value::Int(42));
        let x_str_idx = chunk.intern_string("x");

        // Create {"x": 10}
        chunk.write(OpCode::Const(idx_x), 1);
        chunk.write(OpCode::Const(idx_10), 1);
        chunk.write(OpCode::MakeMap(1), 1);
        // Set x = 42
        chunk.write(OpCode::Const(idx_42), 1);
        chunk.write(OpCode::SetField(x_str_idx), 1);
        // Get x
        chunk.write(OpCode::GetField(x_str_idx), 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn test_run_division_by_zero() {
        let mut vm = VM::new();

        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(10));
        let idx2 = chunk.add_constant(Value::Int(0));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Div, 1);

        let result = vm.run(&chunk);
        assert!(result.is_err());
    }

    #[test]
    fn test_truthy_values() {
        // Test truthy evaluation through Not
        let mut vm = VM::new();

        // !0 = true (0 is falsy)
        let mut chunk = Chunk::new();
        let idx = chunk.add_constant(Value::Int(0));
        chunk.write(OpCode::Const(idx), 1);
        chunk.write(OpCode::Not, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(true)));

        // !1 = false (1 is truthy)
        vm.reset();
        let mut chunk = Chunk::new();
        let idx = chunk.add_constant(Value::Int(1));
        chunk.write(OpCode::Const(idx), 1);
        chunk.write(OpCode::Not, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(false)));

        // !"" = true (empty string is falsy)
        vm.reset();
        let mut chunk = Chunk::new();
        let idx = chunk.add_constant(Value::string(""));
        chunk.write(OpCode::Const(idx), 1);
        chunk.write(OpCode::Not, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(true)));

        // !null = true (null is falsy)
        vm.reset();
        let mut chunk = Chunk::new();
        let idx = chunk.add_constant(Value::Null);
        chunk.write(OpCode::Const(idx), 1);
        chunk.write(OpCode::Not, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_mixed_int_float_arithmetic() {
        let mut vm = VM::new();

        // 10 + 2.5 = 12.5
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(10));
        let idx2 = chunk.add_constant(Value::Float(2.5));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Add, 1);

        let result = vm.run(&chunk).unwrap();
        match result {
            Value::Float(f) => assert!((f - 12.5).abs() < 0.0001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_negative_index() {
        let mut vm = VM::new();

        // [10, 20, 30][-1] = 30
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(10));
        let idx2 = chunk.add_constant(Value::Int(20));
        let idx3 = chunk.add_constant(Value::Int(30));
        let idx_neg1 = chunk.add_constant(Value::Int(-1));

        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Const(idx3), 1);
        chunk.write(OpCode::MakeList(3), 1);
        chunk.write(OpCode::Const(idx_neg1), 1);
        chunk.write(OpCode::Index, 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Int(30)));
    }

    // ========================================================================
    // map_column / map_row Integration Tests
    // ========================================================================

    /// Helper to compile and run source code
    fn compile_and_run(source: &str) -> Result<Value> {
        use crate::compiler::Compiler;
        use crate::syntax::parser;
        use crate::error::LatticeError;

        let program = parser::parse(source).map_err(|e| LatticeError::Parse(e.to_string()))?;
        let result = Compiler::compile(&program)?;
        let mut vm = VM::new();

        // Register types
        for class in result.classes {
            vm.ir_mut().classes.push(class);
        }
        for enum_def in result.enums {
            vm.ir_mut().enums.push(enum_def);
        }

        // Register functions
        for func in result.functions {
            vm.register_function(func);
        }
        for llm_func in result.llm_functions {
            vm.register_llm_function(llm_func);
        }

        vm.run(&result.chunk)
    }

    #[test]
    fn test_map_column_basic() {
        let source = r#"
def double(x: Int) -> Int {
    x * 2
}

let data = [{"val": 1}, {"val": 2}, {"val": 3}]
map_column(data, "val", "doubled", |x| double(x))"#;

        let result = compile_and_run(source).unwrap();
        match result {
            Value::List(list) => {
                assert_eq!(list.len(), 3);
                // Check first row has both "val" and "doubled"
                if let Value::Map(row) = &list[0] {
                    assert!(row.contains_key("val"));
                    assert!(row.contains_key("doubled"));
                    assert!(matches!(row.get("doubled"), Some(Value::Int(2))));
                } else {
                    panic!("Expected row to be a Map");
                }
                // Check second row
                if let Value::Map(row) = &list[1] {
                    assert!(matches!(row.get("doubled"), Some(Value::Int(4))));
                } else {
                    panic!("Expected row to be a Map");
                }
                // Check third row
                if let Value::Map(row) = &list[2] {
                    assert!(matches!(row.get("doubled"), Some(Value::Int(6))));
                } else {
                    panic!("Expected row to be a Map");
                }
            }
            _ => panic!("Expected List result"),
        }
    }

    #[test]
    fn test_map_column_pipe_syntax() {
        let source = r#"
def add_exclaim(s: String) -> String {
    s + "!"
}

let data = [{"msg": "hello"}, {"msg": "world"}]
data |> map_column("msg", "excited", |m| add_exclaim(m))"#;

        let result = compile_and_run(source).unwrap();
        match result {
            Value::List(list) => {
                assert_eq!(list.len(), 2);
                if let Value::Map(row) = &list[0] {
                    match row.get("excited") {
                        Some(Value::String(s)) => assert_eq!(&**s, "hello!"),
                        _ => panic!("Expected String value for 'excited'"),
                    }
                } else {
                    panic!("Expected row to be a Map");
                }
            }
            _ => panic!("Expected List result"),
        }
    }

    #[test]
    fn test_map_column_inline_lambda() {
        let source = r#"
let data = [{"x": 10}, {"x": 20}]
map_column(data, "x", "y", |n| n + 5)"#;

        let result = compile_and_run(source).unwrap();
        match result {
            Value::List(list) => {
                assert_eq!(list.len(), 2);
                if let Value::Map(row) = &list[0] {
                    assert!(matches!(row.get("y"), Some(Value::Int(15))));
                } else {
                    panic!("Expected row to be a Map");
                }
                if let Value::Map(row) = &list[1] {
                    assert!(matches!(row.get("y"), Some(Value::Int(25))));
                } else {
                    panic!("Expected row to be a Map");
                }
            }
            _ => panic!("Expected List result"),
        }
    }

    #[test]
    fn test_map_row_basic() {
        let source = r#"
def combine(row: Map<String, String>) -> String {
    row["a"] + " " + row["b"]
}

let data = [{"a": "hello", "b": "world"}, {"a": "foo", "b": "bar"}]
map_row(data, "combined", |r| combine(r))"#;

        let result = compile_and_run(source).unwrap();
        match result {
            Value::List(list) => {
                assert_eq!(list.len(), 2);
                if let Value::Map(row) = &list[0] {
                    assert!(row.contains_key("a"));
                    assert!(row.contains_key("b"));
                    assert!(row.contains_key("combined"));
                    match row.get("combined") {
                        Some(Value::String(s)) => assert_eq!(&**s, "hello world"),
                        _ => panic!("Expected String value for 'combined'"),
                    }
                } else {
                    panic!("Expected row to be a Map");
                }
                if let Value::Map(row) = &list[1] {
                    match row.get("combined") {
                        Some(Value::String(s)) => assert_eq!(&**s, "foo bar"),
                        _ => panic!("Expected String value for 'combined'"),
                    }
                } else {
                    panic!("Expected row to be a Map");
                }
            }
            _ => panic!("Expected List result"),
        }
    }

    #[test]
    fn test_map_row_pipe_syntax() {
        let source = r#"
def compute_sum(r: Map<String, Int>) -> Int {
    r["x"] + r["y"]
}

let data = [{"x": 1, "y": 2}, {"x": 10, "y": 20}]
data |> map_row("sum", |row| compute_sum(row))"#;

        let result = compile_and_run(source).unwrap();
        match result {
            Value::List(list) => {
                assert_eq!(list.len(), 2);
                if let Value::Map(row) = &list[0] {
                    assert!(matches!(row.get("sum"), Some(Value::Int(3))));
                } else {
                    panic!("Expected row to be a Map");
                }
                if let Value::Map(row) = &list[1] {
                    assert!(matches!(row.get("sum"), Some(Value::Int(30))));
                } else {
                    panic!("Expected row to be a Map");
                }
            }
            _ => panic!("Expected List result"),
        }
    }

    #[test]
    fn test_map_row_inline_lambda() {
        let source = r#"
let data = [{"a": 5, "b": 3}, {"a": 10, "b": 2}]
map_row(data, "product", |r| r["a"] * r["b"])"#;

        let result = compile_and_run(source).unwrap();
        match result {
            Value::List(list) => {
                assert_eq!(list.len(), 2);
                if let Value::Map(row) = &list[0] {
                    assert!(matches!(row.get("product"), Some(Value::Int(15))));
                } else {
                    panic!("Expected row to be a Map");
                }
                if let Value::Map(row) = &list[1] {
                    assert!(matches!(row.get("product"), Some(Value::Int(20))));
                } else {
                    panic!("Expected row to be a Map");
                }
            }
            _ => panic!("Expected List result"),
        }
    }

    #[test]
    fn test_map_column_chain() {
        let source = r#"
let data = [{"x": 1}, {"x": 2}]
data |> map_column("x", "y", |n| n * 2) |> map_column("y", "z", |n| n + 10)"#;

        let result = compile_and_run(source).unwrap();
        match result {
            Value::List(list) => {
                assert_eq!(list.len(), 2);
                // First row: x=1, y=2, z=12
                if let Value::Map(row) = &list[0] {
                    assert!(matches!(row.get("x"), Some(Value::Int(1))));
                    assert!(matches!(row.get("y"), Some(Value::Int(2))));
                    assert!(matches!(row.get("z"), Some(Value::Int(12))));
                } else {
                    panic!("Expected row to be a Map");
                }
                // Second row: x=2, y=4, z=14
                if let Value::Map(row) = &list[1] {
                    assert!(matches!(row.get("x"), Some(Value::Int(2))));
                    assert!(matches!(row.get("y"), Some(Value::Int(4))));
                    assert!(matches!(row.get("z"), Some(Value::Int(14))));
                } else {
                    panic!("Expected row to be a Map");
                }
            }
            _ => panic!("Expected List result"),
        }
    }

    #[test]
    fn test_map_column_and_row_chain() {
        let source = r#"
let data = [{"a": 2, "b": 3}]
data |> map_column("a", "a2", |n| n * 2) |> map_row("sum", |r| r["a2"] + r["b"])"#;

        let result = compile_and_run(source).unwrap();
        match result {
            Value::List(list) => {
                assert_eq!(list.len(), 1);
                // a=2, b=3, a2=4, sum=7
                if let Value::Map(row) = &list[0] {
                    assert!(matches!(row.get("a"), Some(Value::Int(2))));
                    assert!(matches!(row.get("b"), Some(Value::Int(3))));
                    assert!(matches!(row.get("a2"), Some(Value::Int(4))));
                    assert!(matches!(row.get("sum"), Some(Value::Int(7))));
                } else {
                    panic!("Expected row to be a Map");
                }
            }
            _ => panic!("Expected List result"),
        }
    }

    #[test]
    fn test_map_column_empty_table() {
        let source = r#"
let data: [Map<String, Int>] = []
map_column(data, "x", "y", |n| n * 2)"#;

        let result = compile_and_run(source).unwrap();
        match result {
            Value::List(list) => {
                assert_eq!(list.len(), 0);
            }
            _ => panic!("Expected List result"),
        }
    }

    #[test]
    fn test_map_row_empty_table() {
        let source = r#"
let data: [Map<String, Int>] = []
map_row(data, "result", |r| r["x"])"#;

        let result = compile_and_run(source).unwrap();
        match result {
            Value::List(list) => {
                assert_eq!(list.len(), 0);
            }
            _ => panic!("Expected List result"),
        }
    }

    // ========================================================================
    // String Functions Tests
    // ========================================================================

    #[test]
    fn test_contains_true() {
        let source = r#"contains("hello world", "world")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_contains_false() {
        let source = r#"contains("hello world", "foo")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_contains_empty_needle() {
        let source = r#"contains("hello", "")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_contains_case_insensitive_flag() {
        let source = r#"contains("Hello World", "hello", true)"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_contains_case_insensitive_flag_false() {
        let source = r#"contains("Hello World", "hello", false)"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_contains_with_variables() {
        let source = r#"
let text = "The quick brown fox"
let needle = "quick"
contains(text, needle)"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_word_count_basic() {
        let source = r#"word_count("hello world")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Int(2)));
    }

    #[test]
    fn test_word_count_multiple_spaces() {
        let source = r#"word_count("hello   world   foo")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Int(3)));
    }

    #[test]
    fn test_word_count_empty_string() {
        let source = r#"word_count("")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Int(0)));
    }

    #[test]
    fn test_word_count_whitespace_only() {
        let source = r#"word_count("   ")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Int(0)));
    }

    #[test]
    fn test_regex_match_simple() {
        let source = r#"regex_match("hello world", "wor.d")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_regex_match_word_boundary() {
        let source = r#"regex_match("he said hello", "\\bhe\\b")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_regex_match_no_match() {
        let source = r#"regex_match("hello world", "^world")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_regex_match_pronouns() {
        // Test the pronoun matching use case
        let source = r#"regex_match(" he told her ", "\\b(he|she|him|her|his|hers|himself|herself)\\b")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_regex_match_no_pronouns() {
        let source = r#"regex_match("the employee completed the task", "\\b(he|she|him|her|his|hers|himself|herself)\\b")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_regex_match_case_sensitive() {
        let source = r#"regex_match("Hello World", "hello")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_regex_match_case_insensitive() {
        let source = r#"regex_match("Hello World", "(?i)hello")"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_regex_match_case_insensitive_flag() {
        // Using the third parameter for case insensitivity
        let source = r#"regex_match("Hello World", "hello", true)"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_regex_match_case_insensitive_flag_false() {
        // Explicitly case-sensitive
        let source = r#"regex_match("Hello World", "hello", false)"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_regex_match_pronouns_case_insensitive() {
        // Test pronoun matching at start of sentence with case insensitivity
        let source = r#"regex_match("She went home", "\\b(he|she|him|her)\\b", true)"#;
        let result = compile_and_run(source).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_native_contains() {
        let mut vm = VM::new();

        // contains("hello", "ell") = true
        let mut chunk = Chunk::new();
        let idx_haystack = chunk.add_constant(Value::string("hello world"));
        let idx_needle = chunk.add_constant(Value::string("world"));
        let contains_idx = chunk.intern_string("contains");

        chunk.write(OpCode::Const(idx_haystack), 1);
        chunk.write(OpCode::Const(idx_needle), 1);
        chunk.write(OpCode::CallNative(contains_idx, 2), 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_native_regex_match() {
        let mut vm = VM::new();

        // regex_match("hello", "h.llo") = true
        let mut chunk = Chunk::new();
        let idx_text = chunk.add_constant(Value::string("hello"));
        let idx_pattern = chunk.add_constant(Value::string("h.llo"));
        let regex_idx = chunk.intern_string("regex_match");

        chunk.write(OpCode::Const(idx_text), 1);
        chunk.write(OpCode::Const(idx_pattern), 1);
        chunk.write(OpCode::CallNative(regex_idx, 2), 1);

        let result = vm.run(&chunk).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }
}
