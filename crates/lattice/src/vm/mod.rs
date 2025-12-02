//! Virtual machine module
//!
//! Stack-based bytecode interpreter with persistent state across cells.

pub mod bytecode;
pub mod machine;

pub use bytecode::{Chunk, CompiledFunction, LlmFunction, OpCode};
pub use machine::{CallFrame, LlmDebugInfo, VM};
