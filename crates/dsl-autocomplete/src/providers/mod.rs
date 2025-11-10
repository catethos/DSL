//! Built-in completion providers

mod command;
pub mod dynamic;
mod keyword;

pub use command::CommandProvider;
pub use dynamic::{FunctionProvider, TypeProvider, VariableProvider};
pub use keyword::KeywordProvider;
