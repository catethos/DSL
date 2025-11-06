pub mod rust_ast;
pub mod types;
pub mod expressions;
pub mod functions;
pub mod builtins;
pub mod program;

pub use rust_ast::*;

use anyhow::Result;
use dsl_ir::IR;

pub fn generate_executable(ir: &IR) -> Result<String> {
    program::generate_executable(ir)
}

pub fn generate_library(ir: &IR) -> Result<String> {
    program::generate_library(ir)
}
