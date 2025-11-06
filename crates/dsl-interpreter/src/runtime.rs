use crate::type_registry::TypeRegistry;
use dsl_ir::{Value, IRFunction};
use std::collections::HashMap;

pub struct Runtime {
    pub vars: HashMap<String, Value>,
    pub types: TypeRegistry,
    pub functions: HashMap<String, IRFunction>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            types: TypeRegistry::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get_var(&self, name: &str) -> Result<Value, String> {
        self.vars
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Variable '{}' not found", name))
    }

    pub fn set_var(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
