use std::sync::{Arc, Mutex};
use lattice::compiler::Compiler;
use lattice::syntax::parser;
use lattice::types::Value;
use lattice::vm::VM;
use serde::Serialize;

struct AppState {
    vm: Arc<Mutex<VM>>,
}

/// Debug info from an LLM call (serializable for UI)
#[derive(Debug, Clone, Serialize, Default)]
pub struct LlmDebugOutput {
    /// The function name that was called
    pub function_name: String,
    /// The return type as a string
    pub return_type: String,
    /// The generated prompt sent to the LLM
    pub prompt: String,
    /// The raw response from the LLM
    pub raw_response: String,
}

/// Structured output from cell evaluation
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum CellOutput {
    /// Empty output (null result)
    Empty,
    /// Plain text output
    Text(String),
    /// Table output with headers and rows
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

/// Full response from eval_cell including debug info
#[derive(Debug, Clone, Serialize)]
pub struct EvalResponse {
    /// The cell output
    pub output: CellOutput,
    /// Debug info from the last LLM call (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_debug: Option<LlmDebugOutput>,
}

impl CellOutput {
    /// Convert a Value to CellOutput, detecting table-like structures
    fn from_value(value: Value) -> Self {
        match &value {
            Value::Null => CellOutput::Empty,
            Value::List(items) if !items.is_empty() => {
                // Check if all items are maps with the same keys (table-like)
                if let Some(table) = Self::try_as_table(items) {
                    return table;
                }
                // Not a table, return as text
                CellOutput::Text(value.to_string())
            }
            _ => CellOutput::Text(value.to_string()),
        }
    }

    /// Try to interpret a list of values as a table
    fn try_as_table(items: &[Value]) -> Option<CellOutput> {
        // All items must be maps
        let maps: Vec<_> = items
            .iter()
            .filter_map(|v| v.as_map())
            .collect();

        if maps.len() != items.len() || maps.is_empty() {
            return None;
        }

        // Get headers from the first map
        let first_map = maps[0];
        let mut headers: Vec<String> = first_map.keys().cloned().collect();
        headers.sort(); // Consistent ordering

        // Check all maps have the same keys
        for map in &maps {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            if keys != headers {
                return None;
            }
        }

        // Build rows
        let rows: Vec<Vec<String>> = maps
            .iter()
            .map(|map| {
                headers
                    .iter()
                    .map(|h| {
                        map.get(h)
                            .map(|v| Self::value_to_cell_string(v))
                            .unwrap_or_default()
                    })
                    .collect()
            })
            .collect();

        Some(CellOutput::Table { headers, rows })
    }

    /// Convert a Value to a string suitable for a table cell
    fn value_to_cell_string(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Null => "".to_string(),
            other => other.to_string(),
        }
    }
}

#[tauri::command]
async fn reset_vm(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let vm = Arc::clone(&state.vm);
    tokio::task::spawn_blocking(move || {
        let mut vm = vm.lock().map_err(|e| e.to_string())?;
        *vm = VM::new();
        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
async fn eval_cell(state: tauri::State<'_, AppState>, code: String) -> Result<EvalResponse, String> {
    // Clone the Arc so we can move it into the blocking task
    let vm = Arc::clone(&state.vm);

    // Run the VM execution on a blocking thread pool to avoid nested runtime issues
    tokio::task::spawn_blocking(move || {
        let mut vm = vm.lock().map_err(|e| e.to_string())?;

        // Parse source to AST
        let program = parser::parse(&code).map_err(|e| format!("Parse error: {}", e))?;

        // Get known function names from previously executed cells
        let known_functions = vm.function_names();
        let known_llm_functions = vm.llm_function_names();

        // Compile AST to bytecode with knowledge of existing functions
        let compile_result =
            Compiler::compile_with_known_functions_full(&program, known_functions, known_llm_functions)
                .map_err(|e| format!("Compile error: {}", e))?;

        // Register types (classes and enums)
        for class in compile_result.classes {
            vm.ir_mut().classes.push(class);
        }
        for enum_def in compile_result.enums {
            vm.ir_mut().enums.push(enum_def);
        }

        // Register functions
        for func in compile_result.functions {
            vm.register_function(func);
        }
        for llm_func in compile_result.llm_functions {
            vm.register_llm_function(llm_func);
        }

        // Execute the bytecode
        let result = vm
            .run(&compile_result.chunk)
            .map_err(|e| format!("Runtime error: {}", e))?;

        // Get LLM debug info if available
        let llm_debug = vm.take_llm_debug().map(|debug| LlmDebugOutput {
            function_name: debug.function_name,
            return_type: debug.return_type,
            prompt: debug.prompt,
            raw_response: debug.raw_response,
        });

        // Convert result to structured CellOutput
        Ok(EvalResponse {
            output: CellOutput::from_value(result),
            llm_debug,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            vm: Arc::new(Mutex::new(VM::new())),
        })
        .invoke_handler(tauri::generate_handler![eval_cell, reset_vm])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
