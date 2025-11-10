pub mod builtins;
pub mod builtins_metadata;
pub mod error;
pub mod interpreter;
pub mod pattern;
pub mod runtime;
pub mod sql;
pub mod tracing;

pub use builtins::BuiltinFunctions;
pub use builtins_metadata::{all_builtins, builtins_for_autocomplete, BuiltinFunctionInfo};
pub use dsl_ir::TypeRegistry;
pub use error::InterpreterError;
pub use interpreter::Interpreter;
pub use pattern::PatternMatcher;
pub use runtime::Runtime;
pub use sql::SQLExecutor;
pub use tracing::{TraceCollector, TraceConfig, TraceEvent, TracingInterpreter};
