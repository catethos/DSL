//! Rich theme system for data type visualization

use egui::Color32;

/// A comprehensive color palette for the DSL REPL UI
#[derive(Debug, Clone)]
pub struct Theme {
    /// Syntax highlighting colors
    pub syntax: SyntaxColors,
    
    /// Data type colors (for values in output)
    pub data_types: DataTypeColors,
    
    /// UI element colors
    pub ui: UiColors,
    
    /// Status/feedback colors
    pub status: StatusColors,
}

/// Syntax highlighting colors (for code editor/input)
#[derive(Debug, Clone)]
pub struct SyntaxColors {
    pub keyword: Color32,
    pub keyword_special: Color32,
    pub type_name: Color32,
    pub type_builtin: Color32,
    pub function: Color32,
    pub function_call: Color32,
    pub variable: Color32,
    pub constant: Color32,
    pub property: Color32,
    pub operator: Color32,
    pub punctuation: Color32,
    pub string: Color32,
    pub number: Color32,
    pub comment: Color32,
    pub default: Color32,
}

/// Data type colors (for runtime values displayed in REPL output)
#[derive(Debug, Clone)]
pub struct DataTypeColors {
    /// Strings - warm orange
    pub string: Color32,
    
    /// Numbers (int/float) - soft cyan
    pub number: Color32,
    
    /// Booleans - vibrant green/red
    pub bool_true: Color32,
    pub bool_false: Color32,
    
    /// Null/undefined - muted gray
    pub null: Color32,
    
    /// Object keys - light blue
    pub object_key: Color32,
    
    /// Array indices - purple
    pub array_index: Color32,
    
    /// Map/Object braces - bright teal
    pub object_punctuation: Color32,
    
    /// Array brackets - bright purple
    pub array_punctuation: Color32,
    
    /// Function names in output - yellow
    pub function_name: Color32,
    
    /// Special types (Image, Markdown, etc.) - magenta
    pub special_type: Color32,
}

/// UI element colors
#[derive(Debug, Clone)]
pub struct UiColors {
    /// Table header background
    pub table_header_bg: Color32,
    
    /// Table header text
    pub table_header_text: Color32,
    
    /// Table row alternate (striped)
    pub table_row_alt: Color32,
    
    /// Table cell text
    pub table_cell_text: Color32,
    
    /// Selected row highlight
    pub table_selected: Color32,
    
    /// Tree node expand/collapse arrow
    pub tree_arrow: Color32,
    
    /// Tree indentation guide
    pub tree_guide: Color32,
    
    /// Panel borders
    pub panel_border: Color32,
    
    /// Active pane highlight
    pub active_pane: Color32,
}

/// Status and feedback colors
#[derive(Debug, Clone)]
pub struct StatusColors {
    /// Success messages
    pub success: Color32,
    
    /// Error messages
    pub error: Color32,
    
    /// Warning messages
    pub warning: Color32,
    
    /// Info messages
    pub info: Color32,
    
    /// Error background
    pub error_bg: Color32,
    
    /// Error border
    pub error_border: Color32,
}

impl Theme {
    /// Create a rich dark theme with vibrant, modern colors
    pub fn dark() -> Self {
        Self {
            syntax: SyntaxColors {
                keyword: Color32::from_rgb(189, 147, 249),       // Purple (Dracula-inspired)
                keyword_special: Color32::from_rgb(255, 121, 198), // Pink
                type_name: Color32::from_rgb(80, 250, 123),      // Green
                type_builtin: Color32::from_rgb(139, 233, 253),  // Cyan
                function: Color32::from_rgb(241, 250, 140),      // Yellow
                function_call: Color32::from_rgb(241, 250, 140), // Yellow
                variable: Color32::from_rgb(189, 210, 255),      // Light blue
                constant: Color32::from_rgb(139, 233, 253),      // Cyan
                property: Color32::from_rgb(189, 210, 255),      // Light blue
                operator: Color32::from_rgb(248, 248, 242),      // Off-white
                punctuation: Color32::from_rgb(248, 248, 242),   // Off-white
                string: Color32::from_rgb(255, 184, 108),        // Orange
                number: Color32::from_rgb(189, 147, 249),        // Purple
                comment: Color32::from_rgb(98, 114, 164),        // Blue-gray
                default: Color32::from_rgb(248, 248, 242),       // Off-white
            },
            
            data_types: DataTypeColors {
                string: Color32::from_rgb(255, 184, 108),        // Warm orange
                number: Color32::from_rgb(139, 233, 253),        // Bright cyan
                bool_true: Color32::from_rgb(80, 250, 123),      // Vibrant green
                bool_false: Color32::from_rgb(255, 85, 85),      // Vibrant red
                null: Color32::from_rgb(130, 130, 130),          // Gray
                object_key: Color32::from_rgb(139, 233, 253),    // Cyan
                array_index: Color32::from_rgb(189, 147, 249),   // Purple
                object_punctuation: Color32::from_rgb(80, 250, 123), // Green
                array_punctuation: Color32::from_rgb(255, 121, 198), // Pink
                function_name: Color32::from_rgb(241, 250, 140), // Yellow
                special_type: Color32::from_rgb(255, 121, 198),  // Pink
            },
            
            ui: UiColors {
                table_header_bg: Color32::from_rgb(36, 40, 50),
                table_header_text: Color32::from_rgb(248, 248, 242),
                table_row_alt: Color32::from_rgba_premultiplied(139, 233, 253, 8),
                table_cell_text: Color32::from_rgb(248, 248, 242),
                table_selected: Color32::from_rgb(68, 71, 90),
                tree_arrow: Color32::from_rgb(139, 233, 253),
                tree_guide: Color32::from_rgba_premultiplied(98, 114, 164, 40),
                panel_border: Color32::from_rgb(68, 71, 90),
                active_pane: Color32::from_rgb(88, 166, 255),
            },
            
            status: StatusColors {
                success: Color32::from_rgb(80, 250, 123),
                error: Color32::from_rgb(255, 85, 85),
                warning: Color32::from_rgb(241, 250, 140),
                info: Color32::from_rgb(139, 233, 253),
                error_bg: Color32::from_rgb(40, 28, 32),
                error_border: Color32::from_rgb(255, 85, 85),
            },
        }
    }
    
    /// Create a light theme with good contrast
    pub fn light() -> Self {
        Self {
            syntax: SyntaxColors {
                keyword: Color32::from_rgb(140, 40, 180),        // Purple
                keyword_special: Color32::from_rgb(180, 0, 120), // Magenta
                type_name: Color32::from_rgb(0, 120, 140),       // Teal
                type_builtin: Color32::from_rgb(0, 80, 200),     // Blue
                function: Color32::from_rgb(120, 90, 20),        // Brown
                function_call: Color32::from_rgb(120, 90, 20),   // Brown
                variable: Color32::from_rgb(0, 60, 140),         // Dark blue
                constant: Color32::from_rgb(0, 100, 180),        // Blue
                property: Color32::from_rgb(0, 60, 140),         // Dark blue
                operator: Color32::from_rgb(40, 40, 40),         // Dark gray
                punctuation: Color32::from_rgb(40, 40, 40),      // Dark gray
                string: Color32::from_rgb(180, 40, 40),          // Red
                number: Color32::from_rgb(0, 120, 60),           // Green
                comment: Color32::from_rgb(0, 128, 0),           // Green
                default: Color32::from_rgb(0, 0, 0),             // Black
            },
            
            data_types: DataTypeColors {
                string: Color32::from_rgb(200, 60, 40),          // Red-orange
                number: Color32::from_rgb(0, 140, 120),          // Teal
                bool_true: Color32::from_rgb(0, 160, 0),         // Green
                bool_false: Color32::from_rgb(200, 40, 40),      // Red
                null: Color32::from_rgb(120, 120, 120),          // Gray
                object_key: Color32::from_rgb(0, 80, 180),       // Blue
                array_index: Color32::from_rgb(120, 60, 180),    // Purple
                object_punctuation: Color32::from_rgb(0, 120, 100), // Teal
                array_punctuation: Color32::from_rgb(140, 40, 180), // Purple
                function_name: Color32::from_rgb(140, 100, 0),   // Gold
                special_type: Color32::from_rgb(200, 0, 140),    // Magenta
            },
            
            ui: UiColors {
                table_header_bg: Color32::from_rgb(230, 230, 235),
                table_header_text: Color32::from_rgb(40, 40, 40),
                table_row_alt: Color32::from_rgba_premultiplied(200, 200, 200, 25),
                table_cell_text: Color32::from_rgb(40, 40, 40),
                table_selected: Color32::from_rgb(100, 160, 240),
                tree_arrow: Color32::from_rgb(100, 100, 100),
                tree_guide: Color32::from_rgba_premultiplied(150, 150, 150, 80),
                panel_border: Color32::from_rgb(180, 180, 180),
                active_pane: Color32::from_rgb(80, 140, 220),
            },
            
            status: StatusColors {
                success: Color32::from_rgb(0, 160, 0),
                error: Color32::from_rgb(220, 40, 40),
                warning: Color32::from_rgb(200, 140, 0),
                info: Color32::from_rgb(0, 100, 200),
                error_bg: Color32::from_rgb(255, 240, 240),
                error_border: Color32::from_rgb(220, 80, 80),
            },
        }
    }
}

/// Helper function to format a value with appropriate color based on type
pub fn value_color(value_str: &str, theme: &Theme) -> Color32 {
    // Detect value type from string representation
    if value_str == "true" {
        theme.data_types.bool_true
    } else if value_str == "false" {
        theme.data_types.bool_false
    } else if value_str == "null" {
        theme.data_types.null
    } else if value_str.starts_with('"') && value_str.ends_with('"') {
        theme.data_types.string
    } else if value_str.parse::<f64>().is_ok() {
        theme.data_types.number
    } else if value_str.starts_with('{') {
        theme.data_types.object_punctuation
    } else if value_str.starts_with('[') {
        theme.data_types.array_punctuation
    } else if value_str.starts_with("<image:") || value_str.starts_with("<markdown:") {
        theme.data_types.special_type
    } else {
        theme.syntax.default
    }
}
