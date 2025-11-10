//! REPL command definitions
//!
//! This module defines all available REPL commands and their descriptions.
//! These commands are used both for autocomplete and for command execution in app.rs

/// REPL command definition
#[derive(Debug, Clone)]
pub struct CommandDef {
    pub command: &'static str,
    pub description: &'static str,
}

/// Get all available REPL commands with their descriptions
pub fn all_commands() -> Vec<CommandDef> {
    vec![
        CommandDef {
            command: ":help",
            description: "Show help message",
        },
        CommandDef {
            command: ":clear",
            description: "Clear the screen",
        },
        CommandDef {
            command: ":vars",
            description: "Show all variables",
        },
        CommandDef {
            command: ":scopes",
            description: "Show scope stack (debug)",
        },
        CommandDef {
            command: ":globals",
            description: "Show global scope variables",
        },
        CommandDef {
            command: ":types",
            description: "Show all types",
        },
        CommandDef {
            command: ":funcs",
            description: "Show all functions",
        },
        CommandDef {
            command: ":quit",
            description: "Quit the REPL",
        },
        CommandDef {
            command: ":q",
            description: "Quit the REPL (short)",
        },
        CommandDef {
            command: ":save",
            description: "Save session",
        },
        CommandDef {
            command: ":load",
            description: "Load session",
        },
        CommandDef {
            command: ":debug",
            description: "Toggle debug mode",
        },
        CommandDef {
            command: ":copy",
            description: "Copy last result",
        },
    ]
}

/// Get commands formatted for autocomplete (command, description) tuples
pub fn commands_for_autocomplete() -> Vec<(&'static str, &'static str)> {
    all_commands()
        .into_iter()
        .map(|cmd| (cmd.command, cmd.description))
        .collect()
}
