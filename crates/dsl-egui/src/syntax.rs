//! Syntax highlighting for DSL code using Tree-sitter

use egui::{text::LayoutJob, Color32, FontId, TextFormat};
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

/// Lazy static for the Tree-sitter language and highlighter config
static HIGHLIGHTER_CONFIG: std::sync::OnceLock<HighlightConfiguration> = std::sync::OnceLock::new();

/// Highlight names used in the queries/highlights.scm file
const HIGHLIGHT_NAMES: &[&str] = &[
    "keyword",
    "keyword.special",
    "type",
    "type.builtin",
    "type.definition",
    "function",
    "function.call",
    "variable",
    "variable.parameter",
    "constant",
    "constant.builtin",
    "property",
    "operator",
    "punctuation.bracket",
    "punctuation.delimiter",
    "string",
    "embedded",
    "number",
    "comment",
];

/// Get or initialize the highlighter configuration
fn get_highlighter_config() -> &'static HighlightConfiguration {
    HIGHLIGHTER_CONFIG.get_or_init(|| {
        let language = tree_sitter_dsl::language();

        // Load the highlights query from the embedded file
        let highlights_query = include_str!("../../../tree-sitter-dsl/queries/highlights.scm");

        let mut config = HighlightConfiguration::new(
            language,
            "dsl",
            highlights_query,
            "", // injections query (not used)
            "", // locals query (not used)
        )
        .expect("Failed to create highlight configuration");

        config.configure(HIGHLIGHT_NAMES);
        config
    })
}

/// Color scheme for syntax highlighting
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub keyword: Color32,
    pub keyword_special: Color32,
    pub type_name: Color32,
    pub type_builtin: Color32,
    pub type_definition: Color32,
    pub function: Color32,
    pub function_call: Color32,
    pub variable: Color32,
    pub variable_parameter: Color32,
    pub constant: Color32,
    pub constant_builtin: Color32,
    pub property: Color32,
    pub operator: Color32,
    pub punctuation_bracket: Color32,
    pub punctuation_delimiter: Color32,
    pub string: Color32,
    pub embedded: Color32,
    pub number: Color32,
    pub comment: Color32,
    pub default: Color32,
}

impl ColorScheme {
    /// Create a color scheme suitable for dark backgrounds
    pub fn dark() -> Self {
        Self {
            keyword: Color32::from_rgb(197, 134, 192),         // Purple
            keyword_special: Color32::from_rgb(220, 140, 190), // Bright purple
            type_name: Color32::from_rgb(78, 201, 176),        // Teal
            type_builtin: Color32::from_rgb(86, 156, 214),     // Blue
            type_definition: Color32::from_rgb(78, 201, 176),  // Teal
            function: Color32::from_rgb(220, 220, 170),        // Yellow
            function_call: Color32::from_rgb(220, 220, 170),   // Yellow
            variable: Color32::from_rgb(156, 220, 254),        // Light blue
            variable_parameter: Color32::from_rgb(156, 220, 254), // Light blue
            constant: Color32::from_rgb(79, 193, 255),         // Cyan
            constant_builtin: Color32::from_rgb(86, 156, 214), // Blue
            property: Color32::from_rgb(156, 220, 254),        // Light blue
            operator: Color32::from_rgb(212, 212, 212),        // Light gray
            punctuation_bracket: Color32::from_rgb(212, 212, 212), // Light gray
            punctuation_delimiter: Color32::from_rgb(212, 212, 212), // Light gray
            string: Color32::from_rgb(206, 145, 120),          // Orange
            embedded: Color32::from_rgb(156, 220, 254),        // Light blue
            number: Color32::from_rgb(181, 206, 168),          // Green
            comment: Color32::from_rgb(106, 153, 85),          // Dark green
            default: Color32::from_rgb(212, 212, 212),         // Light gray
        }
    }

    /// Create a color scheme suitable for light backgrounds
    pub fn light() -> Self {
        Self {
            keyword: Color32::from_rgb(175, 0, 219),           // Purple
            keyword_special: Color32::from_rgb(200, 0, 150),   // Bright purple
            type_name: Color32::from_rgb(38, 127, 153),        // Teal
            type_builtin: Color32::from_rgb(0, 0, 255),        // Blue
            type_definition: Color32::from_rgb(38, 127, 153),  // Teal
            function: Color32::from_rgb(121, 94, 38),          // Brown
            function_call: Color32::from_rgb(121, 94, 38),     // Brown
            variable: Color32::from_rgb(0, 16, 128),           // Dark blue
            variable_parameter: Color32::from_rgb(0, 16, 128), // Dark blue
            constant: Color32::from_rgb(0, 92, 197),           // Blue
            constant_builtin: Color32::from_rgb(0, 0, 255),    // Blue
            property: Color32::from_rgb(0, 16, 128),           // Dark blue
            operator: Color32::from_rgb(0, 0, 0),              // Black
            punctuation_bracket: Color32::from_rgb(0, 0, 0),   // Black
            punctuation_delimiter: Color32::from_rgb(0, 0, 0), // Black
            string: Color32::from_rgb(163, 21, 21),            // Red
            embedded: Color32::from_rgb(0, 16, 128),           // Dark blue
            number: Color32::from_rgb(9, 134, 88),             // Green
            comment: Color32::from_rgb(0, 128, 0),             // Green
            default: Color32::from_rgb(0, 0, 0),               // Black
        }
    }

    /// Get color for a highlight name
    fn get_color(&self, highlight_index: usize) -> Color32 {
        match HIGHLIGHT_NAMES.get(highlight_index) {
            Some(&"keyword") => self.keyword,
            Some(&"keyword.special") => self.keyword_special,
            Some(&"type") => self.type_name,
            Some(&"type.builtin") => self.type_builtin,
            Some(&"type.definition") => self.type_definition,
            Some(&"function") => self.function,
            Some(&"function.call") => self.function_call,
            Some(&"variable") => self.variable,
            Some(&"variable.parameter") => self.variable_parameter,
            Some(&"constant") => self.constant,
            Some(&"constant.builtin") => self.constant_builtin,
            Some(&"property") => self.property,
            Some(&"operator") => self.operator,
            Some(&"punctuation.bracket") => self.punctuation_bracket,
            Some(&"punctuation.delimiter") => self.punctuation_delimiter,
            Some(&"string") => self.string,
            Some(&"embedded") => self.embedded,
            Some(&"number") => self.number,
            Some(&"comment") => self.comment,
            _ => self.default,
        }
    }
}

/// Highlight DSL code and return an egui LayoutJob
pub fn highlight_code(code: &str, font_id: FontId, color_scheme: &ColorScheme) -> LayoutJob {
    let mut job = LayoutJob::default();

    // Try to highlight with Tree-sitter
    if let Ok(highlighted) = highlight_code_internal(code, &font_id, color_scheme) {
        return highlighted;
    }

    // Fallback: return unhighlighted text
    job.append(
        code,
        0.0,
        TextFormat {
            font_id,
            color: color_scheme.default,
            ..Default::default()
        },
    );

    job
}

/// Internal highlighting implementation
fn highlight_code_internal(
    code: &str,
    font_id: &FontId,
    color_scheme: &ColorScheme,
) -> Result<LayoutJob, Box<dyn std::error::Error>> {
    let mut job = LayoutJob::default();
    let config = get_highlighter_config();
    let mut highlighter = Highlighter::new();

    let highlights = highlighter.highlight(config, code.as_bytes(), None, |_| None)?;

    let mut current_pos = 0;
    let mut current_highlight: Option<usize> = None;

    for event in highlights {
        match event? {
            HighlightEvent::Source { start, end } => {
                // Get the text for this range
                let text = &code[start..end];

                // Get the color based on current highlight
                let color = if let Some(highlight_idx) = current_highlight {
                    color_scheme.get_color(highlight_idx)
                } else {
                    color_scheme.default
                };

                // Append to the layout job
                job.append(
                    text,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );

                current_pos = end;
            }
            HighlightEvent::HighlightStart(highlight) => {
                current_highlight = Some(highlight.0);
            }
            HighlightEvent::HighlightEnd => {
                current_highlight = None;
            }
        }
    }

    // Add any remaining text
    if current_pos < code.len() {
        job.append(
            &code[current_pos..],
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color: color_scheme.default,
                ..Default::default()
            },
        );
    }

    Ok(job)
}

/// Create a simple highlighted text widget
pub fn highlighted_code_label(
    ui: &mut egui::Ui,
    code: &str,
    font_id: FontId,
    color_scheme: &ColorScheme,
) {
    let job = highlight_code(code, font_id, color_scheme);
    ui.label(job);
}

/// Create a highlighted multiline text widget (non-editable)
pub fn highlighted_code_block(
    ui: &mut egui::Ui,
    code: &str,
    font_id: FontId,
    color_scheme: &ColorScheme,
) {
    let job = highlight_code(code, font_id, color_scheme);
    ui.add(egui::Label::new(job).wrap());
}
