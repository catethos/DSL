pub mod ir;
pub mod lowering;
pub mod serde_impl;
pub mod type_registry;
pub mod types;
pub mod value;

// Re-export main types
pub use ir::{
    IRAgent, IRBinding, IRContextStore, IRExecution, IRFunction, IRFunctionClause, IRFunctionGroup,
    IRMatchCase, IRMessageHandler, IRNode, IRPattern, IRProperty, IRTemplateSegment, IR,
    EffectKind, Span,
};
pub use lowering::{Lowering, DebugInfoTable, DebugInfo, NodeId};
pub use type_registry::TypeRegistry;
pub use types::{Class, Enum, Field, FieldType};
pub use value::Value;
