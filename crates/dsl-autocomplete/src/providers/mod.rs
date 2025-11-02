//! Built-in completion providers

mod keyword;
mod command;
pub mod dynamic;

pub use keyword::KeywordProvider;
pub use command::CommandProvider;
pub use dynamic::{FunctionProvider, VariableProvider, TypeProvider};
