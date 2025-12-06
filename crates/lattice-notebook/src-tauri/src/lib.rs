use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use lattice::runtime::{LatticeRuntime, LatticeValue, RuntimeBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use chrono::Utc;

struct AppState {
    /// Map from session_id to LatticeRuntime instance - each tab gets its own runtime
    runtimes: Arc<Mutex<HashMap<String, LatticeRuntime>>>,
}

/// Debug info from an LLM call (serializable for UI)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// UI state for a cell (persisted for user convenience)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CellUiState {
    /// Editor height in pixels (default: 120)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_height: Option<u32>,
    /// Banner/description height in pixels (default: 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_height: Option<u32>,
    /// Whether the code editor is collapsed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_collapsed: Option<bool>,
    /// Whether the output section is collapsed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_collapsed: Option<bool>,
    /// Monaco editor view state (JSON string, includes folding state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_view_state: Option<String>,
    /// Hierarchy level (0 = root, 1 = child, 2 = grandchild, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
    /// Whether children of this cell are collapsed/hidden
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapsed: Option<bool>,
}

/// A single cell in a notebook file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCell {
    pub id: String,
    pub code: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<CellOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_debug: Option<LlmDebugOutput>,
    /// UI state (heights, collapse states)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_state: Option<CellUiState>,
}

/// The notebook file format (.lat.nb)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub version: String,
    pub created_at: String,
    pub modified_at: String,
    pub cells: Vec<NotebookCell>,
}

impl CellOutput {
    /// Convert a LatticeValue to CellOutput, detecting table-like structures
    fn from_value(value: LatticeValue) -> Self {
        match &value {
            LatticeValue::Null => CellOutput::Empty,
            LatticeValue::List(items) if !items.is_empty() => {
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

    /// Try to interpret a list of LatticeValues as a table
    fn try_as_table(items: &[LatticeValue]) -> Option<CellOutput> {
        // All items must be maps
        let maps: Vec<_> = items
            .iter()
            .filter_map(|v| v.as_map())
            .collect();

        if maps.len() != items.len() || maps.is_empty() {
            return None;
        }

        // Get headers from the first map (LatticeValue::Map is Vec<(String, LatticeValue)>)
        let first_map = maps[0];
        let mut headers: Vec<String> = first_map.iter().map(|(k, _)| k.clone()).collect();
        headers.sort(); // Consistent ordering

        // Check all maps have the same keys
        for map in &maps {
            let mut keys: Vec<String> = map.iter().map(|(k, _)| k.clone()).collect();
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
                        map.iter()
                            .find(|(k, _)| k == h)
                            .map(|(_, v)| Self::value_to_cell_string(v))
                            .unwrap_or_default()
                    })
                    .collect()
            })
            .collect();

        Some(CellOutput::Table { headers, rows })
    }

    /// Convert a LatticeValue to a string suitable for a table cell
    fn value_to_cell_string(value: &LatticeValue) -> String {
        match value {
            LatticeValue::String(s) => s.clone(),
            LatticeValue::Null => "".to_string(),
            other => other.to_string(),
        }
    }
}

/// Create a new runtime session and return its session_id
#[tauri::command]
async fn create_session(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let runtimes = Arc::clone(&state.runtimes);
    tokio::task::spawn_blocking(move || {
        let mut runtimes = runtimes.lock().map_err(|e| e.to_string())?;
        let session_id = generate_id();
        let runtime = RuntimeBuilder::new()
            .with_default_providers()
            .map_err(|e| e.to_string())?
            .build()
            .map(LatticeRuntime::from_built)
            .map_err(|e| e.to_string())?;
        runtimes.insert(session_id.clone(), runtime);
        Ok(session_id)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Destroy a runtime session
#[tauri::command]
async fn destroy_session(state: tauri::State<'_, AppState>, session_id: String) -> Result<(), String> {
    let runtimes = Arc::clone(&state.runtimes);
    tokio::task::spawn_blocking(move || {
        let mut runtimes = runtimes.lock().map_err(|e| e.to_string())?;
        runtimes.remove(&session_id);
        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
async fn reset_runtime(state: tauri::State<'_, AppState>, session_id: String) -> Result<(), String> {
    let runtimes = Arc::clone(&state.runtimes);
    tokio::task::spawn_blocking(move || {
        let mut runtimes = runtimes.lock().map_err(|e| e.to_string())?;
        if let Some(runtime) = runtimes.get_mut(&session_id) {
            runtime.reset();
        } else {
            return Err(format!("Session {} not found", session_id));
        }
        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
async fn eval_cell(state: tauri::State<'_, AppState>, session_id: String, code: String) -> Result<EvalResponse, String> {
    // Clone the Arc so we can move it into the blocking task
    let runtimes = Arc::clone(&state.runtimes);

    // Run the runtime execution on a blocking thread pool to avoid nested runtime issues
    tokio::task::spawn_blocking(move || {
        let start_time = std::time::Instant::now();

        let mut runtimes = runtimes.lock().map_err(|e| e.to_string())?;
        let runtime = runtimes.get_mut(&session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;

        // Evaluate the code using LatticeRuntime
        let result = runtime
            .eval(&code)
            .map_err(|e| format!("{}", e))?;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Convert result to structured CellOutput
        // Note: LLM debug info is not currently exposed through the runtime API
        Ok(EvalResponse {
            output: CellOutput::from_value(result),
            llm_debug: None,
            execution_time_ms,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Save a notebook to a file (.lat.nb format)
#[tauri::command]
async fn save_notebook(path: String, notebook: Notebook) -> Result<(), String> {
    let notebook = Notebook {
        modified_at: Utc::now().to_rfc3339(),
        ..notebook
    };

    let json = serde_json::to_string_pretty(&notebook)
        .map_err(|e| format!("Serialization error: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

/// Load a notebook from a file (.lat.nb format)
#[tauri::command]
async fn load_notebook(path: String) -> Result<Notebook, String> {
    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let notebook: Notebook = serde_json::from_str(&contents)
        .map_err(|e| format!("Invalid notebook format: {}", e))?;

    Ok(notebook)
}

/// Export notebook cells as plain .lat source code
/// Descriptions become block comments, cells are separated by blank lines
#[tauri::command]
async fn export_lat(path: String, cells: Vec<NotebookCell>) -> Result<(), String> {
    let mut output = String::new();

    for (i, cell) in cells.iter().enumerate() {
        if i > 0 {
            output.push_str("\n\n");
        }

        // Add description as block comment if present
        if !cell.description.is_empty() {
            output.push_str("/* ");
            output.push_str(&cell.description);
            output.push_str(" */\n");
        }

        // Add code
        output.push_str(&cell.code);
    }

    fs::write(&path, output)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

/// Import a .lat file as notebook cells
/// Parses block comments as descriptions, splits at blank lines after comments
#[tauri::command]
async fn import_lat(path: String) -> Result<Vec<NotebookCell>, String> {
    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut cells = Vec::new();
    let mut current_description = String::new();
    let mut current_code = String::new();
    let mut in_block_comment = false;

    for line in contents.lines() {
        let trimmed = line.trim();

        // Handle block comment start
        if let Some(after_start) = trimmed.strip_prefix("/*") {
            in_block_comment = true;
            // Check if comment ends on same line
            if let Some(end_idx) = after_start.find("*/") {
                let comment_content = after_start[..end_idx].trim();
                current_description = comment_content.to_string();
                in_block_comment = false;
            } else {
                current_description = after_start.trim().to_string();
            }
            continue;
        }

        // Handle inside block comment
        if in_block_comment {
            if let Some(end_idx) = trimmed.find("*/") {
                let before_end = &trimmed[..end_idx].trim();
                if !before_end.is_empty() {
                    if !current_description.is_empty() {
                        current_description.push('\n');
                    }
                    current_description.push_str(before_end);
                }
                in_block_comment = false;
            } else {
                if !current_description.is_empty() {
                    current_description.push('\n');
                }
                current_description.push_str(trimmed);
            }
            continue;
        }

        // Regular code line
        if !current_code.is_empty() {
            current_code.push('\n');
        }
        current_code.push_str(line);
    }

    // Add the last cell if there's any code
    let code = current_code.trim().to_string();
    if !code.is_empty() || !current_description.is_empty() {
        cells.push(NotebookCell {
            id: generate_id(),
            code,
            description: current_description,
            output: None,
            llm_debug: None,
            ui_state: None,
        });
    }

    // If no cells were created, create one empty cell
    if cells.is_empty() {
        cells.push(NotebookCell {
            id: generate_id(),
            code: String::new(),
            description: String::new(),
            output: None,
            llm_debug: None,
            ui_state: None,
        });
    }

    Ok(cells)
}

/// Generate a simple random ID for cells
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", now)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            runtimes: Arc::new(Mutex::new(HashMap::new())),
        })
        .invoke_handler(tauri::generate_handler![
            create_session,
            destroy_session,
            eval_cell,
            reset_runtime,
            save_notebook,
            load_notebook,
            export_lat,
            import_lat
        ])
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
