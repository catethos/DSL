use dsl_types::{Class, Enum};
use std::collections::HashMap;

/// Registry for user-defined types
#[derive(Clone)]
pub struct TypeRegistry {
    classes: HashMap<String, Class>,
    enums: HashMap<String, Enum>,
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    /// Register a new class (struct) type
    pub fn register_class(&mut self, class: Class) {
        self.classes.insert(class.name.clone(), class);
    }

    /// Register a new enum type
    pub fn register_enum(&mut self, enum_def: Enum) {
        self.enums.insert(enum_def.name.clone(), enum_def);
    }

    /// Get a class by name
    pub fn get_class(&self, name: &str) -> Option<&Class> {
        self.classes.get(name)
    }

    /// Get an enum by name
    pub fn get_enum(&self, name: &str) -> Option<&Enum> {
        self.enums.get(name)
    }

    /// Check if a type exists
    pub fn has_type(&self, name: &str) -> bool {
        self.classes.contains_key(name) || self.enums.contains_key(name)
    }

    /// Get all registered classes
    pub fn all_classes(&self) -> Vec<&Class> {
        self.classes.values().collect()
    }

    /// Get all registered enums
    pub fn all_enums(&self) -> Vec<&Enum> {
        self.enums.values().collect()
    }

    /// Get all registered classes as (name, class) pairs
    pub fn all(&self) -> Vec<(&str, &Class)> {
        self.classes.iter().map(|(k, v)| (k.as_str(), v)).collect()
    }

    /// Get all classes and enums for building IR
    pub fn to_ir_types(&self) -> (Vec<Class>, Vec<Enum>) {
        (
            self.classes.values().cloned().collect(),
            self.enums.values().cloned().collect(),
        )
    }
}
