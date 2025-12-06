//! FFI-safe function signatures for exported Lattice functions.
//!
//! This module provides `FunctionSignature`, an FFI-friendly representation of
//! function signatures that enables host languages to:
//! - Discover callable functions and their types
//! - Generate typed bindings/wrappers
//! - Validate arguments before calling

use serde::{Deserialize, Serialize};

use super::schema::TypeSchema;
use crate::types::FieldType;
use crate::vm::{CompiledFunction, LlmFunction};

/// FFI-safe representation of a function parameter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub type_schema: TypeSchema,
}

/// FFI-safe representation of a function signature.
///
/// This provides all the information host languages need to:
/// - Generate typed function wrappers
/// - Validate arguments before calling
/// - Display function documentation
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Function parameters with their types
    pub params: Vec<ParameterSchema>,
    /// Return type
    pub return_type: TypeSchema,
    /// Whether this is an LLM function (inherently async)
    pub is_llm: bool,
    /// Whether the function is async
    /// Currently, only LLM functions are async
    pub is_async: bool,
}

impl FunctionSignature {
    /// Create a new function signature.
    pub fn new(
        name: String,
        params: Vec<ParameterSchema>,
        return_type: TypeSchema,
        is_llm: bool,
    ) -> Self {
        Self {
            name,
            params,
            return_type,
            is_llm,
            // LLM functions are inherently async
            is_async: is_llm,
        }
    }

    /// Create a signature from an LlmFunction.
    pub fn from_llm_function(func: &LlmFunction) -> Self {
        let params = func
            .parameters
            .iter()
            .map(|(name, field_type)| ParameterSchema {
                name: name.clone(),
                type_schema: TypeSchema::from_field_type(field_type),
            })
            .collect();

        let return_type = TypeSchema::from_field_type(&func.return_type);

        Self::new(func.name.clone(), params, return_type, true)
    }

    /// Create a signature from a CompiledFunction.
    ///
    /// Note: CompiledFunction currently stores limited type information.
    /// Parameters are typed as Any and return type as Any.
    /// For full type info, use the extended signature metadata stored separately.
    pub fn from_compiled_function(func: &CompiledFunction) -> Self {
        // CompiledFunction only has arity, not parameter names/types
        // Generate placeholder parameter names
        let params = (0..func.arity)
            .map(|i| ParameterSchema {
                name: format!("arg{}", i),
                type_schema: TypeSchema::Any,
            })
            .collect();

        Self::new(func.name.clone(), params, TypeSchema::Any, false)
    }

    /// Create a signature from a CompiledFunction with type metadata.
    pub fn from_compiled_function_with_types(
        func: &CompiledFunction,
        param_types: &[(String, FieldType)],
        return_type: &FieldType,
    ) -> Self {
        let params = param_types
            .iter()
            .map(|(name, field_type)| ParameterSchema {
                name: name.clone(),
                type_schema: TypeSchema::from_field_type(field_type),
            })
            .collect();

        Self::new(
            func.name.clone(),
            params,
            TypeSchema::from_field_type(return_type),
            false,
        )
    }

    /// Get the number of parameters.
    pub fn arity(&self) -> usize {
        self.params.len()
    }

    /// Check if a parameter exists by name.
    pub fn has_param(&self, name: &str) -> bool {
        self.params.iter().any(|p| p.name == name)
    }

    /// Get a parameter by name.
    pub fn get_param(&self, name: &str) -> Option<&ParameterSchema> {
        self.params.iter().find(|p| p.name == name)
    }

    /// Get a parameter by index.
    pub fn get_param_by_index(&self, index: usize) -> Option<&ParameterSchema> {
        self.params.get(index)
    }
}

impl std::fmt::Display for FunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params_str = self
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.type_schema))
            .collect::<Vec<_>>()
            .join(", ");

        let async_prefix = if self.is_async { "async " } else { "" };
        let llm_suffix = if self.is_llm { " [LLM]" } else { "" };

        write!(
            f,
            "{}fn {}({}) -> {}{}",
            async_prefix, self.name, params_str, self.return_type, llm_suffix
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::Chunk;

    #[test]
    fn test_function_signature_new() {
        let sig = FunctionSignature::new(
            "add".to_string(),
            vec![
                ParameterSchema {
                    name: "a".to_string(),
                    type_schema: TypeSchema::Int,
                },
                ParameterSchema {
                    name: "b".to_string(),
                    type_schema: TypeSchema::Int,
                },
            ],
            TypeSchema::Int,
            false,
        );

        assert_eq!(sig.name, "add");
        assert_eq!(sig.arity(), 2);
        assert!(!sig.is_llm);
        assert!(!sig.is_async);
        assert_eq!(sig.return_type, TypeSchema::Int);
    }

    #[test]
    fn test_llm_function_is_async() {
        let sig = FunctionSignature::new(
            "analyze".to_string(),
            vec![ParameterSchema {
                name: "text".to_string(),
                type_schema: TypeSchema::String,
            }],
            TypeSchema::String,
            true,
        );

        assert!(sig.is_llm);
        assert!(sig.is_async);
    }

    #[test]
    fn test_from_compiled_function() {
        let func = CompiledFunction {
            name: "test_fn".to_string(),
            arity: 2,
            local_count: 2,
            chunk: Chunk::new(),
        };

        let sig = FunctionSignature::from_compiled_function(&func);

        assert_eq!(sig.name, "test_fn");
        assert_eq!(sig.arity(), 2);
        assert!(!sig.is_llm);
        assert!(!sig.is_async);
        // Without type info, params are named arg0, arg1 with Any type
        assert_eq!(sig.params[0].name, "arg0");
        assert_eq!(sig.params[0].type_schema, TypeSchema::Any);
    }

    #[test]
    fn test_from_compiled_function_with_types() {
        let func = CompiledFunction {
            name: "greet".to_string(),
            arity: 1,
            local_count: 1,
            chunk: Chunk::new(),
        };

        let sig = FunctionSignature::from_compiled_function_with_types(
            &func,
            &[("name".to_string(), FieldType::String)],
            &FieldType::String,
        );

        assert_eq!(sig.name, "greet");
        assert_eq!(sig.params[0].name, "name");
        assert_eq!(sig.params[0].type_schema, TypeSchema::String);
        assert_eq!(sig.return_type, TypeSchema::String);
    }

    #[test]
    fn test_from_llm_function() {
        let llm_func = LlmFunction::new(
            "summarize".to_string(),
            "https://api.example.com".to_string(),
            "gpt-4".to_string(),
            "API_KEY".to_string(),
            "Summarize: ${text}".to_string(),
            FieldType::String,
            vec![("text".to_string(), FieldType::String)],
        );

        let sig = FunctionSignature::from_llm_function(&llm_func);

        assert_eq!(sig.name, "summarize");
        assert_eq!(sig.params.len(), 1);
        assert_eq!(sig.params[0].name, "text");
        assert_eq!(sig.params[0].type_schema, TypeSchema::String);
        assert_eq!(sig.return_type, TypeSchema::String);
        assert!(sig.is_llm);
        assert!(sig.is_async);
    }

    #[test]
    fn test_display() {
        let sig = FunctionSignature::new(
            "add".to_string(),
            vec![
                ParameterSchema {
                    name: "a".to_string(),
                    type_schema: TypeSchema::Int,
                },
                ParameterSchema {
                    name: "b".to_string(),
                    type_schema: TypeSchema::Int,
                },
            ],
            TypeSchema::Int,
            false,
        );

        assert_eq!(sig.to_string(), "fn add(a: Int, b: Int) -> Int");
    }

    #[test]
    fn test_display_llm() {
        let sig = FunctionSignature::new(
            "analyze".to_string(),
            vec![ParameterSchema {
                name: "text".to_string(),
                type_schema: TypeSchema::String,
            }],
            TypeSchema::String,
            true,
        );

        assert_eq!(
            sig.to_string(),
            "async fn analyze(text: String) -> String [LLM]"
        );
    }

    #[test]
    fn test_json_serialization() {
        let sig = FunctionSignature::new(
            "add".to_string(),
            vec![
                ParameterSchema {
                    name: "a".to_string(),
                    type_schema: TypeSchema::Int,
                },
                ParameterSchema {
                    name: "b".to_string(),
                    type_schema: TypeSchema::Int,
                },
            ],
            TypeSchema::Int,
            false,
        );

        let json = serde_json::to_string(&sig).expect("Should serialize");
        let back: FunctionSignature = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(sig, back);
    }

    #[test]
    fn test_has_param() {
        let sig = FunctionSignature::new(
            "test".to_string(),
            vec![
                ParameterSchema {
                    name: "foo".to_string(),
                    type_schema: TypeSchema::Int,
                },
                ParameterSchema {
                    name: "bar".to_string(),
                    type_schema: TypeSchema::String,
                },
            ],
            TypeSchema::Bool,
            false,
        );

        assert!(sig.has_param("foo"));
        assert!(sig.has_param("bar"));
        assert!(!sig.has_param("baz"));
    }

    #[test]
    fn test_get_param() {
        let sig = FunctionSignature::new(
            "test".to_string(),
            vec![ParameterSchema {
                name: "x".to_string(),
                type_schema: TypeSchema::Float,
            }],
            TypeSchema::Float,
            false,
        );

        let param = sig.get_param("x").unwrap();
        assert_eq!(param.name, "x");
        assert_eq!(param.type_schema, TypeSchema::Float);

        assert!(sig.get_param("y").is_none());
    }

    #[test]
    fn test_get_param_by_index() {
        let sig = FunctionSignature::new(
            "test".to_string(),
            vec![
                ParameterSchema {
                    name: "first".to_string(),
                    type_schema: TypeSchema::Int,
                },
                ParameterSchema {
                    name: "second".to_string(),
                    type_schema: TypeSchema::String,
                },
            ],
            TypeSchema::Bool,
            false,
        );

        assert_eq!(sig.get_param_by_index(0).unwrap().name, "first");
        assert_eq!(sig.get_param_by_index(1).unwrap().name, "second");
        assert!(sig.get_param_by_index(2).is_none());
    }
}
