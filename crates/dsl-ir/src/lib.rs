pub mod ir;
pub mod serde_impl;
pub mod types;
pub mod value;

// Re-export main types
pub use ir::{
    IRAgent, IRBinding, IRExecution, IRFunction, IRMessageHandler, IRNode, IRProperty,
    IRTemplateSegment, IR,
};
pub use types::{Class, Enum, Field, FieldType};
pub use value::Value;
