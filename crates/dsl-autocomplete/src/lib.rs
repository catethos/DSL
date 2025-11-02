//! A modular, trait-based autocomplete system for DSL
//!
//! This crate provides a framework-agnostic autocomplete engine that can be
//! integrated with any UI framework. It uses a pluggable provider system
//! to generate suggestions and supports multiple matching algorithms.

mod context;
mod engine;
mod matcher;
mod provider;
mod suggestion;

pub mod providers;

#[cfg(feature = "fuzzy")]
pub mod matchers;

// Public exports
pub use context::{CompletionContext, ContextKind};
pub use engine::{AutocompleteEngine, CompletionResult};
pub use matcher::{AllMatcher, Matcher, PrefixMatcher};
pub use provider::{CompletionProvider, StaticProvider};
pub use suggestion::{Suggestion, SuggestionKind};

// Re-export ItemSource for custom providers
pub use providers::dynamic::ItemSource;

// Re-export common types for convenience
pub mod prelude {
    pub use crate::{
        AutocompleteEngine, CompletionContext, CompletionProvider, CompletionResult, ContextKind,
        Matcher, Suggestion, SuggestionKind,
    };
}
