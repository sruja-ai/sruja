//! DOT layout and styling constants (ported from Go)

// Layout Constants
pub const DEFAULT_NODE_SEP: f64 = 110.0;
pub const DEFAULT_RANK_SEP: f64 = 120.0;
pub const DEFAULT_NODE_WIDTH: f64 = 0.0; // Dynamic sizing
pub const DEFAULT_NODE_HEIGHT: f64 = 0.0;

// Min dimensions
pub const MIN_WIDTH_PERSON: f64 = 200.0;
pub const MIN_HEIGHT_PERSON: f64 = 180.0;
pub const MIN_WIDTH_SYSTEM: f64 = 220.0;
pub const MIN_HEIGHT_SYSTEM: f64 = 140.0;
pub const MIN_WIDTH_CONTAINER: f64 = 200.0;
pub const MIN_HEIGHT_CONTAINER: f64 = 120.0;
pub const MIN_WIDTH_COMPONENT: f64 = 180.0;
pub const MIN_HEIGHT_COMPONENT: f64 = 100.0;
pub const MIN_WIDTH_INFRASTRUCTURE: f64 = 200.0;
pub const MIN_HEIGHT_INFRASTRUCTURE: f64 = 100.0;

// Styling
pub const FONT_NAME: &str = "Arial";
pub const FONT_SIZE_GLOBAL: u32 = 12;
pub const FONT_SIZE_EDGE: u32 = 11;
pub const COLOR_SLATE_500: &str = "#596980";
pub const COLOR_SLATE_700: &str = "#4A5568";
pub const COLOR_SLATE_800: &str = "#2D3748";
pub const COLOR_GRAY_BG: &str = "#f8f9fa";
pub const PEN_WIDTH_EDGE: u32 = 2;
pub const ARROW_SIZE: f64 = 0.75;
pub const GRAPH_PAD: f64 = 0.2;
