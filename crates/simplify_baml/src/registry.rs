/// Schema Registry for collecting and building IR from types
///
/// This module provides a builder API for registering types that implement
/// the `BamlSchema` trait and automatically building the complete IR.
use crate::ir::{BamlSchema, Class, Enum, IR};
use std::collections::HashSet;

/// Registry for collecting schema definitions and building IR
///
/// # Example
/// ```ignore
/// use simplify_baml::*;
///
/// #[derive(BamlSchema)]
/// struct Person {
///     name: String,
///     age: i64,
/// }
///
/// let ir = BamlSchemaRegistry::new()
///     .register::<Person>()
///     .build();
/// ```
pub struct BamlSchemaRegistry {
    classes: Vec<Class>,
    enums: Vec<Enum>,
    registered_types: HashSet<String>,
}

impl BamlSchemaRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
            enums: Vec::new(),
            registered_types: HashSet::new(),
        }
    }

    /// Register a type that implements BamlSchema
    ///
    /// This will automatically register the type and all its dependencies
    /// into the registry.
    pub fn register<T: BamlSchema>(mut self) -> Self {
        let type_name = T::schema_name();

        // Avoid duplicate registration
        if self.registered_types.contains(type_name) {
            return self;
        }

        self.registered_types.insert(type_name.to_string());
        T::register_schemas(&mut self);
        self
    }

    /// Add a class to the registry (called by the derive macro)
    pub fn add_class(&mut self, class: Class) {
        // Check if already registered
        if !self.classes.iter().any(|c| c.name == class.name) {
            self.classes.push(class);
        }
    }

    /// Add an enum to the registry (called by the derive macro)
    pub fn add_enum(&mut self, enum_def: Enum) {
        // Check if already registered
        if !self.enums.iter().any(|e| e.name == enum_def.name) {
            self.enums.push(enum_def);
        }
    }

    /// Build the final IR from all registered types
    pub fn build(self) -> IR {
        IR {
            classes: self.classes,
            enums: self.enums,
            functions: Vec::new(),
        }
    }

    /// Build IR and include additional functions
    pub fn build_with_functions(self, functions: Vec<crate::ir::Function>) -> IR {
        IR {
            classes: self.classes,
            enums: self.enums,
            functions,
        }
    }
}

impl Default for BamlSchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = BamlSchemaRegistry::new();
        let ir = registry.build();
        assert_eq!(ir.classes.len(), 0);
        assert_eq!(ir.enums.len(), 0);
    }

    #[test]
    fn test_manual_class_addition() {
        let mut registry = BamlSchemaRegistry::new();
        registry.add_class(Class {
            name: "TestClass".to_string(),
            description: None,
            fields: vec![],
        });

        let ir = registry.build();
        assert_eq!(ir.classes.len(), 1);
        assert_eq!(ir.classes[0].name, "TestClass");
    }

    #[test]
    fn test_duplicate_prevention() {
        let mut registry = BamlSchemaRegistry::new();

        // Add same class twice
        registry.add_class(Class {
            name: "TestClass".to_string(),
            description: None,
            fields: vec![],
        });

        registry.add_class(Class {
            name: "TestClass".to_string(),
            description: None,
            fields: vec![],
        });

        let ir = registry.build();
        assert_eq!(ir.classes.len(), 1, "Should only have one class despite duplicate addition");
    }
}
