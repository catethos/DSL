//! Virtual Machine implementation for Lattice
//!
//! This module defines the VM struct, call frames, and stack operations
//! for executing bytecode.

use std::collections::HashMap;

use crate::error::{LatticeError, Result};
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
/// - SQL context for DuckDB queries
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
    /// SQL execution context (DuckDB connection)
    sql_context: SqlContext,
    /// Debug info from the last LLM call (if any)
    last_llm_debug: Option<LlmDebugInfo>,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    /// Create a new VM instance
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: HashMap::new(),
            ir: IR::new(),
            llm_functions: Vec::new(),
            user_functions: HashMap::new(),
            sql_context: SqlContext::default(),
            last_llm_debug: None,
        }
    }

    /// Create a new VM instance with a file-based DuckDB database
    pub fn with_database(path: &str) -> Result<Self> {
        let sql_context = SqlContext::open(path).map_err(|e| {
            LatticeError::Runtime(format!("Failed to open database: {}", e))
        })?;
        Ok(Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: HashMap::new(),
            ir: IR::new(),
            llm_functions: Vec::new(),
            user_functions: HashMap::new(),
            sql_context,
            last_llm_debug: None,
        })
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
    pub fn sql_context(&self) -> &SqlContext {
        &self.sql_context
    }

    /// Get a mutable reference to the SQL context
    pub fn sql_context_mut(&mut self) -> &mut SqlContext {
        &mut self.sql_context
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

    /// The main fetch-decode-execute loop
    fn execute(&mut self) -> Result<Value> {
        loop {
            // Fetch: get current instruction
            let (op, line) = {
                let frame = self.current_frame()?;
                if frame.ip >= frame.function.chunk.code.len() {
                    // End of code reached - return Null if nothing on stack
                    return if self.stack.is_empty() {
                        Ok(Value::Null)
                    } else {
                        self.pop()
                    };
                }
                let op = frame.function.chunk.code[frame.ip].clone();
                let line = frame.function.chunk.lines.get(frame.ip).copied().unwrap_or(0);
                (op, line)
            };

            // Advance IP before execution (so jumps work correctly)
            self.current_frame_mut()?.ip += 1;

            // Decode and execute
            match op {
                // Stack Operations
                OpCode::Const(idx) => self.op_const(idx)?,
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::Dup => self.op_dup()?,

                // Variables
                OpCode::GetLocal(slot) => self.op_get_local(slot)?,
                OpCode::SetLocal(slot) => self.op_set_local(slot)?,
                OpCode::GetGlobal(ref name) => self.op_get_global(name)?,
                OpCode::SetGlobal(ref name) => self.op_set_global(name.clone())?,

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
                OpCode::Jump(target) => self.op_jump(target)?,
                OpCode::JumpIfFalse(target) => self.op_jump_if_false(target)?,
                OpCode::JumpIfTrue(target) => self.op_jump_if_true(target)?,

                // Functions
                OpCode::Call(arg_count) => self.op_call(arg_count)?,
                OpCode::CallNative(ref name, arg_count) => {
                    self.op_call_native(name, arg_count, line)?
                }
                OpCode::CallUser(ref name, arg_count) => {
                    self.op_call_user(name, arg_count)?
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
                OpCode::MakeStruct(ref type_name, field_count) => {
                    self.op_make_struct(type_name.clone(), field_count)?
                }
                OpCode::GetField(ref field_name) => self.op_get_field(field_name)?,
                OpCode::SetField(ref field_name) => self.op_set_field(field_name.clone())?,

                // Async / Special Operations
                OpCode::LlmCall(ref func_name) => self.op_llm_call(func_name)?,
                OpCode::SqlQuery => self.op_sql_query()?,
                OpCode::SqlQueryTyped(ref type_name) => self.op_sql_query_typed(type_name)?,
                OpCode::Parallel(count) => self.op_parallel(count)?,
                OpCode::ParallelMap => self.op_parallel_map()?,
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

    fn op_const(&mut self, idx: usize) -> Result<()> {
        let frame = self.current_frame()?;
        let value = frame
            .function
            .chunk
            .constants
            .get(idx)
            .cloned()
            .ok_or_else(|| {
                LatticeError::Runtime(format!("Invalid constant index: {}", idx))
            })?;
        self.push(value)
    }

    fn op_dup(&mut self) -> Result<()> {
        let value = self.peek()?.clone();
        self.push(value)
    }

    // ========================================================================
    // Variable Handlers
    // ========================================================================

    fn op_get_local(&mut self, slot: usize) -> Result<()> {
        let value = self.get_local(slot)?.clone();
        self.push(value)
    }

    fn op_set_local(&mut self, slot: usize) -> Result<()> {
        let value = self.peek()?.clone();
        self.set_local(slot, value)
    }

    fn op_get_global(&mut self, name: &str) -> Result<()> {
        let value = self.get_global(name)?.clone();
        self.push(value)
    }

    fn op_set_global(&mut self, name: String) -> Result<()> {
        let value = self.peek()?.clone();
        self.set_global(name, value);
        Ok(())
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
            (Value::String(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
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
    // Control Flow Handlers
    // ========================================================================

    fn op_jump(&mut self, target: usize) -> Result<()> {
        self.current_frame_mut()?.ip = target;
        Ok(())
    }

    fn op_jump_if_false(&mut self, target: usize) -> Result<()> {
        let condition = self.pop()?;
        if !is_truthy(&condition) {
            self.current_frame_mut()?.ip = target;
        }
        Ok(())
    }

    fn op_jump_if_true(&mut self, target: usize) -> Result<()> {
        let condition = self.pop()?;
        if is_truthy(&condition) {
            self.current_frame_mut()?.ip = target;
        }
        Ok(())
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
            .get(&func_name)
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
        self.push(Value::List(items))
    }

    fn op_make_map(&mut self, count: usize) -> Result<()> {
        let mut map = HashMap::new();
        for _ in 0..count {
            let value = self.pop()?;
            let key = self.pop()?;
            let key_str = match key {
                Value::String(s) => s,
                _ => {
                    return Err(LatticeError::Runtime(
                        "Map keys must be strings".to_string(),
                    ))
                }
            };
            map.insert(key_str, value);
        }
        self.push(Value::Map(map))
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
                map.get(key).cloned().ok_or_else(|| {
                    LatticeError::Runtime(format!("Key '{}' not found in map", key))
                })?
            }
            (Value::String(s), Value::Int(i)) => {
                let idx = if *i < 0 {
                    (s.len() as i64 + *i) as usize
                } else {
                    *i as usize
                };
                s.chars().nth(idx).map(|c| Value::String(c.to_string())).ok_or_else(|| {
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
                list[idx] = value;
            }
            (Value::Map(map), Value::String(key)) => {
                map.insert(key.clone(), value);
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
        fields.insert("__type".to_string(), Value::String(type_name));

        // Pop field values (in reverse order, so we need to collect and reverse)
        let mut field_values = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            let value = self.pop()?;
            let name = self.pop()?;
            match name {
                Value::String(s) => field_values.push((s, value)),
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

        self.push(Value::Map(fields))
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
            Value::Map(map) => {
                map.insert(field_name, value);
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
        let string_value = match value {
            Value::String(s) => s,
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Path(p) => p.display().to_string(),
            Value::List(items) => format!("{}", Value::List(items)),
            Value::Map(map) => format!("{}", Value::Map(map)),
        };
        self.push(Value::String(string_value))
    }

    // ========================================================================
    // Async / Special Operation Handlers
    // ========================================================================

    /// Call an LLM function by name
    ///
    /// Pops arguments from the stack, calls the LLM, parses the response,
    /// and pushes the result (or error) onto the stack.
    ///
    /// Note: This is a synchronous wrapper around async LLM calls.
    /// It uses tokio's Runtime to block on the async operation.
    fn op_llm_call(&mut self, func_name: &str) -> Result<()> {
        use crate::llm::{generate_prompt_from_ir, parse_llm_response_with_ir, LLMClient};

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

        // Build the params HashMap from args and input_names
        let input_names = func.input_names();
        let mut params = HashMap::new();
        for (name, value) in input_names.iter().zip(args.into_iter()) {
            params.insert(name.clone(), value);
        }

        // Generate the prompt
        let prompt = generate_prompt_from_ir(
            &self.ir,
            &func.prompt_template,
            &params,
            &func.return_type,
        ).map_err(|e| LatticeError::Runtime(format!("Failed to render prompt: {}", e)))?;

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

        // Create the LLM client with the function's configuration
        let mut client = LLMClient::custom(api_key, func.base_url.clone(), func.model.clone());

        if let Some(temp) = func.temperature {
            client = client.with_temperature(temp as f32);
        }
        if let Some(max_tok) = func.max_tokens {
            client = client.with_max_tokens(max_tok as u32);
        }

        // Call the LLM (blocking on async)
        let raw_response = tokio::runtime::Runtime::new()
            .map_err(|e| LatticeError::Runtime(format!("Failed to create async runtime: {}", e)))?
            .block_on(client.call(&prompt))
            .map_err(|e| {
                // Store debug info even on error
                self.last_llm_debug = Some(debug_info.clone());
                LatticeError::Runtime(format!("LLM call failed: {}", e))
            })?;

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

    /// Execute a SQL query via DuckDB
    ///
    /// Pops query string from stack, executes via DuckDB,
    /// pushes List<Map<String, Value>> (rows) onto stack.
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

        // Execute query via DuckDB
        let result = self.sql_context.execute_query(&query_str)?;

        // Push result onto stack
        self.push(result)
    }

    /// Execute a SQL query with typed results
    ///
    /// Pops query string from stack, executes via DuckDB,
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

        // Execute query via DuckDB
        let result = self.sql_context.execute_query(&query_str)?;

        // Convert result rows to typed structs
        let typed_result = match result {
            Value::List(rows) => {
                let mut typed_rows = Vec::with_capacity(rows.len());
                for row in rows {
                    if let Value::Map(mut map) = row {
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
                        map.insert("__type".to_string(), Value::String(type_name.to_string()));
                        typed_rows.push(Value::Map(map));
                    } else {
                        return Err(LatticeError::Runtime(
                            "SQL result row is not a map".to_string(),
                        ));
                    }
                }
                Value::List(typed_rows)
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
        let _func = self.pop()?;
        let _collection = self.pop()?;

        // TODO: Implement parallel map
        // 1. Verify collection is a List
        // 2. Apply function to each element concurrently
        // 3. Collect results
        // 4. Push List onto stack

        Err(LatticeError::Runtime(
            "Parallel map not yet implemented".to_string(),
        ))
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
        "type" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "type() takes exactly 1 argument".to_string(),
                ));
            }
            Ok(Value::String(type_name(&args[0]).to_string()))
        }
        "str" => {
            if args.len() != 1 {
                return Err(LatticeError::Runtime(
                    "str() takes exactly 1 argument".to_string(),
                ));
            }
            Ok(Value::String(format!("{}", args[0])))
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
                    let mut list = l.clone();
                    list.push(args[1].clone());
                    Ok(Value::List(list))
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
                        let mut list = l.clone();
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
                    let keys: Vec<Value> = m.keys().map(|k| Value::String(k.clone())).collect();
                    Ok(Value::List(keys))
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
                    Ok(Value::List(values))
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
                Value::String(s) => Ok(Value::Path(std::path::PathBuf::from(s))),
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
                Value::Path(p) => p.clone(),
                Value::String(s) => std::path::PathBuf::from(s),
                _ => return Err(LatticeError::Runtime(format!(
                    "path_join() first argument must be Path or String, got {}",
                    type_name(&args[0])
                ))),
            };
            let to_join = match &args[1] {
                Value::Path(p) => p.clone(),
                Value::String(s) => std::path::PathBuf::from(s),
                _ => return Err(LatticeError::Runtime(format!(
                    "path_join() second argument must be Path or String, got {}",
                    type_name(&args[1])
                ))),
            };
            Ok(Value::Path(base.join(to_join)))
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
                        Some(parent) => Ok(Value::Path(parent.to_path_buf())),
                        None => Ok(Value::Null),
                    }
                }
                Value::String(s) => {
                    let p = std::path::Path::new(s);
                    match p.parent() {
                        Some(parent) => Ok(Value::Path(parent.to_path_buf())),
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
                        Some(name) => Ok(Value::String(name.to_string_lossy().to_string())),
                        None => Ok(Value::Null),
                    }
                }
                Value::String(s) => {
                    let p = std::path::Path::new(s);
                    match p.file_name() {
                        Some(name) => Ok(Value::String(name.to_string_lossy().to_string())),
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
                        Some(ext) => Ok(Value::String(ext.to_string_lossy().to_string())),
                        None => Ok(Value::Null),
                    }
                }
                Value::String(s) => {
                    let p = std::path::Path::new(s);
                    match p.extension() {
                        Some(ext) => Ok(Value::String(ext.to_string_lossy().to_string())),
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
                Value::String(s) => Ok(Value::Bool(std::path::Path::new(s).exists())),
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
                Value::String(s) => Ok(Value::Bool(std::path::Path::new(s).is_file())),
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
                Value::String(s) => Ok(Value::Bool(std::path::Path::new(s).is_dir())),
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
                Value::Path(p) => Ok(Value::String(p.display().to_string())),
                Value::String(s) => Ok(Value::String(s.clone())),
                _ => Err(LatticeError::Runtime(format!(
                    "path_to_str() expects Path or String, got {}",
                    type_name(&args[0])
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
        vm.push(Value::String("hello".to_string())).unwrap();
        assert_eq!(vm.stack_size(), 2);

        // Peek
        assert!(matches!(vm.peek().unwrap(), Value::String(s) if s == "hello"));
        assert!(matches!(vm.peek_at(1).unwrap(), Value::Int(42)));

        // Pop
        let val = vm.pop().unwrap();
        assert!(matches!(val, Value::String(s) if s == "hello"));
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
        let idx1 = chunk.add_constant(Value::String("Hello, ".to_string()));
        let idx2 = chunk.add_constant(Value::String("World!".to_string()));
        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Add, 1);

        let result = vm.run(&chunk).unwrap();
        match result {
            Value::String(s) => assert_eq!(s, "Hello, World!"),
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
        chunk.write(OpCode::Const(idx), 1);
        chunk.write(OpCode::SetGlobal("x".to_string()), 1);
        chunk.write(OpCode::Pop, 1);
        chunk.write(OpCode::GetGlobal("x".to_string()), 1);

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
        let idx_a = chunk.add_constant(Value::String("a".to_string()));
        let idx_1 = chunk.add_constant(Value::Int(1));
        let idx_b = chunk.add_constant(Value::String("b".to_string()));
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
        let idx_x = chunk.add_constant(Value::String("x".to_string()));
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

        chunk.write(OpCode::Const(idx1), 1);
        chunk.write(OpCode::Const(idx2), 1);
        chunk.write(OpCode::Const(idx3), 1);
        chunk.write(OpCode::MakeList(3), 1);
        chunk.write(OpCode::CallNative("len".to_string(), 1), 1);

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
        let idx_x = chunk.add_constant(Value::String("x".to_string()));
        let idx_10 = chunk.add_constant(Value::Int(10));
        let idx_42 = chunk.add_constant(Value::Int(42));

        // Create {"x": 10}
        chunk.write(OpCode::Const(idx_x), 1);
        chunk.write(OpCode::Const(idx_10), 1);
        chunk.write(OpCode::MakeMap(1), 1);
        // Set x = 42
        chunk.write(OpCode::Const(idx_42), 1);
        chunk.write(OpCode::SetField("x".to_string()), 1);
        // Get x
        chunk.write(OpCode::GetField("x".to_string()), 1);

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
        let idx = chunk.add_constant(Value::String("".to_string()));
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
}
