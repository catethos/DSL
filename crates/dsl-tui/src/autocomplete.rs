//! Autocomplete integration for the TUI

use dsl_autocomplete::{
    providers::{
        dynamic::{FunctionProvider, ItemSource, TypeProvider, VariableProvider},
        CommandProvider, KeywordProvider,
    },
    AutocompleteEngine, Suggestion,
};
use dsl_core::Evaluator;
use std::sync::Arc;

/// Autocomplete state for the TUI
pub struct AutocompleteState {
    engine: AutocompleteEngine,
    pub suggestions: Vec<Suggestion>,
    pub selected_index: usize,
    pub show_popup: bool,
}

impl AutocompleteState {
    /// Create a new autocomplete state
    pub fn new(evaluator: &Evaluator) -> Self {
        let mut engine = AutocompleteEngine::new();

        // Register keyword provider
        engine.register_provider(Box::new(KeywordProvider::new()));

        // Register command provider
        engine.register_provider(Box::new(CommandProvider::new()));

        // Register function provider
        let func_source = Arc::new(EvaluatorFunctionSource::new(evaluator));
        engine.register_provider(Box::new(FunctionProvider::new(func_source)));

        // Register variable provider
        let var_source = Arc::new(EvaluatorVariableSource::new(evaluator));
        engine.register_provider(Box::new(VariableProvider::new(var_source)));

        // Register type provider
        let type_source = Arc::new(EvaluatorTypeSource::new(evaluator));
        engine.register_provider(Box::new(TypeProvider::new(type_source)));

        Self {
            engine,
            suggestions: Vec::new(),
            selected_index: 0,
            show_popup: false,
        }
    }

    /// Update suggestions based on current input
    pub fn update(&mut self, input: &str, cursor_position: usize) {
        let result = self.engine.complete(input, cursor_position);
        self.suggestions = result.suggestions;
        self.selected_index = 0;
        self.show_popup = !self.suggestions.is_empty();
    }

    /// Select the next suggestion
    pub fn select_next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.suggestions.len();
        }
    }

    /// Select the previous suggestion
    pub fn select_previous(&mut self) {
        if !self.suggestions.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.suggestions.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Get the currently selected suggestion
    pub fn get_selected(&self) -> Option<&Suggestion> {
        self.suggestions.get(self.selected_index)
    }

    /// Accept the currently selected suggestion
    pub fn accept_selected(&mut self, input: &mut String, cursor_position: &mut usize) {
        if let Some(suggestion) = self.get_selected() {
            // Find the start of the word being completed
            let word_start = input[..*cursor_position]
                .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != ':')
                .map(|pos| pos + 1)
                .unwrap_or(0);

            // Replace the partial word with the completion
            let before = &input[..word_start];
            let after = &input[*cursor_position..];
            let insert_text = suggestion.get_insert_text();

            *input = format!("{}{}{}", before, insert_text, after);
            *cursor_position = word_start + insert_text.len();

            self.hide();
        }
    }

    /// Hide the autocomplete popup
    pub fn hide(&mut self) {
        self.show_popup = false;
        self.suggestions.clear();
        self.selected_index = 0;
    }

    /// Check if popup should be shown
    pub fn is_visible(&self) -> bool {
        self.show_popup && !self.suggestions.is_empty()
    }
}

/// Item source that reads from Evaluator functions
struct EvaluatorFunctionSource {
    functions: Vec<(String, Option<String>)>,
}

impl EvaluatorFunctionSource {
    fn new(evaluator: &Evaluator) -> Self {
        let mut functions = Vec::new();

        // Add builtin functions (case-insensitive, but we show them with capital first letter)
        functions.push(("Upper".to_string(), Some("(String) -> String".to_string())));
        functions.push(("Lower".to_string(), Some("(String) -> String".to_string())));
        functions.push(("Length".to_string(), Some("(String) -> Int".to_string())));
        functions.push(("Join".to_string(), Some("(List, String) -> String".to_string())));
        functions.push(("Ask".to_string(), Some("(String) -> String".to_string())));
        functions.push(("ExtractPerson".to_string(), Some("(String) -> Person".to_string())));
        functions.push(("ExtractAs".to_string(), Some("(String, Type) -> Type".to_string())));
        functions.push(("SQL".to_string(), Some("(String) -> Table".to_string())));

        // Add user-defined functions
        for (name, func_def) in evaluator.functions.iter() {
            let detail = if let Some(return_type) = &func_def.return_type {
                Some(format!("({}) -> {}",
                    func_def.params.join(", "),
                    format!("{:?}", return_type)))
            } else {
                Some(format!("({})", func_def.params.join(", ")))
            };
            functions.push((name.clone(), detail));
        }

        Self { functions }
    }
}

impl ItemSource for EvaluatorFunctionSource {
    fn items(&self) -> Vec<(String, Option<String>)> {
        self.functions.clone()
    }
}

/// Item source that reads from Evaluator variables
struct EvaluatorVariableSource {
    variables: Vec<(String, Option<String>)>,
}

impl EvaluatorVariableSource {
    fn new(evaluator: &Evaluator) -> Self {
        let variables = evaluator
            .vars
            .iter()
            .map(|(name, value)| {
                let detail = Some(value.type_name().to_string());
                (name.clone(), detail)
            })
            .collect();

        Self { variables }
    }
}

impl ItemSource for EvaluatorVariableSource {
    fn items(&self) -> Vec<(String, Option<String>)> {
        self.variables.clone()
    }
}

/// Item source that reads from Evaluator types
struct EvaluatorTypeSource {
    types: Vec<(String, Option<String>)>,
}

impl EvaluatorTypeSource {
    fn new(evaluator: &Evaluator) -> Self {
        let mut types = Vec::new();

        // Add classes
        for class in evaluator.types.all_classes() {
            types.push((class.name.clone(), Some("type".to_string())));
        }

        // Add enums
        for enum_def in evaluator.types.all_enums() {
            types.push((enum_def.name.clone(), Some("enum".to_string())));
        }

        Self { types }
    }
}

impl ItemSource for EvaluatorTypeSource {
    fn items(&self) -> Vec<(String, Option<String>)> {
        self.types.clone()
    }
}
