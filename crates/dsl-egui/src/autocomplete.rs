//! Autocomplete integration for the egui app

use dsl_autocomplete::{
    providers::{
        dynamic::{FunctionProvider, ItemSource, TypeProvider, VariableProvider},
        KeywordProvider,
    },
    AutocompleteEngine, Suggestion,
};
use dsl_interpreter::Runtime;
use std::sync::Arc;

/// Autocomplete state for the egui REPL
pub struct AutocompleteState {
    engine: AutocompleteEngine,
    pub suggestions: Vec<Suggestion>,
    pub selected_index: usize,
    pub show_popup: bool,
}

impl AutocompleteState {
    /// Create a new autocomplete state from Runtime
    pub fn new(runtime: &Runtime) -> Self {
        let mut engine = AutocompleteEngine::new();

        // Register keyword provider (keywords sourced from dsl-core)
        engine.register_provider(Box::new(KeywordProvider::new()));

        // Register function provider
        let func_source = Arc::new(RuntimeFunctionSource::new(runtime));
        engine.register_provider(Box::new(FunctionProvider::new(func_source)));

        // Register variable provider
        let var_source = Arc::new(RuntimeVariableSource::new(runtime));
        engine.register_provider(Box::new(VariableProvider::new(var_source)));

        // Register type provider
        let type_source = Arc::new(RuntimeTypeSource::new(runtime));
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

    /// Update with runtime context (to refresh variables, functions, types)
    pub fn update_with_runtime(&mut self, runtime: &Runtime) {
        let mut engine = AutocompleteEngine::new();

        // Re-register all providers with updated runtime
        engine.register_provider(Box::new(KeywordProvider::new()));

        let func_source = Arc::new(RuntimeFunctionSource::new(runtime));
        engine.register_provider(Box::new(FunctionProvider::new(func_source)));

        let var_source = Arc::new(RuntimeVariableSource::new(runtime));
        engine.register_provider(Box::new(VariableProvider::new(var_source)));

        let type_source = Arc::new(RuntimeTypeSource::new(runtime));
        engine.register_provider(Box::new(TypeProvider::new(type_source)));

        self.engine = engine;
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
    pub fn accept_selected(&mut self, input: &mut String, cursor_position: &mut usize) -> bool {
        if let Some(suggestion) = self.get_selected() {
            // Find the start of the word being completed
            let before_cursor = &input[..*cursor_position];
            let word_start = before_cursor
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
            true
        } else {
            false
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

/// Item source that reads from Runtime functions
struct RuntimeFunctionSource {
    functions: Vec<(String, Option<String>)>,
}

impl RuntimeFunctionSource {
    fn new(runtime: &Runtime) -> Self {
        // Get builtin functions from dsl-interpreter metadata (single source of truth)
        let mut functions = dsl_interpreter::builtins_for_autocomplete();

        // Add user-defined functions
        for (name, func_def) in runtime.functions.iter() {
            #[allow(clippy::format_in_format_args)]
            let detail = if let Some(return_type) = &func_def.return_type {
                Some(format!(
                    "({}) -> {}",
                    func_def.params.join(", "),
                    format!("{:?}", return_type)
                ))
            } else {
                Some(format!("({})", func_def.params.join(", ")))
            };
            functions.push((name.clone(), detail));
        }

        Self { functions }
    }
}

impl ItemSource for RuntimeFunctionSource {
    fn items(&self) -> Vec<(String, Option<String>)> {
        self.functions.clone()
    }
}

/// Item source that reads from Runtime variables
struct RuntimeVariableSource {
    variables: Vec<(String, Option<String>)>,
}

impl RuntimeVariableSource {
    fn new(runtime: &Runtime) -> Self {
        let variables = runtime
            .all_visible_vars()
            .iter()
            .map(|(name, value)| {
                let detail = Some(value.type_name().to_string());
                (name.clone(), detail)
            })
            .collect();

        Self { variables }
    }
}

impl ItemSource for RuntimeVariableSource {
    fn items(&self) -> Vec<(String, Option<String>)> {
        self.variables.clone()
    }
}

/// Item source that reads from Runtime types
struct RuntimeTypeSource {
    types: Vec<(String, Option<String>)>,
}

impl RuntimeTypeSource {
    fn new(runtime: &Runtime) -> Self {
        let mut types = Vec::new();

        // Add classes
        for class in runtime.types.all_classes() {
            types.push((class.name.clone(), Some("type".to_string())));
        }

        // Add enums
        for enum_def in runtime.types.all_enums() {
            types.push((enum_def.name.clone(), Some("enum".to_string())));
        }

        Self { types }
    }
}

impl ItemSource for RuntimeTypeSource {
    fn items(&self) -> Vec<(String, Option<String>)> {
        self.types.clone()
    }
}
