//! DSL egui - Desktop GUI with egui
//!
//! This crate provides a desktop GUI for the DSL language using egui.
//! It includes:
//! - Workspace mode with resizable Editor and REPL panes
//! - Syntax highlighting with Tree-sitter
//! - Text editor integration
//! - Interactive REPL with history
//! - Rich output rendering (tables, trees, images, etc.)

pub mod animations;
pub mod app;
pub mod autocomplete;
pub mod editor;
pub mod output_item;
pub mod renderers;
pub mod repl;
pub mod syntax;
pub mod theme;

pub use app::DslApp;
