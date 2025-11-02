//! Dynamic completion providers that work with any data source

use crate::{CompletionContext, CompletionProvider, ContextKind, Suggestion, SuggestionKind};
use std::sync::Arc;

/// A trait for providing a list of items dynamically
///
/// This allows providers to query any data source (HashMap, Vec, database, etc.)
/// without coupling to a specific implementation.
pub trait ItemSource: Send + Sync {
    /// Get all items as (name, detail) pairs
    fn items(&self) -> Vec<(String, Option<String>)>;
}

/// A provider for functions from any source
pub struct FunctionProvider {
    source: Arc<dyn ItemSource>,
}

impl FunctionProvider {
    /// Create a new function provider with a custom source
    pub fn new(source: Arc<dyn ItemSource>) -> Self {
        Self { source }
    }
}

impl CompletionProvider for FunctionProvider {
    fn name(&self) -> &str {
        "functions"
    }

    fn can_provide(&self, context: &CompletionContext) -> bool {
        // Provide functions in general context, but not in specific contexts
        matches!(context.kind, ContextKind::General)
    }

    fn provide(&self, _context: &CompletionContext) -> Vec<Suggestion> {
        self.source
            .items()
            .into_iter()
            .map(|(name, detail)| {
                let mut suggestion = Suggestion::new(name, SuggestionKind::Function)
                    .priority(SuggestionKind::Function.default_priority());

                if let Some(detail) = detail {
                    suggestion = suggestion.detail(detail);
                }

                suggestion
            })
            .collect()
    }

    fn priority(&self) -> i32 {
        80 // High priority - functions are commonly used
    }
}

/// A provider for variables from any source
pub struct VariableProvider {
    source: Arc<dyn ItemSource>,
}

impl VariableProvider {
    /// Create a new variable provider with a custom source
    pub fn new(source: Arc<dyn ItemSource>) -> Self {
        Self { source }
    }
}

impl CompletionProvider for VariableProvider {
    fn name(&self) -> &str {
        "variables"
    }

    fn can_provide(&self, context: &CompletionContext) -> bool {
        // Provide variables in general context
        matches!(context.kind, ContextKind::General)
    }

    fn provide(&self, _context: &CompletionContext) -> Vec<Suggestion> {
        self.source
            .items()
            .into_iter()
            .map(|(name, detail)| {
                let mut suggestion = Suggestion::new(name, SuggestionKind::Variable)
                    .priority(SuggestionKind::Variable.default_priority());

                if let Some(detail) = detail {
                    suggestion = suggestion.detail(detail);
                }

                suggestion
            })
            .collect()
    }

    fn priority(&self) -> i32 {
        90 // Highest priority - variables are most commonly used
    }
}

/// A provider for custom types from any source
pub struct TypeProvider {
    source: Arc<dyn ItemSource>,
}

impl TypeProvider {
    /// Create a new type provider with a custom source
    pub fn new(source: Arc<dyn ItemSource>) -> Self {
        Self { source }
    }
}

impl CompletionProvider for TypeProvider {
    fn name(&self) -> &str {
        "types"
    }

    fn can_provide(&self, context: &CompletionContext) -> bool {
        // Provide types in general context
        matches!(context.kind, ContextKind::General)
    }

    fn provide(&self, _context: &CompletionContext) -> Vec<Suggestion> {
        self.source
            .items()
            .into_iter()
            .map(|(name, detail)| {
                let mut suggestion = Suggestion::new(name, SuggestionKind::Type)
                    .priority(SuggestionKind::Type.default_priority());

                if let Some(detail) = detail {
                    suggestion = suggestion.detail(detail);
                }

                suggestion
            })
            .collect()
    }

    fn priority(&self) -> i32 {
        60 // Medium priority
    }
}

/// A simple in-memory item source for testing
#[derive(Clone)]
pub struct StaticItemSource {
    items: Vec<(String, Option<String>)>,
}

impl StaticItemSource {
    /// Create a new static item source
    pub fn new(items: Vec<(String, Option<String>)>) -> Self {
        Self { items }
    }
}

impl ItemSource for StaticItemSource {
    fn items(&self) -> Vec<(String, Option<String>)> {
        self.items.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_provider() {
        let source = Arc::new(StaticItemSource::new(vec![
            ("Upper".to_string(), Some("(String) -> String".to_string())),
            ("Lower".to_string(), Some("(String) -> String".to_string())),
        ]));

        let provider = FunctionProvider::new(source);
        let context = CompletionContext::new("Up", 2);

        assert!(provider.can_provide(&context));
        let suggestions = provider.provide(&context);

        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.label == "Upper"));
    }

    #[test]
    fn test_variable_provider() {
        let source = Arc::new(StaticItemSource::new(vec![
            ("foo".to_string(), Some("String".to_string())),
            ("bar".to_string(), Some("Int".to_string())),
        ]));

        let provider = VariableProvider::new(source);
        let context = CompletionContext::new("f", 1);

        assert!(provider.can_provide(&context));
        let suggestions = provider.provide(&context);

        assert_eq!(suggestions.len(), 2);
        assert!(suggestions
            .iter()
            .all(|s| s.kind == SuggestionKind::Variable));
    }
}
