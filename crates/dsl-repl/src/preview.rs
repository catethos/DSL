use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, StepStatus};

pub fn render_preview(f: &mut Frame, area: Rect, app: &App) {
    render_preview_with_title(f, area, app, "Preview");
}

pub fn render_preview_with_title(f: &mut Frame, area: Rect, app: &App, title: &str) {
    if app.preview_steps.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No executions yet",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Tip: Use Ctrl+R to run all",
                Style::default().fg(Color::Cyan),
            )),
            Line::from("     or Ctrl+E to send line"),
        ])
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty_msg, area);
        return;
    }

    // Split preview area into steps list and detail view
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(10)])
        .split(area);

    // Render execution steps
    let items: Vec<ListItem> = app
        .preview_steps
        .iter()
        .map(|step| {
            let icon = match step.status {
                StepStatus::Pending => "○",
                StepStatus::Running => "⏳",
                StepStatus::Complete => "✓",
                StepStatus::Error(_) => "✗",
            };

            let duration = step
                .duration
                .map(|d| format!(" [{:.2}s]", d.as_secs_f64()))
                .unwrap_or_default();

            let text = format!("{} {}{}", icon, step.name, duration);

            let style = match &step.status {
                StepStatus::Complete => Style::default().fg(Color::Green),
                StepStatus::Running => Style::default().fg(Color::Yellow),
                StepStatus::Error(_) => Style::default().fg(Color::Red),
                StepStatus::Pending => Style::default().fg(Color::DarkGray),
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(list, chunks[0]);

    // Render detail view for the first error or running step
    if let Some(step) = app
        .preview_steps
        .iter()
        .find(|s| matches!(s.status, StepStatus::Error(_) | StepStatus::Running))
    {
        let detail_text = match &step.status {
            StepStatus::Error(msg) => vec![Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Color::Red)),
                Span::raw(msg),
            ])],
            StepStatus::Running => vec![Line::from(vec![
                Span::styled("Running: ", Style::default().fg(Color::Yellow)),
                Span::raw(&step.name),
            ])],
            _ => vec![],
        };

        let detail = Paragraph::new(detail_text)
            .block(Block::default().borders(Borders::ALL).title("Details"));

        f.render_widget(detail, chunks[1]);
    } else if let Some(step) = app.preview_steps.last() {
        // Show output of last completed step
        if let (StepStatus::Complete, Some(output)) = (&step.status, &step.output) {
            let output_text = vec![
                Line::from(vec![
                    Span::styled("Output: ", Style::default().fg(Color::Green)),
                    Span::raw(output.type_name()),
                ]),
                Line::from(""),
                Line::from(output.display()),
            ];

            let detail = Paragraph::new(output_text)
                .block(Block::default().borders(Borders::ALL).title("Details"));

            f.render_widget(detail, chunks[1]);
        }
    }
}

pub fn render_type_explorer(f: &mut Frame, area: Rect, app: &App) {
    let types = app.evaluator.types.all();

    if types.is_empty() {
        let empty_msg = Paragraph::new("No types defined yet.\n\nDefine types in the editor or REPL:\n\n  type Person {\n    name: String\n    age: Int\n  }")
            .block(Block::default().borders(Borders::ALL).title("Type Explorer"))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty_msg, area);
        return;
    }

    let mut lines = Vec::new();

    for (name, class) in types {
        lines.push(Line::from(vec![
            Span::styled("type ", Style::default().fg(Color::Magenta)),
            Span::styled(name, Style::default().fg(Color::Cyan)),
        ]));

        for field in &class.fields {
            let optional = if field.optional { "?" } else { "" };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(&field.name, Style::default().fg(Color::Yellow)),
                Span::raw(": "),
                Span::styled(
                    format!("{}{}", field.field_type.to_string(), optional),
                    Style::default().fg(Color::Blue),
                ),
            ]));
        }

        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Type Explorer (F3)"),
    );

    f.render_widget(paragraph, area);
}
