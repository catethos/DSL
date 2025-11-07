pub mod ir;
pub mod serde_impl;
pub mod sql;
pub mod type_registry;
pub mod types;
pub mod value;

// Re-export main types
pub use ir::{
    IRAgent, IRBinding, IRContextStore, IRExecution, IRFunction, IRFunctionClause,
    IRFunctionGroup, IRMatchCase, IRMessageHandler, IRNode, IRPattern, IRProperty,
    IRTemplateSegment, IR,
};
pub use sql::SQLExecutor;
pub use type_registry::TypeRegistry;
pub use types::{Class, Enum, Field, FieldType};
pub use value::Value;
