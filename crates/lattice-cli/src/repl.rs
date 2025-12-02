//! Interactive REPL (Read-Eval-Print Loop) for Lattice
//!
//! Provides an interactive session with:
//! - Persistent VM state across inputs
//! - Multiline input support
//! - Command history via rustyline
//! - Syntax error reporting with line numbers

use anyhow::{Context, Result};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Completer, Editor, Helper, Highlighter, Hinter, Validator};

use lattice::compiler::{CompileResult, Compiler};
use lattice::syntax::parser;
use lattice::types::Value;
use lattice::vm::VM;

/// REPL helper combining bracket validation, highlighting, hints, etc.
#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
struct ReplHelper {
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Highlighter)]
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}

impl Default for ReplHelper {
    fn default() -> Self {
        Self {
            validator: MatchingBracketValidator::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter::new(),
        }
    }
}

/// The interactive REPL state
pub struct Repl {
    /// The VM instance - persists across inputs
    vm: VM,
    /// Line editor with history
    editor: Editor<ReplHelper, DefaultHistory>,
    /// Current input buffer for multiline input
    buffer: String,
    /// Whether we're in multiline input mode
    multiline: bool,
    /// Line offset for error reporting in multiline mode
    line_offset: usize,
}

impl Repl {
    /// Create a new REPL instance
    pub fn new() -> Result<Self> {
        let helper = ReplHelper::default();
        let config = rustyline::Config::builder()
            .auto_add_history(true)
            .build();
        let mut editor = Editor::with_config(config)?;
        editor.set_helper(Some(helper));

        // Try to load history
        let history_path = dirs_history_path();
        if let Some(ref path) = history_path {
            let _ = editor.load_history(path);
        }

        Ok(Self {
            vm: VM::new(),
            editor,
            buffer: String::new(),
            multiline: false,
            line_offset: 0,
        })
    }

    /// Run the REPL loop
    pub fn run(&mut self) -> Result<()> {
        println!("Lattice REPL v{}", env!("CARGO_PKG_VERSION"));
        println!("Type :help for help, :quit to exit");
        println!();

        loop {
            let prompt = if self.multiline { "... " } else { ">>> " };

            match self.editor.readline(prompt) {
                Ok(line) => {
                    let line = line.to_string();
                    if let Some(result) = self.handle_line(&line)? {
                        if result == ReplAction::Quit {
                            break;
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl-C: cancel current input
                    if self.multiline {
                        println!("^C");
                        self.reset_input();
                    } else {
                        println!("^C (use :quit to exit)");
                    }
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl-D: exit
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    break;
                }
            }
        }

        // Save history
        if let Some(ref path) = dirs_history_path() {
            let _ = self.editor.save_history(path);
        }

        Ok(())
    }

    /// Handle a line of input
    fn handle_line(&mut self, line: &str) -> Result<Option<ReplAction>> {
        // Handle REPL commands (start with :)
        if !self.multiline && line.trim().starts_with(':') {
            return self.handle_command(line.trim());
        }

        // Accumulate input
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(line);

        // Check if input is complete
        if self.is_input_complete(&self.buffer) {
            let input = std::mem::take(&mut self.buffer);
            self.multiline = false;
            self.evaluate(&input)?;
            self.line_offset = 0;
        } else {
            self.multiline = true;
            if !self.multiline {
                self.line_offset = 0;
            }
        }

        Ok(None)
    }

    /// Check if the current input buffer is complete
    fn is_input_complete(&self, input: &str) -> bool {
        let trimmed = input.trim();

        // Empty input is complete
        if trimmed.is_empty() {
            return true;
        }

        // Count brackets/braces/parens
        let mut brace_count = 0i32;
        let mut bracket_count = 0i32;
        let mut paren_count = 0i32;
        let mut in_string = false;
        let mut in_raw_string = false;
        let mut prev_char = ' ';

        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];

            // Handle raw strings (triple quotes)
            if !in_string && i + 2 < chars.len() && c == '"' && chars[i + 1] == '"' && chars[i + 2] == '"' {
                if in_raw_string {
                    in_raw_string = false;
                    i += 3;
                    continue;
                } else {
                    in_raw_string = true;
                    i += 3;
                    continue;
                }
            }

            // Handle regular strings
            if !in_raw_string && c == '"' && prev_char != '\\' {
                in_string = !in_string;
            }

            // Count brackets only outside strings
            if !in_string && !in_raw_string {
                match c {
                    '{' => brace_count += 1,
                    '}' => brace_count -= 1,
                    '[' => bracket_count += 1,
                    ']' => bracket_count -= 1,
                    '(' => paren_count += 1,
                    ')' => paren_count -= 1,
                    _ => {}
                }
            }

            prev_char = c;
            i += 1;
        }

        // Input is incomplete if:
        // - We're inside a string
        // - Unbalanced brackets
        if in_string || in_raw_string {
            return false;
        }
        if brace_count != 0 || bracket_count != 0 || paren_count != 0 {
            return false;
        }

        // Check for continuation patterns
        let trimmed = input.trim_end();

        // Lines ending with operators that expect more
        let continuation_patterns = [
            "&&", "||", "+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=",
            ",", "->", "=>", "=",
        ];

        for pattern in &continuation_patterns {
            if trimmed.ends_with(pattern) {
                return false;
            }
        }

        true
    }

    /// Evaluate input and print the result
    fn evaluate(&mut self, input: &str) -> Result<()> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        // Parse the input
        let program = match parser::parse(trimmed) {
            Ok(p) => p,
            Err(e) => {
                self.print_parse_error(&e.to_string(), input);
                return Ok(());
            }
        };

        // Compile
        let compile_result = match Compiler::compile(&program) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Compile error: {}", e);
                return Ok(());
            }
        };

        // Register types and functions in the VM (persistent state)
        self.register_artifacts(&compile_result);

        // Execute
        match self.vm.run(&compile_result.chunk) {
            Ok(value) => {
                // Print result (unless null)
                if !matches!(value, Value::Null) {
                    println!("{}", value);
                }
            }
            Err(e) => {
                eprintln!("Runtime error: {}", e);
            }
        }

        // Reset VM execution state but keep globals and registered items
        self.vm.reset();

        Ok(())
    }

    /// Register compiled artifacts in the VM
    fn register_artifacts(&mut self, result: &CompileResult) {
        // Register types
        for class in &result.classes {
            self.vm.ir_mut().classes.push(class.clone());
        }
        for enum_def in &result.enums {
            self.vm.ir_mut().enums.push(enum_def.clone());
        }

        // Register functions
        for func in &result.functions {
            self.vm.register_function(func.clone());
        }
        for llm_func in &result.llm_functions {
            self.vm.register_llm_function(llm_func.clone());
        }
    }

    /// Print a parse error with context
    fn print_parse_error(&self, error: &str, _input: &str) {
        eprintln!("Parse error: {}", error);
    }

    /// Handle REPL commands
    fn handle_command(&mut self, cmd: &str) -> Result<Option<ReplAction>> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let command = parts.first().map(|s| *s).unwrap_or("");

        match command {
            ":quit" | ":q" | ":exit" => {
                println!("Goodbye!");
                Ok(Some(ReplAction::Quit))
            }
            ":help" | ":h" | ":?" => {
                self.print_help();
                Ok(None)
            }
            ":clear" | ":reset" => {
                self.vm.clear();
                println!("VM state cleared.");
                Ok(None)
            }
            ":vars" | ":globals" => {
                self.print_globals();
                Ok(None)
            }
            ":types" => {
                self.print_types();
                Ok(None)
            }
            ":functions" | ":funcs" => {
                self.print_functions();
                Ok(None)
            }
            ":load" => {
                if parts.len() < 2 {
                    eprintln!("Usage: :load <filename>");
                } else {
                    self.load_file(parts[1])?;
                }
                Ok(None)
            }
            _ => {
                eprintln!("Unknown command: {}", command);
                eprintln!("Type :help for available commands");
                Ok(None)
            }
        }
    }

    /// Print help message
    fn print_help(&self) {
        println!("Lattice REPL Commands:");
        println!("  :help, :h, :?     Show this help message");
        println!("  :quit, :q, :exit  Exit the REPL");
        println!("  :clear, :reset    Clear all VM state (globals, types, functions)");
        println!("  :vars, :globals   List all global variables");
        println!("  :types            List all defined types");
        println!("  :functions        List all defined functions");
        println!("  :load <file>      Load and execute a Lattice file");
        println!();
        println!("Tips:");
        println!("  - Press Ctrl+C to cancel current input");
        println!("  - Press Ctrl+D to exit");
        println!("  - Use arrow keys for history navigation");
        println!("  - Multiline input is supported (brackets auto-detect continuation)");
    }

    /// Print all global variables
    fn print_globals(&self) {
        let names: Vec<_> = self.vm.global_names().collect();
        if names.is_empty() {
            println!("No global variables defined.");
        } else {
            println!("Global variables:");
            for name in names {
                if let Ok(value) = self.vm.get_global(name) {
                    println!("  {} = {}", name, value);
                }
            }
        }
    }

    /// Print all defined types
    fn print_types(&self) {
        let ir = self.vm.ir();

        if ir.classes.is_empty() && ir.enums.is_empty() {
            println!("No types defined.");
            return;
        }

        if !ir.classes.is_empty() {
            println!("Classes:");
            for class in &ir.classes {
                print!("  type {} {{ ", class.name);
                let fields: Vec<_> = class.fields.iter()
                    .map(|f| format!("{}: {:?}", f.name, f.field_type))
                    .collect();
                print!("{}", fields.join(", "));
                println!(" }}");
            }
        }

        if !ir.enums.is_empty() {
            println!("Enums:");
            for enum_def in &ir.enums {
                println!("  enum {} {{ {} }}", enum_def.name, enum_def.values.join(", "));
            }
        }
    }

    /// Print all defined functions
    fn print_functions(&self) {
        // Note: We can't easily iterate user_functions since it's private
        // For now, just indicate that functions exist
        println!("(Function listing not yet implemented)");
    }

    /// Load and execute a file
    fn load_file(&mut self, path: &str) -> Result<()> {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;

        println!("Loading {}...", path);
        self.evaluate(&source)?;
        println!("Done.");

        Ok(())
    }

    /// Reset the input buffer
    fn reset_input(&mut self) {
        self.buffer.clear();
        self.multiline = false;
        self.line_offset = 0;
    }
}

/// REPL action result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplAction {
    Quit,
}

/// Get the path for REPL history file
fn dirs_history_path() -> Option<std::path::PathBuf> {
    dirs::data_local_dir().map(|mut p| {
        p.push("lattice");
        let _ = std::fs::create_dir_all(&p);
        p.push("repl_history");
        p
    })
}
