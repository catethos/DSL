use dsl_ir::{IRFunction, IRFunctionGroup, TypeRegistry, Value};
use std::collections::HashMap;
use crate::error::InterpreterError;

pub struct Runtime {
    pub scopes: Vec<HashMap<String, Value>>,
    pub types: TypeRegistry,
    pub functions: HashMap<String, IRFunction>,
    pub function_groups: HashMap<String, IRFunctionGroup>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Initialize with global scope
            types: TypeRegistry::new(),
            functions: HashMap::new(),
            function_groups: HashMap::new(),
        }
    }

    /// Push a new scope (for function calls, blocks, etc.)
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope (when exiting function/block)
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        } else {
            // Safety: never pop global scope - this indicates a bug in the interpreter
            panic!(
                "Internal error: attempted to pop global scope. \
                This is a bug in the interpreter - scope stack is unbalanced. \
                Please report this issue."
            );
        }
    }

    /// Get variable - search from innermost to outermost scope
    pub fn get_var(&self, name: &str) -> Result<Value, InterpreterError> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        Err(InterpreterError::UnknownVariable {
            name: name.to_string(),
            source_span: None,
        })
    }

    /// Set variable in current scope
    pub fn set_var(&mut self, name: String, value: Value) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, value);
        }
    }

    /// Set variable in global scope (for REPL persistence)
    pub fn set_global_var(&mut self, name: String, value: Value) {
        if let Some(global_scope) = self.scopes.first_mut() {
            global_scope.insert(name, value);
        }
    }

    /// Get all variables in current scope (for debugging)
    pub fn current_scope_vars(&self) -> Option<&HashMap<String, Value>> {
        self.scopes.last()
    }

    /// Get all variables including parent scopes (for debugging)
    pub fn all_visible_vars(&self) -> HashMap<String, Value> {
        let mut result = HashMap::new();
        // Iterate from outermost to innermost so inner scopes shadow outer
        for scope in &self.scopes {
            result.extend(scope.clone());
        }
        result
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
