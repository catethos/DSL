//! Autocomplete popup rendering

use crate::autocomplete::AutocompleteState;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

/// Render the autocomplete popup
pub fn render_autocomplete_popup(
    area: Rect,
    buf: &mut Buffer,
    autocomplete: &AutocompleteState,
    cursor_x: u16,
    cursor_y: u16,
) {
    if !autocomplete.is_visible() {
        return;
    }

    let suggestions = &autocomplete.suggestions;
    if suggestions.is_empty() {
        return;
    }

    // Calculate popup dimensions
    let max_width = 50;
    let max_height = 10;

    // Find the longest suggestion label for width calculation
    let content_width = suggestions
        .iter()
        .map(|s| s.label.len() + s.kind.prefix().len() + 3) // +3 for "[X] "
        .max()
        .unwrap_or(20)
        .min(max_width - 2); // -2 for borders

    let popup_width = (content_width + 2) as u16; // +2 for borders
    let popup_height = (suggestions.len().min(max_height) + 2) as u16; // +2 for borders

    // Position popup below cursor
    let popup_x = cursor_x.min(area.width.saturating_sub(popup_width));
    let popup_y = (cursor_y + 1).min(area.height.saturating_sub(popup_height));

    let popup_area = Rect {
        x: area.x + popup_x,
        y: area.y + popup_y,
        width: popup_width.min(area.width.saturating_sub(popup_x)),
        height: popup_height.min(area.height.saturating_sub(popup_y)),
    };

    // Don't render if popup would be too small
    if popup_area.width < 10 || popup_area.height < 3 {
        return;
    }

    // Create list items
    let items: Vec<ListItem> = suggestions
        .iter()
        .take(max_height)
        .enumerate()
        .map(|(i, suggestion)| {
            let is_selected = i == autocomplete.selected_index;

            let kind_style = Style::default().fg(match suggestion.kind {
                dsl_autocomplete::SuggestionKind::Keyword => Color::Magenta,
                dsl_autocomplete::SuggestionKind::Function => Color::Blue,
                dsl_autocomplete::SuggestionKind::Variable => Color::Cyan,
                dsl_autocomplete::SuggestionKind::Type => Color::Yellow,
                dsl_autocomplete::SuggestionKind::Field => Color::Green,
                dsl_autocomplete::SuggestionKind::Command => Color::Red,
                dsl_autocomplete::SuggestionKind::Operator => Color::White,
                dsl_autocomplete::SuggestionKind::Snippet => Color::LightBlue,
                dsl_autocomplete::SuggestionKind::Other => Color::Gray,
            });

            let label_style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let detail_style = if is_selected {
                Style::default().fg(Color::DarkGray).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let mut spans = vec![
                Span::styled(format!("[{}] ", suggestion.kind.prefix()), kind_style),
                Span::styled(&suggestion.label, label_style),
            ];

            if let Some(detail) = &suggestion.detail {
                spans.push(Span::styled(format!(" {}", detail), detail_style));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    // Create the list widget
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Suggestions ")
            .title_alignment(Alignment::Left)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    // Render the list
    list.render(popup_area, buf);
}
