//! Builtin function metadata for autocomplete and documentation
//!
//! This module provides a central definition of all builtin functions
//! and their signatures, ensuring consistency between the interpreter
//! and autocomplete systems.

/// Builtin function metadata
#[derive(Debug, Clone)]
pub struct BuiltinFunctionInfo {
    pub name: &'static str,
    pub signature: &'static str,
    pub description: &'static str,
}

/// Get all builtin functions with their signatures and descriptions
pub fn all_builtins() -> Vec<BuiltinFunctionInfo> {
    vec![
        // String functions
        BuiltinFunctionInfo {
            name: "Upper",
            signature: "(String) -> String",
            description: "Convert string to uppercase",
        },
        BuiltinFunctionInfo {
            name: "Lower",
            signature: "(String) -> String",
            description: "Convert string to lowercase",
        },
        BuiltinFunctionInfo {
            name: "Length",
            signature: "(String) -> Int",
            description: "Get length of a string or list",
        },
        BuiltinFunctionInfo {
            name: "Trim",
            signature: "(String) -> String",
            description: "Remove whitespace from both ends",
        },
        BuiltinFunctionInfo {
            name: "Split",
            signature: "(String, String) -> List",
            description: "Split string by separator",
        },
        BuiltinFunctionInfo {
            name: "Replace",
            signature: "(String, String, String) -> String",
            description: "Replace all occurrences of pattern with replacement",
        },
        BuiltinFunctionInfo {
            name: "Contains",
            signature: "(String, String) -> Bool",
            description: "Check if string contains substring",
        },
        BuiltinFunctionInfo {
            name: "StartsWith",
            signature: "(String, String) -> Bool",
            description: "Check if string starts with prefix",
        },
        BuiltinFunctionInfo {
            name: "EndsWith",
            signature: "(String, String) -> Bool",
            description: "Check if string ends with suffix",
        },
        BuiltinFunctionInfo {
            name: "Join",
            signature: "(List, String) -> String",
            description: "Join list elements with separator",
        },
        // LLM functions
        BuiltinFunctionInfo {
            name: "Ask",
            signature: "(String) -> String",
            description: "Send prompt to LLM and get response",
        },
        // Rendering functions
        BuiltinFunctionInfo {
            name: "RenderMarkdown",
            signature: "(String) -> Markdown",
            description: "Render markdown text",
        },
        // SQL functions
        BuiltinFunctionInfo {
            name: "SQL",
            signature: "(String) -> Table",
            description: "Execute SQL query and return table. Use $variable syntax to auto-register tables",
        },
        BuiltinFunctionInfo {
            name: "refresh_table",
            signature: "(String) -> Null",
            description: "Clear cached SQL table, forcing re-registration on next use",
        },
        // Concurrency functions
        BuiltinFunctionInfo {
            name: "Par",
            signature: "(...) -> List",
            description: "Execute expressions in parallel",
        },
        // List functions
        BuiltinFunctionInfo {
            name: "Reverse",
            signature: "(List) -> List",
            description: "Reverse a list",
        },
        BuiltinFunctionInfo {
            name: "Sort",
            signature: "(List) -> List",
            description: "Sort a list in ascending order",
        },
        BuiltinFunctionInfo {
            name: "Unique",
            signature: "(List) -> List",
            description: "Remove duplicate elements from list",
        },
        BuiltinFunctionInfo {
            name: "Take",
            signature: "(List, Int) -> List",
            description: "Take first N elements from list",
        },
        BuiltinFunctionInfo {
            name: "Skip",
            signature: "(List, Int) -> List",
            description: "Skip first N elements from list",
        },
        BuiltinFunctionInfo {
            name: "First",
            signature: "(List) -> Any",
            description: "Get first element of list",
        },
        BuiltinFunctionInfo {
            name: "Last",
            signature: "(List) -> Any",
            description: "Get last element of list",
        },
        BuiltinFunctionInfo {
            name: "Flatten",
            signature: "(List) -> List",
            description: "Flatten nested lists by one level",
        },
        // Math functions
        BuiltinFunctionInfo {
            name: "Abs",
            signature: "(Int | Float) -> Int | Float",
            description: "Absolute value",
        },
        BuiltinFunctionInfo {
            name: "Min",
            signature: "(List) -> Int | Float",
            description: "Find minimum value in list",
        },
        BuiltinFunctionInfo {
            name: "Max",
            signature: "(List) -> Int | Float",
            description: "Find maximum value in list",
        },
        BuiltinFunctionInfo {
            name: "Sum",
            signature: "(List) -> Int | Float",
            description: "Sum all numbers in list",
        },
        BuiltinFunctionInfo {
            name: "Average",
            signature: "(List) -> Float",
            description: "Calculate average of numbers in list",
        },
        BuiltinFunctionInfo {
            name: "Round",
            signature: "(Float) -> Int",
            description: "Round to nearest integer",
        },
        BuiltinFunctionInfo {
            name: "Floor",
            signature: "(Float) -> Int",
            description: "Round down to integer",
        },
        BuiltinFunctionInfo {
            name: "Ceil",
            signature: "(Float) -> Int",
            description: "Round up to integer",
        },
        // Type conversion functions
        BuiltinFunctionInfo {
            name: "ToString",
            signature: "(Any) -> String",
            description: "Convert value to string",
        },
        BuiltinFunctionInfo {
            name: "ToInt",
            signature: "(String | Float) -> Int",
            description: "Convert to integer",
        },
        BuiltinFunctionInfo {
            name: "ToFloat",
            signature: "(String | Int) -> Float",
            description: "Convert to float",
        },
        // Functional programming functions
        BuiltinFunctionInfo {
            name: "Map",
            signature: "(List, Function) -> List",
            description: "Apply function to each element",
        },
        BuiltinFunctionInfo {
            name: "Filter",
            signature: "(List, Function) -> List",
            description: "Keep elements matching predicate",
        },
        BuiltinFunctionInfo {
            name: "Reduce",
            signature: "(List, Any, Function) -> Any",
            description: "Accumulate result using function",
        },
        BuiltinFunctionInfo {
            name: "Any",
            signature: "(List, Function) -> Bool",
            description: "Check if any element matches predicate",
        },
        BuiltinFunctionInfo {
            name: "All",
            signature: "(List, Function) -> Bool",
            description: "Check if all elements match predicate",
        },
        BuiltinFunctionInfo {
            name: "Find",
            signature: "(List, Function) -> Any",
            description: "Find first element matching predicate",
        },
        BuiltinFunctionInfo {
            name: "Count",
            signature: "(List, Function) -> Int",
            description: "Count elements matching predicate",
        },
        BuiltinFunctionInfo {
            name: "Zip",
            signature: "(List, List) -> List",
            description: "Combine two lists into pairs",
        },
        BuiltinFunctionInfo {
            name: "Range",
            signature: "(Int, Int) -> List",
            description: "Generate range of integers [start, end)",
        },
        BuiltinFunctionInfo {
            name: "Repeat",
            signature: "(Any, Int) -> List",
            description: "Create list with value repeated N times",
        },
        BuiltinFunctionInfo {
            name: "Chunk",
            signature: "(List, Int) -> List",
            description: "Split list into chunks of size N",
        },
        // Logic functions
        BuiltinFunctionInfo {
            name: "Not",
            signature: "(Bool) -> Bool",
            description: "Logical NOT operation",
        },
        // Chart generation functions
        BuiltinFunctionInfo {
            name: "GenerateBarChart",
            signature: "(List, String?) -> Image",
            description: "Generate a bar chart from data",
        },
        BuiltinFunctionInfo {
            name: "GenerateLineChart",
            signature: "(List, String?) -> Image",
            description: "Generate a line chart from data",
        },
        BuiltinFunctionInfo {
            name: "GeneratePieChart",
            signature: "(List, String?) -> Image",
            description: "Generate a pie chart from data",
        },
    ]
}

/// Get builtin functions formatted for autocomplete (name, signature) tuples
pub fn builtins_for_autocomplete() -> Vec<(String, Option<String>)> {
    all_builtins()
        .into_iter()
        .map(|info| (info.name.to_string(), Some(info.signature.to_string())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_builtins() {
        let builtins = all_builtins();
        assert!(builtins.iter().any(|b| b.name == "Ask"));
        assert!(builtins.iter().any(|b| b.name == "Upper"));
        assert!(builtins.iter().any(|b| b.name == "SQL"));
    }

    #[test]
    fn test_builtins_for_autocomplete() {
        let builtins = builtins_for_autocomplete();
        assert!(builtins.iter().any(|(name, _)| name == "Ask"));
        assert!(builtins
            .iter()
            .any(|(name, sig)| name == "Upper" && sig.as_deref() == Some("(String) -> String")));
    }
}
