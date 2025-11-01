use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

const REPL_BANNER: &str = r#"
     ██████╗ █████╗ ██████╗ ██╗   ██╗    ███████╗██╗      ██████╗ ██╗    ██╗
    ██╔════╝██╔══██╗██╔══██╗╚██╗ ██╔╝    ██╔════╝██║     ██╔═══██╗██║    ██║
    ██║     ███████║██████╔╝ ╚████╔╝     █████╗  ██║     ██║   ██║██║ █╗ ██║
    ██║     ██╔══██║██╔═══╝   ╚██╔╝      ██╔══╝  ██║     ██║   ██║██║███╗██║
    ╚██████╗██║  ██║██║        ██║       ██║     ███████╗╚██████╔╝╚███╔███╔╝
     ╚═════╝╚═╝  ╚═╝╚═╝        ╚═╝       ╚═╝     ╚══════╝ ╚═════╝  ╚══╝╚══╝

    Agentic LLM Workflow DSL - v0.1.0
"#;

pub fn get_banner() -> Vec<Line<'static>> {
    vec![
        // ASCII Art Logo - "capyFlow"
        Line::from(vec![
            Span::styled(
                "                         ",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                "    _____ _                ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "   ___ __ _ _ __  _   _  ",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                "   |  ___| | _____      __",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  / __/ _` | '_ \\| | | | ",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                "   | |_  | |/ _ \\ \\ /\\ / /",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                " | (_| (_| | |_) | |_| | ",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                "   |  _| | | (_) \\ V  V / ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  \\___\\__,_| .__/ \\__, | ",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                "   |_|   |_|\\___/ \\_/\\_/  ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "           |_|    |___/  ",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                "                          ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            "                      Agentic LLM Workflow DSL",
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        // System Info
        Line::from(vec![
            Span::styled(
                "Version:  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "0.3.0 (Phase 2 Complete)",
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Phase:    ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "3/9 phases complete (33%)",
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Features: ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Literals, Arithmetic, Variables, Binding",
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Commands: ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":vars, :help, :clear", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        // Quick Start
        Line::from(vec![Span::styled(
            "Quick Start:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  → ", Style::default().fg(Color::Blue)),
            Span::styled("5 as x", Style::default().fg(Color::Magenta)),
            Span::styled(
                "          # Bind variable",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("  → ", Style::default().fg(Color::Blue)),
            Span::styled("x + 10", Style::default().fg(Color::Magenta)),
            Span::styled(
                "         # Use in expression",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("  → ", Style::default().fg(Color::Blue)),
            Span::styled("[1,2,3] as nums", Style::default().fg(Color::Magenta)),
            Span::styled(" # Lists", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  → ", Style::default().fg(Color::Blue)),
            Span::styled("nums[0]", Style::default().fg(Color::Magenta)),
            Span::styled(
                "        # Index access",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        // Status Bar
        Line::from(vec![Span::styled(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::White)),
            Span::styled(
                "Esc",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to quit  |  Type ", Style::default().fg(Color::White)),
            Span::styled(
                ":help",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" for help", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![Span::styled(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(""),
    ]
}

pub fn get_repl_artwork() -> Vec<Line<'static>> {
    // Simple, clean banner for workspace mode
    vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  capy",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                "FLOW",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  ════════════",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![Span::styled(
            "  Agentic LLM Workflow DSL",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(""),
    ]
}

pub fn get_help_text() -> Vec<String> {
    vec![
        "DSL REPL - Help".to_string(),
        "".to_string(),
        "Available Commands:".to_string(),
        "  :vars    - Show all variables".to_string(),
        "  :help    - Show this help message".to_string(),
        "  :clear   - Clear the screen".to_string(),
        "".to_string(),
        "Syntax:".to_string(),
        "  Literals:    42, \"hello\", true, [1,2,3]".to_string(),
        "  Arithmetic:  10 + 5, x * 2, y / 3".to_string(),
        "  Binding:     expr as varname".to_string(),
        "  Access:      list[0], obj.field".to_string(),
        "  Special:     _  (last result)".to_string(),
        "".to_string(),
        "Examples:".to_string(),
        "  5 as x           # Bind 5 to variable x".to_string(),
        "  x + 10           # Use x in expression".to_string(),
        "  [1,2,3] as nums  # Create a list".to_string(),
        "  nums[0]          # Access first element".to_string(),
        "".to_string(),
    ]
}
