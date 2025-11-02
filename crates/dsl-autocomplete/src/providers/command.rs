//! REPL command completion provider

use crate::{CompletionContext, CompletionProvider, ContextKind, Suggestion, SuggestionKind};

/// A provider for REPL commands
pub struct CommandProvider {
    commands: Vec<Suggestion>,
}

impl CommandProvider {
    /// Create a new command provider with default REPL commands
    pub fn new() -> Self {
        let commands = vec![
            Suggestion::new(":help", SuggestionKind::Command)
                .detail("Show help message")
                .priority(SuggestionKind::Command.default_priority()),
            Suggestion::new(":clear", SuggestionKind::Command)
                .detail("Clear the screen")
                .priority(SuggestionKind::Command.default_priority()),
            Suggestion::new(":vars", SuggestionKind::Command)
                .detail("Show all variables")
                .priority(SuggestionKind::Command.default_priority()),
            Suggestion::new(":types", SuggestionKind::Command)
                .detail("Show all types")
                .priority(SuggestionKind::Command.default_priority()),
            Suggestion::new(":funcs", SuggestionKind::Command)
                .detail("Show all functions")
                .priority(SuggestionKind::Command.default_priority()),
            Suggestion::new(":quit", SuggestionKind::Command)
                .detail("Quit the REPL")
                .priority(SuggestionKind::Command.default_priority()),
            Suggestion::new(":q", SuggestionKind::Command)
                .detail("Quit the REPL (short)")
                .priority(SuggestionKind::Command.default_priority()),
            Suggestion::new(":save", SuggestionKind::Command)
                .detail("Save session")
                .priority(SuggestionKind::Command.default_priority()),
            Suggestion::new(":load", SuggestionKind::Command)
                .detail("Load session")
                .priority(SuggestionKind::Command.default_priority()),
            Suggestion::new(":debug", SuggestionKind::Command)
                .detail("Toggle debug mode")
                .priority(SuggestionKind::Command.default_priority()),
            Suggestion::new(":copy", SuggestionKind::Command)
                .detail("Copy last result")
                .priority(SuggestionKind::Command.default_priority()),
        ];

        Self { commands }
    }

    /// Create a custom command provider with specific commands
    pub fn with_commands(commands: Vec<(&str, &str)>) -> Self {
        let commands = commands
            .into_iter()
            .map(|(cmd, detail)| {
                Suggestion::new(cmd, SuggestionKind::Command)
                    .detail(detail)
                    .priority(SuggestionKind::Command.default_priority())
            })
            .collect();

        Self { commands }
    }
}

impl Default for CommandProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionProvider for CommandProvider {
    fn name(&self) -> &str {
        "commands"
    }

    fn can_provide(&self, context: &CompletionContext) -> bool {
        // Only provide commands when we're in command context
        matches!(context.kind, ContextKind::Command)
    }

    fn provide(&self, _context: &CompletionContext) -> Vec<Suggestion> {
        self.commands.clone()
    }

    fn priority(&self) -> i32 {
        100 // High priority for commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_provider() {
        let provider = CommandProvider::new();
        let context = CompletionContext::new(":h", 2);

        assert!(provider.can_provide(&context));
        let suggestions = provider.provide(&context);

        assert!(suggestions.iter().any(|s| s.label == ":help"));
        assert!(suggestions.iter().any(|s| s.label == ":clear"));
    }

    #[test]
    fn test_command_provider_filtering() {
        let provider = CommandProvider::new();

        // Should not provide for general context
        let context = CompletionContext::new("foo", 3);
        assert!(!provider.can_provide(&context));

        // Should provide for command context
        let context = CompletionContext::new(":q", 2);
        assert!(provider.can_provide(&context));
    }
}
