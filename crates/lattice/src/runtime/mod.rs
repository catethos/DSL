//! Runtime module for embeddable Lattice
//!
//! This module provides FFI-safe types and APIs for embedding Lattice
//! in other languages via bindings (Rustler, PyO3, Neon, etc.).
//!
//! ## Key Components
//!
//! - [`LatticeRuntime`]: The main embedding API for evaluating Lattice code
//! - [`LatticeValue`]: FFI-safe value type for cross-language marshaling
//! - [`TypeSchema`]: FFI-safe type schema for host language codegen
//! - [`FunctionSignature`]: FFI-safe function signatures for discovering callable functions
//! - [`providers`]: Injectable provider traits (LLM, SQL, etc.)
//! - [`RuntimeBuilder`]: Builder pattern for constructing isolated runtime instances
//!
//! ## Example
//!
//! ```ignore
//! use lattice::runtime::{LatticeRuntime, RuntimeBuilder, LatticeValue};
//!
//! // Create a runtime with default providers
//! let mut runtime = LatticeRuntime::builder()
//!     .with_default_providers()?
//!     .build()?;
//!
//! // Evaluate code
//! let result = runtime.eval("1 + 2")?;
//! assert_eq!(result, LatticeValue::Int(3));
//!
//! // Create a minimal runtime without LLM/SQL
//! let mut minimal = LatticeRuntime::builder()
//!     .without_llm()
//!     .without_sql()
//!     .build()?;
//! ```

mod builder;
mod schema;
mod signature;
mod value;
pub mod providers;

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::compiler::Compiler;
use crate::error::LatticeError;
use crate::syntax::{imports, parser};
use crate::vm::VM;

pub use builder::{BuiltRuntime, RuntimeBuilder, RuntimeConfig};
pub use schema::{EnumSchema, FieldSchema, StructSchema, TypeSchema};
pub use signature::{FunctionSignature, ParameterSchema};
pub use value::{ConversionError, LatticeValue};

// Re-export LlmDebugInfo from VM for FFI access
pub use crate::vm::LlmDebugInfo;
pub use providers::{
    DefaultLlmProvider, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmUsage,
    NoLlmProvider, NoSqlProvider, SqlError, SqlProvider, SqlResult, SqlRow,
};

#[cfg(feature = "sql")]
pub use providers::DuckDbProvider;

/// The main embedding API for Lattice.
///
/// `LatticeRuntime` provides a complete, isolated environment for evaluating
/// Lattice code. Each runtime instance is fully independent with no shared
/// global state.
///
/// # Example
///
/// ```ignore
/// use lattice::runtime::{LatticeRuntime, LatticeValue};
///
/// let mut runtime = LatticeRuntime::builder()
///     .without_llm()
///     .without_sql()
///     .build()?;
///
/// // Evaluate expressions
/// let result = runtime.eval("1 + 2 * 3")?;
/// assert_eq!(result, LatticeValue::Int(7));
///
/// // Evaluate with bindings
/// let result = runtime.eval_with_bindings(
///     "x + y",
///     vec![
///         ("x".to_string(), LatticeValue::Int(10)),
///         ("y".to_string(), LatticeValue::Int(20)),
///     ],
/// )?;
/// assert_eq!(result, LatticeValue::Int(30));
/// ```
pub struct LatticeRuntime {
    /// Internal VM instance
    vm: VM,
    /// Runtime configuration
    config: RuntimeConfig,
    /// Known user function names (for multi-cell evaluation)
    known_functions: Vec<String>,
    /// Known LLM function names (for multi-cell evaluation)
    known_llm_functions: Vec<String>,
}

impl LatticeRuntime {
    /// Create a new RuntimeBuilder for configuring a LatticeRuntime.
    ///
    /// This is the preferred way to create a LatticeRuntime instance.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let runtime = LatticeRuntime::builder()
    ///     .with_default_providers()?
    ///     .with_timeout(Duration::from_secs(30))
    ///     .build()?;
    /// ```
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::new()
    }

    /// Create a LatticeRuntime from a BuiltRuntime configuration.
    ///
    /// This is called by `RuntimeBuilder::build()` or can be used directly
    /// if you have a `BuiltRuntime` instance.
    ///
    /// The providers from the BuiltRuntime are injected into the VM,
    /// enabling injectable LLM and SQL capabilities.
    pub fn from_built(built: BuiltRuntime) -> Self {
        // Create VM with the configured providers
        let mut vm = VM::with_providers(built.llm_provider, built.sql_provider);

        // Apply configuration
        if let Some(limit) = built.config.max_concurrent_llm_calls {
            vm.set_max_concurrent_llm_calls(Some(limit));
        }

        // Copy IR (type definitions) to VM
        for class in &built.ir.classes {
            vm.ir_mut().classes.push(class.clone());
        }
        for enum_def in &built.ir.enums {
            vm.ir_mut().enums.push(enum_def.clone());
        }

        Self {
            vm,
            config: built.config,
            known_functions: Vec::new(),
            known_llm_functions: Vec::new(),
        }
    }

    /// Evaluate source code and return the result.
    ///
    /// This is the main entry point for evaluating Lattice code. The source
    /// is parsed, compiled, and executed, with the final value returned.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = runtime.eval("let x = 10\nlet y = 20\nx + y")?;
    /// assert_eq!(result, LatticeValue::Int(30));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The source fails to parse
    /// - Compilation fails (type errors, undefined variables, etc.)
    /// - Execution fails (runtime errors)
    /// - The result cannot be converted to LatticeValue
    pub fn eval(&mut self, source: &str) -> Result<LatticeValue, LatticeError> {
        // Parse source
        let program = parser::parse(source)?;

        // Compile with known function names
        let result = Compiler::compile_with_known_functions_full(
            &program,
            self.known_functions.clone(),
            self.known_llm_functions.clone(),
        )?;

        // Register types
        for class in result.classes {
            self.vm.ir_mut().classes.push(class);
        }
        for enum_def in result.enums {
            self.vm.ir_mut().enums.push(enum_def);
        }

        // Register functions and track names for future cells
        for func in result.functions {
            self.known_functions.push(func.name.clone());
            self.vm.register_function(func);
        }
        for llm_func in result.llm_functions {
            self.known_llm_functions.push(llm_func.name.clone());
            self.vm.register_llm_function(llm_func);
        }

        // Execute
        let internal_result = self.vm.run(&result.chunk)?;

        // Convert to FFI-safe value
        LatticeValue::from_internal(&internal_result).map_err(|e| {
            LatticeError::Runtime(format!("Failed to convert result: {}", e))
        })
    }

    /// Evaluate a file with import resolution.
    ///
    /// This reads the file, resolves any import statements by including the
    /// contents of imported files, then evaluates the result.
    ///
    /// Supports both `.lat` and `.md` files:
    /// - `.lat` files: Standard Lattice source with import resolution
    /// - `.md` files: Markdown LLM functions, transpiled to Lattice source
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to evaluate
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Given files:
    /// // types.lat: type Person { name: String }
    /// // main.lat:  import "types.lat"
    /// //            Person { name: "Alice" }
    ///
    /// let result = runtime.eval_file(Path::new("main.lat"))?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - An imported file cannot be found
    /// - Circular imports are detected
    /// - Parsing or execution fails
    /// - For `.md` files: invalid frontmatter or missing required fields
    pub fn eval_file(&mut self, path: &Path) -> Result<LatticeValue, LatticeError> {
        // Read the source file
        let source = std::fs::read_to_string(path).map_err(|e| {
            LatticeError::Runtime(format!("Cannot read file '{}': {}", path.display(), e))
        })?;

        // Handle based on file extension
        let source = if path.extension().map_or(false, |ext| ext == "md") {
            // Markdown LLM file: transpile to Lattice source
            use crate::syntax::markdown::parse_markdown_llm;
            let md_def = parse_markdown_llm(&source).map_err(|e| {
                LatticeError::Parse(format!(
                    "Error parsing markdown file '{}': {}",
                    path.display(),
                    e
                ))
            })?;
            md_def.to_lattice_source()
        } else {
            source
        };

        // Resolve imports relative to the file's directory
        let base_path = path.parent().unwrap_or(Path::new("."));
        let resolved_source = imports::resolve_imports(&source, base_path)?;

        // Evaluate the resolved source
        self.eval(&resolved_source)
    }

    /// Evaluate source code with import resolution relative to a base path.
    ///
    /// This is useful for notebooks where cells contain code with imports
    /// but the notebook itself is saved at a specific location. Imports
    /// will be resolved relative to the provided base path.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Notebook saved at /home/user/notebooks/demo.lat.nb
    /// let base_path = Path::new("/home/user/notebooks");
    /// let result = runtime.eval_with_base_path(
    ///     r#"import "utils/helpers.lat" as h
    ///        h.greet("World")"#,
    ///     base_path,
    /// )?;
    /// // Will look for /home/user/notebooks/utils/helpers.lat
    /// ```
    pub fn eval_with_base_path(
        &mut self,
        source: &str,
        base_path: &Path,
    ) -> Result<LatticeValue, LatticeError> {
        let resolved_source = imports::resolve_imports(source, base_path)?;
        self.eval(&resolved_source)
    }

    /// Evaluate source code with pre-bound variables.
    ///
    /// The bindings are set as global variables before evaluation.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = runtime.eval_with_bindings(
    ///     "name + \" is \" + str(age) + \" years old\"",
    ///     vec![
    ///         ("name".to_string(), LatticeValue::String("Alice".to_string())),
    ///         ("age".to_string(), LatticeValue::Int(30)),
    ///     ],
    /// )?;
    /// ```
    pub fn eval_with_bindings(
        &mut self,
        source: &str,
        bindings: Vec<(String, LatticeValue)>,
    ) -> Result<LatticeValue, LatticeError> {
        // Set bindings as globals before eval
        for (name, value) in bindings {
            self.vm.set_global(name, value.to_internal());
        }
        self.eval(source)
    }

    /// Get all registered type schemas (classes and enums).
    ///
    /// This is useful for generating typed structures in host languages.
    ///
    /// # Example
    ///
    /// ```ignore
    /// runtime.eval("type Person { name: String, age: Int }")?;
    /// let types = runtime.get_types();
    /// // Generate Elixir/Python/TypeScript types from schemas
    /// ```
    pub fn get_types(&self) -> Vec<TypeSchema> {
        let mut schemas = Vec::new();

        for class in &self.vm.ir().classes {
            schemas.push(TypeSchema::from_class(class));
        }

        for enum_def in &self.vm.ir().enums {
            schemas.push(TypeSchema::from_enum(enum_def));
        }

        schemas
    }

    /// Get a specific type schema by name.
    ///
    /// Returns None if the type is not found.
    pub fn get_type(&self, name: &str) -> Option<TypeSchema> {
        if let Some(class) = self.vm.ir().find_class(name) {
            return Some(TypeSchema::from_class(class));
        }

        if let Some(enum_def) = self.vm.ir().find_enum(name) {
            return Some(TypeSchema::from_enum(enum_def));
        }

        None
    }

    /// Register an external type (for host-defined types).
    ///
    /// This allows host languages to define types that can be used in Lattice code.
    ///
    /// # Errors
    ///
    /// Returns an error if the schema cannot be converted to an internal type.
    pub fn register_type(&mut self, schema: TypeSchema) -> Result<(), LatticeError> {
        match &schema {
            TypeSchema::Struct(_) => {
                if let Some(class) = schema.to_class() {
                    self.vm.ir_mut().classes.push(class);
                    Ok(())
                } else {
                    Err(LatticeError::Runtime(
                        "Failed to convert schema to class".to_string(),
                    ))
                }
            }
            TypeSchema::Enum(_) => {
                if let Some(enum_def) = schema.to_enum() {
                    self.vm.ir_mut().enums.push(enum_def);
                    Ok(())
                } else {
                    Err(LatticeError::Runtime(
                        "Failed to convert schema to enum".to_string(),
                    ))
                }
            }
            _ => Err(LatticeError::Runtime(
                "Only Struct and Enum schemas can be registered".to_string(),
            )),
        }
    }

    /// Get the current value of a global variable.
    ///
    /// Returns None if the variable doesn't exist.
    pub fn get_global(&self, name: &str) -> Option<LatticeValue> {
        self.vm
            .get_global(name)
            .ok()
            .and_then(|v| LatticeValue::from_internal(v).ok())
    }

    /// Set a global variable.
    ///
    /// This persists across evaluations until the runtime is reset.
    pub fn set_global(&mut self, name: &str, value: LatticeValue) {
        self.vm.set_global(name.to_string(), value.to_internal());
    }

    /// Get all global variable names.
    pub fn global_names(&self) -> Vec<String> {
        self.vm.global_names().cloned().collect()
    }

    /// Clear all state (globals, types, functions).
    ///
    /// This returns the runtime to its initial state.
    pub fn reset(&mut self) {
        self.vm.clear();
        self.known_functions.clear();
        self.known_llm_functions.clear();
    }

    /// Get the runtime configuration.
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Check if the runtime has LLM support configured.
    pub fn has_llm(&self) -> bool {
        // Check if we have a non-No provider
        // For now, we assume LLM is available if config says so
        true
    }

    /// Check if the runtime has SQL support configured.
    #[cfg(feature = "sql")]
    pub fn has_sql(&self) -> bool {
        true
    }

    /// Get the list of registered function names.
    pub fn function_names(&self) -> &[String] {
        &self.known_functions
    }

    /// Get the list of registered LLM function names.
    pub fn llm_function_names(&self) -> &[String] {
        &self.known_llm_functions
    }

    /// Get debug info from the last LLM call, if any.
    ///
    /// This returns a reference to the debug info without consuming it.
    /// Use `take_llm_debug()` if you want to take ownership and clear the info.
    pub fn last_llm_debug(&self) -> Option<&crate::vm::LlmDebugInfo> {
        self.vm.last_llm_debug()
    }

    /// Take debug info from the last LLM call, clearing it from the runtime.
    ///
    /// This is useful for the notebook UI to display LLM debug information
    /// (prompt, raw response, function name, return type) after evaluating code.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = runtime.eval("summarize(\"hello world\")")?;
    /// if let Some(debug) = runtime.take_llm_debug() {
    ///     println!("Prompt: {}", debug.prompt);
    ///     println!("Response: {}", debug.raw_response);
    /// }
    /// ```
    pub fn take_llm_debug(&mut self) -> Option<crate::vm::LlmDebugInfo> {
        self.vm.take_llm_debug()
    }

    /// Get all function signatures (both regular and LLM functions).
    ///
    /// This provides host languages with the information needed to:
    /// - Discover available functions
    /// - Generate typed bindings
    /// - Validate arguments before calling
    ///
    /// # Example
    ///
    /// ```ignore
    /// runtime.eval(r#"
    ///     def add(a: Int, b: Int) -> Int { a + b }
    /// "#)?;
    ///
    /// let signatures = runtime.get_function_signatures();
    /// for sig in signatures {
    ///     println!("{}", sig); // "fn add(a: Int, b: Int) -> Int"
    /// }
    /// ```
    pub fn get_function_signatures(&self) -> Vec<FunctionSignature> {
        let mut signatures = Vec::new();

        // Get signatures from regular functions
        for name in &self.known_functions {
            if let Some(func) = self.vm.get_function(name) {
                signatures.push(FunctionSignature::from_compiled_function(func));
            }
        }

        // Get signatures from LLM functions
        for name in &self.known_llm_functions {
            if let Some(func) = self.vm.get_llm_function_by_name(name) {
                signatures.push(FunctionSignature::from_llm_function(func));
            }
        }

        signatures
    }

    /// Get a specific function signature by name.
    ///
    /// Returns None if the function is not found.
    pub fn get_function_signature(&self, name: &str) -> Option<FunctionSignature> {
        // Check regular functions first
        if let Some(func) = self.vm.get_function(name) {
            return Some(FunctionSignature::from_compiled_function(func));
        }

        // Then check LLM functions
        if let Some(func) = self.vm.get_llm_function_by_name(name) {
            return Some(FunctionSignature::from_llm_function(func));
        }

        None
    }

    /// Check if a function exists by name.
    pub fn has_function(&self, name: &str) -> bool {
        self.known_functions.contains(&name.to_string())
            || self.known_llm_functions.contains(&name.to_string())
    }

    /// Call a function by name with the given arguments.
    ///
    /// This is the primary FFI entry point for host languages to invoke
    /// Lattice functions. It looks up the function by name, validates
    /// argument count, executes the function, and returns the result.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function to call
    /// * `args` - The arguments to pass to the function as LatticeValues
    ///
    /// # Returns
    ///
    /// Returns the function's return value as a LatticeValue, or an error
    /// if the function doesn't exist, argument count is wrong, or execution fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Define a function
    /// runtime.eval(r#"
    ///     def add(a: Int, b: Int) -> Int {
    ///         a + b
    ///     }
    /// "#)?;
    ///
    /// // Call it
    /// let result = runtime.call("add", vec![
    ///     LatticeValue::Int(3),
    ///     LatticeValue::Int(4),
    /// ])?;
    /// assert_eq!(result, LatticeValue::Int(7));
    /// ```
    pub fn call(&mut self, name: &str, args: Vec<LatticeValue>) -> Result<LatticeValue, LatticeError> {
        // Check if it's a regular function
        if let Some(func) = self.vm.get_function(name) {
            let func = func.clone();
            let arg_count = args.len();

            // Push arguments onto the VM stack
            for arg in args {
                self.vm.push(arg.to_internal())?;
            }

            // Run the function with args already on stack
            let result = self.vm.run_function_with_args(func, arg_count)?;

            // Convert result to LatticeValue
            return LatticeValue::from_internal(&result).map_err(|e| {
                LatticeError::Runtime(format!("Failed to convert result: {}", e))
            });
        }

        // Check if it's an LLM function
        if self.known_llm_functions.contains(&name.to_string()) {
            // For LLM functions, we need to generate code that calls the function
            // and evaluate it
            let arg_strs: Vec<String> = args.iter().map(|a| format_value_as_code(a)).collect();
            let call_code = format!("{}({})", name, arg_strs.join(", "));
            return self.eval(&call_code);
        }

        Err(LatticeError::Runtime(format!("Undefined function: {}", name)))
    }

    /// Call a function by name with named arguments.
    ///
    /// This is an alternative entry point that allows passing arguments by name,
    /// which can be more convenient when the function has many parameters.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function to call
    /// * `args` - A map of parameter names to values
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Define a function
    /// runtime.eval(r#"
    ///     def greet(name: String, greeting: String) -> String {
    ///         greeting + ", " + name + "!"
    ///     }
    /// "#)?;
    ///
    /// // Call with named arguments
    /// let result = runtime.call_with_named_args("greet", vec![
    ///     ("greeting".to_string(), LatticeValue::String("Hello".to_string())),
    ///     ("name".to_string(), LatticeValue::String("World".to_string())),
    /// ])?;
    /// ```
    pub fn call_with_named_args(
        &mut self,
        name: &str,
        args: Vec<(String, LatticeValue)>,
    ) -> Result<LatticeValue, LatticeError> {
        // Get the function signature to determine parameter order
        let sig = self.get_function_signature(name).ok_or_else(|| {
            LatticeError::Runtime(format!("Undefined function: {}", name))
        })?;

        // Validate we have the right number of arguments
        if args.len() != sig.arity() {
            return Err(LatticeError::Runtime(format!(
                "Function '{}' expects {} arguments, got {}",
                name, sig.arity(), args.len()
            )));
        }

        // Create a map for quick lookup
        let arg_map: std::collections::HashMap<_, _> = args.into_iter().collect();

        // Build positional args in the order the function expects
        let mut positional_args = Vec::with_capacity(sig.arity());
        for param in &sig.params {
            let value = arg_map.get(&param.name).ok_or_else(|| {
                LatticeError::Runtime(format!(
                    "Missing argument '{}' for function '{}'",
                    param.name, name
                ))
            })?;
            positional_args.push(value.clone());
        }

        // Now call with positional args
        self.call(name, positional_args)
    }
}

/// Format a LatticeValue as Lattice source code for evaluation.
///
/// This is used internally to generate code for calling LLM functions.
fn format_value_as_code(value: &LatticeValue) -> String {
    match value {
        LatticeValue::Null => "null".to_string(),
        LatticeValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        LatticeValue::Int(i) => i.to_string(),
        LatticeValue::Float(f) => {
            // Ensure float has decimal point
            let s = f.to_string();
            if s.contains('.') || s.contains('e') || s.contains('E') {
                s
            } else {
                format!("{}.0", s)
            }
        }
        LatticeValue::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        LatticeValue::Path(p) => format!("path(\"{}\")", p.replace('\\', "\\\\").replace('"', "\\\"")),
        LatticeValue::List(items) => {
            let inner: Vec<String> = items.iter().map(format_value_as_code).collect();
            format!("[{}]", inner.join(", "))
        }
        LatticeValue::Map(pairs) => {
            let inner: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, format_value_as_code(v)))
                .collect();
            format!("{{{}}}", inner.join(", "))
        }
    }
}

/// Thread-safe wrapper for LatticeRuntime.
///
/// Use this when you need to share a runtime across multiple threads.
///
/// # Example
///
/// ```ignore
/// let runtime = LatticeRuntime::builder()
///     .without_llm()
///     .without_sql()
///     .build()?;
///
/// let shared = SharedRuntime::new(runtime);
///
/// // Can be cloned and shared across threads
/// let shared_clone = shared.clone();
/// std::thread::spawn(move || {
///     let result = shared_clone.eval("1 + 1");
/// });
/// ```
#[derive(Clone)]
pub struct SharedRuntime(Arc<Mutex<LatticeRuntime>>);

impl SharedRuntime {
    /// Create a new SharedRuntime from a LatticeRuntime.
    pub fn new(runtime: LatticeRuntime) -> Self {
        SharedRuntime(Arc::new(Mutex::new(runtime)))
    }

    /// Evaluate source code.
    ///
    /// Acquires a lock on the runtime for the duration of evaluation.
    pub fn eval(&self, source: &str) -> Result<LatticeValue, LatticeError> {
        self.0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .eval(source)
    }

    /// Evaluate source code with bindings.
    ///
    /// Acquires a lock on the runtime for the duration of evaluation.
    pub fn eval_with_bindings(
        &self,
        source: &str,
        bindings: Vec<(String, LatticeValue)>,
    ) -> Result<LatticeValue, LatticeError> {
        self.0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .eval_with_bindings(source, bindings)
    }

    /// Get a global variable value.
    pub fn get_global(&self, name: &str) -> Result<Option<LatticeValue>, LatticeError> {
        Ok(self
            .0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .get_global(name))
    }

    /// Set a global variable.
    pub fn set_global(&self, name: &str, value: LatticeValue) -> Result<(), LatticeError> {
        self.0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .set_global(name, value);
        Ok(())
    }

    /// Get all registered type schemas.
    pub fn get_types(&self) -> Result<Vec<TypeSchema>, LatticeError> {
        Ok(self
            .0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .get_types())
    }

    /// Reset the runtime.
    pub fn reset(&self) -> Result<(), LatticeError> {
        self.0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .reset();
        Ok(())
    }

    /// Get all function signatures.
    pub fn get_function_signatures(&self) -> Result<Vec<FunctionSignature>, LatticeError> {
        Ok(self
            .0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .get_function_signatures())
    }

    /// Get a specific function signature by name.
    pub fn get_function_signature(&self, name: &str) -> Result<Option<FunctionSignature>, LatticeError> {
        Ok(self
            .0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .get_function_signature(name))
    }

    /// Check if a function exists by name.
    pub fn has_function(&self, name: &str) -> Result<bool, LatticeError> {
        Ok(self
            .0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .has_function(name))
    }

    /// Call a function by name with the given arguments.
    ///
    /// Acquires a lock on the runtime for the duration of the call.
    pub fn call(&self, name: &str, args: Vec<LatticeValue>) -> Result<LatticeValue, LatticeError> {
        self.0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .call(name, args)
    }

    /// Call a function by name with named arguments.
    ///
    /// Acquires a lock on the runtime for the duration of the call.
    pub fn call_with_named_args(
        &self,
        name: &str,
        args: Vec<(String, LatticeValue)>,
    ) -> Result<LatticeValue, LatticeError> {
        self.0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .call_with_named_args(name, args)
    }

    /// Take debug info from the last LLM call, clearing it from the runtime.
    ///
    /// Acquires a lock on the runtime for the duration of the operation.
    pub fn take_llm_debug(&self) -> Result<Option<crate::vm::LlmDebugInfo>, LatticeError> {
        Ok(self
            .0
            .lock()
            .map_err(|e| LatticeError::Runtime(format!("Lock poisoned: {}", e)))?
            .take_llm_debug())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_runtime() -> LatticeRuntime {
        RuntimeBuilder::new()
            .without_llm()
            .without_sql()
            .build()
            .map(LatticeRuntime::from_built)
            .expect("Failed to create test runtime")
    }

    #[test]
    fn test_eval_simple_expression() {
        let mut runtime = create_test_runtime();
        let result = runtime.eval("1 + 2 * 3").unwrap();
        assert_eq!(result, LatticeValue::Int(7));
    }

    #[test]
    fn test_eval_string() {
        let mut runtime = create_test_runtime();
        let result = runtime.eval(r#""hello" + " world""#).unwrap();
        assert_eq!(result, LatticeValue::String("hello world".to_string()));
    }

    #[test]
    fn test_eval_list() {
        let mut runtime = create_test_runtime();
        let result = runtime.eval("[1, 2, 3]").unwrap();
        assert_eq!(
            result,
            LatticeValue::List(vec![
                LatticeValue::Int(1),
                LatticeValue::Int(2),
                LatticeValue::Int(3),
            ])
        );
    }

    #[test]
    fn test_eval_with_bindings() {
        let mut runtime = create_test_runtime();
        let result = runtime
            .eval_with_bindings(
                "x + y",
                vec![
                    ("x".to_string(), LatticeValue::Int(10)),
                    ("y".to_string(), LatticeValue::Int(20)),
                ],
            )
            .unwrap();
        assert_eq!(result, LatticeValue::Int(30));
    }

    #[test]
    fn test_global_variables() {
        let mut runtime = create_test_runtime();

        // Set a global
        runtime.set_global("count", LatticeValue::Int(42));

        // Get it back
        let value = runtime.get_global("count");
        assert_eq!(value, Some(LatticeValue::Int(42)));

        // Use it in eval
        let result = runtime.eval("count + 8").unwrap();
        assert_eq!(result, LatticeValue::Int(50));
    }

    #[test]
    fn test_global_persists_across_evals() {
        let mut runtime = create_test_runtime();

        // First eval creates global
        runtime.eval("let x = 100").unwrap();

        // Second eval uses it
        let result = runtime.eval("x * 2").unwrap();
        assert_eq!(result, LatticeValue::Int(200));
    }

    #[test]
    fn test_type_definition() {
        let mut runtime = create_test_runtime();

        // Define a type
        runtime
            .eval("type Person { name: String, age: Int }")
            .unwrap();

        // Check it's registered
        let schema = runtime.get_type("Person");
        assert!(schema.is_some());

        match schema.unwrap() {
            TypeSchema::Struct(s) => {
                assert_eq!(s.name, "Person");
                assert_eq!(s.fields.len(), 2);
            }
            _ => panic!("Expected Struct schema"),
        }
    }

    #[test]
    fn test_enum_definition() {
        let mut runtime = create_test_runtime();

        runtime.eval("enum Color { Red, Green, Blue }").unwrap();

        let schema = runtime.get_type("Color");
        assert!(schema.is_some());

        match schema.unwrap() {
            TypeSchema::Enum(e) => {
                assert_eq!(e.name, "Color");
                assert_eq!(e.variants, vec!["Red", "Green", "Blue"]);
            }
            _ => panic!("Expected Enum schema"),
        }
    }

    #[test]
    fn test_get_types() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            type Person { name: String }
            enum Status { Active, Inactive }
        "#,
            )
            .unwrap();

        let types = runtime.get_types();
        assert_eq!(types.len(), 2);
    }

    #[test]
    fn test_register_type() {
        let mut runtime = create_test_runtime();

        let schema = TypeSchema::Struct(StructSchema {
            name: "ExternalType".to_string(),
            fields: vec![FieldSchema {
                name: "value".to_string(),
                type_schema: TypeSchema::Int,
                optional: false,
                description: None,
            }],
            description: None,
        });

        runtime.register_type(schema).unwrap();

        // Verify it's registered
        let retrieved = runtime.get_type("ExternalType");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_reset() {
        let mut runtime = create_test_runtime();

        // Set up some state
        runtime.set_global("x", LatticeValue::Int(42));
        runtime.eval("type Foo { bar: String }").unwrap();

        // Reset
        runtime.reset();

        // State should be cleared
        assert!(runtime.get_global("x").is_none());
        assert!(runtime.get_type("Foo").is_none());
    }

    #[test]
    fn test_function_definition() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def add(a: Int, b: Int) -> Int {
                a + b
            }
        "#,
            )
            .unwrap();

        // Function should be registered
        assert!(runtime.function_names().contains(&"add".to_string()));

        // Should be callable in subsequent eval
        let result = runtime.eval("add(3, 4)").unwrap();
        assert_eq!(result, LatticeValue::Int(7));
    }

    #[test]
    fn test_runtime_isolation() {
        // Create two separate runtimes
        let mut runtime1 = create_test_runtime();
        let mut runtime2 = create_test_runtime();

        // Set different values in each
        runtime1.set_global("x", LatticeValue::Int(100));
        runtime2.set_global("x", LatticeValue::Int(200));

        // They should be independent
        assert_eq!(runtime1.get_global("x"), Some(LatticeValue::Int(100)));
        assert_eq!(runtime2.get_global("x"), Some(LatticeValue::Int(200)));
    }

    #[test]
    fn test_runtime_type_isolation() {
        // Create two separate runtimes
        let mut rt1 = create_test_runtime();
        let mut rt2 = create_test_runtime();

        // Define type in rt1
        rt1.eval("type Foo { x: Int }").unwrap();

        // Verify rt1 has the type registered
        assert!(rt1.get_type("Foo").is_some(), "rt1 should have Foo type");

        // rt2 should NOT have the type in its registry
        assert!(rt2.get_type("Foo").is_none(), "rt2 should not see Foo type defined in rt1");

        // Define a different type with same name in rt2
        rt2.eval("type Bar { y: String }").unwrap();

        // Each runtime should have only its own type
        assert!(rt1.get_type("Foo").is_some());
        assert!(rt1.get_type("Bar").is_none());
        assert!(rt2.get_type("Bar").is_some());
        assert!(rt2.get_type("Foo").is_none());
    }

    #[test]
    fn test_runtime_global_isolation() {
        // Create two separate runtimes
        let mut rt1 = create_test_runtime();
        let rt2 = create_test_runtime();

        // Define variable in rt1
        rt1.eval("let shared = 42").unwrap();

        // rt2 should NOT see shared
        assert!(rt2.get_global("shared").is_none());

        // Verify rt1 still has it
        assert_eq!(rt1.get_global("shared"), Some(LatticeValue::Int(42)));
    }

    #[test]
    fn test_runtime_function_isolation() {
        // Create two separate runtimes
        let mut rt1 = create_test_runtime();
        let mut rt2 = create_test_runtime();

        // Define function in rt1
        rt1.eval("def add_ten(x: Int) -> Int { x + 10 }").unwrap();

        // rt2 should NOT see add_ten
        let result = rt2.eval("add_ten(5)");
        assert!(result.is_err(), "rt2 should not see add_ten defined in rt1");

        // rt1 should still work
        let val = rt1.eval("add_ten(5)").unwrap();
        assert_eq!(val, LatticeValue::Int(15));
    }

    #[test]
    fn test_runtime_enum_isolation() {
        let mut rt1 = create_test_runtime();
        let mut rt2 = create_test_runtime();

        // Define enum in rt1
        rt1.eval("enum Status { Active, Inactive }").unwrap();

        // Verify rt1 has the enum registered
        assert!(rt1.get_type("Status").is_some(), "rt1 should have Status enum");

        // rt2 should NOT have the enum in its registry
        assert!(rt2.get_type("Status").is_none(), "rt2 should not see Status defined in rt1");

        // Define a different enum in rt2
        rt2.eval("enum Priority { High, Low }").unwrap();

        // Each runtime should have only its own enum
        assert!(rt1.get_type("Status").is_some());
        assert!(rt1.get_type("Priority").is_none());
        assert!(rt2.get_type("Priority").is_some());
        assert!(rt2.get_type("Status").is_none());
    }

    #[test]
    fn test_shared_runtime() {
        let runtime = create_test_runtime();
        let shared = SharedRuntime::new(runtime);

        // Should be clonable
        let shared2 = shared.clone();

        // Both should work
        let result1 = shared.eval("1 + 1").unwrap();
        let result2 = shared2.eval("2 + 2").unwrap();

        assert_eq!(result1, LatticeValue::Int(2));
        assert_eq!(result2, LatticeValue::Int(4));
    }

    #[test]
    fn test_eval_error() {
        let mut runtime = create_test_runtime();

        // Syntax error
        let result = runtime.eval("1 +");
        assert!(result.is_err());

        // Undefined variable
        let result = runtime.eval("undefined_var");
        assert!(result.is_err());
    }

    #[test]
    fn test_global_names() {
        let mut runtime = create_test_runtime();

        runtime.set_global("foo", LatticeValue::Int(1));
        runtime.set_global("bar", LatticeValue::Int(2));

        let names = runtime.global_names();
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"bar".to_string()));
    }

    #[test]
    fn test_get_function_signatures_regular() {
        let mut runtime = create_test_runtime();

        // Define a function
        runtime
            .eval(
                r#"
            def add(a: Int, b: Int) -> Int {
                a + b
            }
        "#,
            )
            .unwrap();

        let signatures = runtime.get_function_signatures();
        assert_eq!(signatures.len(), 1);

        let sig = &signatures[0];
        assert_eq!(sig.name, "add");
        assert_eq!(sig.arity(), 2);
        assert!(!sig.is_llm);
        assert!(!sig.is_async);
    }

    #[test]
    fn test_get_function_signature_by_name() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def greet(name: String) -> String {
                "Hello, " + name
            }

            def square(n: Int) -> Int {
                n * n
            }
        "#,
            )
            .unwrap();

        // Get specific signature
        let sig = runtime.get_function_signature("greet");
        assert!(sig.is_some());
        assert_eq!(sig.unwrap().name, "greet");

        // Get non-existent
        let sig = runtime.get_function_signature("nonexistent");
        assert!(sig.is_none());
    }

    #[test]
    fn test_has_function() {
        let mut runtime = create_test_runtime();

        runtime
            .eval("def my_func() -> Int { 42 }")
            .unwrap();

        assert!(runtime.has_function("my_func"));
        assert!(!runtime.has_function("other_func"));
    }

    #[test]
    fn test_function_signatures_multiple() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def add(a: Int, b: Int) -> Int { a + b }
            def sub(a: Int, b: Int) -> Int { a - b }
            def mul(a: Int, b: Int) -> Int { a * b }
        "#,
            )
            .unwrap();

        let signatures = runtime.get_function_signatures();
        assert_eq!(signatures.len(), 3);

        let names: Vec<&str> = signatures.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"add"));
        assert!(names.contains(&"sub"));
        assert!(names.contains(&"mul"));
    }

    #[test]
    fn test_function_signature_display() {
        let sig = FunctionSignature::new(
            "calculate".to_string(),
            vec![
                ParameterSchema {
                    name: "x".to_string(),
                    type_schema: TypeSchema::Int,
                },
                ParameterSchema {
                    name: "y".to_string(),
                    type_schema: TypeSchema::Int,
                },
            ],
            TypeSchema::Int,
            false,
        );

        assert_eq!(sig.to_string(), "fn calculate(x: Int, y: Int) -> Int");
    }

    // ============================================================
    // Tests for runtime.call()
    // ============================================================

    #[test]
    fn test_call_simple_function() {
        let mut runtime = create_test_runtime();

        // Define a simple function
        runtime
            .eval(
                r#"
            def add(a: Int, b: Int) -> Int {
                a + b
            }
        "#,
            )
            .unwrap();

        // Call it
        let result = runtime
            .call("add", vec![LatticeValue::Int(3), LatticeValue::Int(4)])
            .unwrap();
        assert_eq!(result, LatticeValue::Int(7));
    }

    #[test]
    fn test_call_no_args() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def get_answer() -> Int {
                42
            }
        "#,
            )
            .unwrap();

        let result = runtime.call("get_answer", vec![]).unwrap();
        assert_eq!(result, LatticeValue::Int(42));
    }

    #[test]
    fn test_call_with_string() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def greet(name: String) -> String {
                "Hello, " + name + "!"
            }
        "#,
            )
            .unwrap();

        let result = runtime
            .call("greet", vec![LatticeValue::String("World".to_string())])
            .unwrap();
        assert_eq!(result, LatticeValue::String("Hello, World!".to_string()));
    }

    #[test]
    fn test_call_with_list() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def sum_list(nums: [Int]) -> Int {
                let total = 0
                for n in nums {
                    total = total + n
                }
                total
            }
        "#,
            )
            .unwrap();

        let result = runtime
            .call(
                "sum_list",
                vec![LatticeValue::List(vec![
                    LatticeValue::Int(1),
                    LatticeValue::Int(2),
                    LatticeValue::Int(3),
                ])],
            )
            .unwrap();
        assert_eq!(result, LatticeValue::Int(6));
    }

    #[test]
    fn test_call_wrong_arity() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def add(a: Int, b: Int) -> Int {
                a + b
            }
        "#,
            )
            .unwrap();

        // Too few arguments
        let result = runtime.call("add", vec![LatticeValue::Int(1)]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("expects 2 arguments, got 1"));

        // Too many arguments
        let result = runtime.call(
            "add",
            vec![
                LatticeValue::Int(1),
                LatticeValue::Int(2),
                LatticeValue::Int(3),
            ],
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("expects 2 arguments, got 3"));
    }

    #[test]
    fn test_call_undefined_function() {
        let mut runtime = create_test_runtime();

        let result = runtime.call("nonexistent", vec![]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Undefined function"));
    }

    #[test]
    fn test_call_multiple_times() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def multiply(a: Int, b: Int) -> Int {
                a * b
            }
        "#,
            )
            .unwrap();

        // Call multiple times
        let r1 = runtime
            .call("multiply", vec![LatticeValue::Int(2), LatticeValue::Int(3)])
            .unwrap();
        let r2 = runtime
            .call("multiply", vec![LatticeValue::Int(4), LatticeValue::Int(5)])
            .unwrap();
        let r3 = runtime
            .call("multiply", vec![LatticeValue::Int(6), LatticeValue::Int(7)])
            .unwrap();

        assert_eq!(r1, LatticeValue::Int(6));
        assert_eq!(r2, LatticeValue::Int(20));
        assert_eq!(r3, LatticeValue::Int(42));
    }

    #[test]
    fn test_call_with_global_state() {
        let mut runtime = create_test_runtime();

        // Set a global variable
        runtime.set_global("multiplier", LatticeValue::Int(10));

        runtime
            .eval(
                r#"
            def scale(x: Int) -> Int {
                x * multiplier
            }
        "#,
            )
            .unwrap();

        let result = runtime
            .call("scale", vec![LatticeValue::Int(5)])
            .unwrap();
        assert_eq!(result, LatticeValue::Int(50));
    }

    #[test]
    fn test_call_function_calling_function() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def double(x: Int) -> Int {
                x * 2
            }

            def quadruple(x: Int) -> Int {
                double(double(x))
            }
        "#,
            )
            .unwrap();

        let result = runtime
            .call("quadruple", vec![LatticeValue::Int(5)])
            .unwrap();
        assert_eq!(result, LatticeValue::Int(20));
    }

    #[test]
    fn test_call_with_named_args() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def greet(greeting: String, name: String) -> String {
                greeting + ", " + name + "!"
            }
        "#,
            )
            .unwrap();

        // Note: Currently, compiled functions only store arity, not parameter names.
        // The signature generates placeholder names (arg0, arg1), so named args
        // must use those placeholder names. This is a known limitation.
        let result = runtime
            .call_with_named_args(
                "greet",
                vec![
                    ("arg0".to_string(), LatticeValue::String("Hello".to_string())),
                    ("arg1".to_string(), LatticeValue::String("World".to_string())),
                ],
            )
            .unwrap();
        assert_eq!(result, LatticeValue::String("Hello, World!".to_string()));
    }

    #[test]
    fn test_call_with_named_args_missing() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def greet(greeting: String, name: String) -> String {
                greeting + ", " + name
            }
        "#,
            )
            .unwrap();

        // Missing argument (using placeholder names since CompiledFunction doesn't store real names)
        let result = runtime.call_with_named_args(
            "greet",
            vec![("arg0".to_string(), LatticeValue::String("Hello".to_string()))],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_call_returns_list() {
        let mut runtime = create_test_runtime();

        runtime
            .eval(
                r#"
            def get_nums() -> [Int] {
                [1, 2, 3]
            }
        "#,
            )
            .unwrap();

        let result = runtime
            .call("get_nums", vec![])
            .unwrap();
        assert_eq!(
            result,
            LatticeValue::List(vec![
                LatticeValue::Int(1),
                LatticeValue::Int(2),
                LatticeValue::Int(3),
            ])
        );
    }

    #[test]
    fn test_call_shared_runtime() {
        let runtime = create_test_runtime();
        let shared = SharedRuntime::new(runtime);

        // Define function
        shared.eval(r#"def square(x: Int) -> Int { x * x }"#).unwrap();

        // Call from shared runtime
        let result = shared
            .call("square", vec![LatticeValue::Int(5)])
            .unwrap();
        assert_eq!(result, LatticeValue::Int(25));
    }

    #[test]
    fn test_format_value_as_code() {
        assert_eq!(format_value_as_code(&LatticeValue::Null), "null");
        assert_eq!(format_value_as_code(&LatticeValue::Bool(true)), "true");
        assert_eq!(format_value_as_code(&LatticeValue::Bool(false)), "false");
        assert_eq!(format_value_as_code(&LatticeValue::Int(42)), "42");
        assert_eq!(format_value_as_code(&LatticeValue::Float(3.14)), "3.14");
        assert_eq!(
            format_value_as_code(&LatticeValue::String("hello".to_string())),
            "\"hello\""
        );
        assert_eq!(
            format_value_as_code(&LatticeValue::String("say \"hi\"".to_string())),
            "\"say \\\"hi\\\"\""
        );
        assert_eq!(
            format_value_as_code(&LatticeValue::List(vec![
                LatticeValue::Int(1),
                LatticeValue::Int(2),
            ])),
            "[1, 2]"
        );
        assert_eq!(
            format_value_as_code(&LatticeValue::Map(vec![
                ("a".to_string(), LatticeValue::Int(1)),
            ])),
            "{\"a\": 1}"
        );
    }
}
