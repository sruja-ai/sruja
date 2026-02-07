//! Mermaid styling and formatting constants (ported from Go)
//!
//! Label length limits follow best practice: keep node/edge labels short so
//! diagrams render without cutoff. Mermaid's wrappingWidth (e.g. 280px) works
//! with these; we truncate at word boundaries and add ellipsis when over limit.

// Node Styles
pub const STYLE_PERSON: &str = "fill:#ffcccc,stroke:#333,stroke-width:2px,color:#000";
pub const STYLE_SYSTEM: &str = "fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000";
pub const STYLE_CONTAINER: &str = "fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000";
pub const STYLE_DATABASE: &str = "fill:#ccffcc,stroke:#333,stroke-width:2px,color:#000";
pub const STYLE_QUEUE: &str = "fill:#ffe5cc,stroke:#333,stroke-width:2px,color:#000";
pub const STYLE_EXTERNAL: &str =
    "fill:#eeeeee,stroke:#666,stroke-width:2px,color:#000,stroke-dasharray: 3 3";
pub const STYLE_COMPONENT: &str = "fill:#e6f7ff,stroke:#333,stroke-width:2px,color:#000";

// Class Names
pub const CLASS_PERSON: &str = "personStyle";
pub const CLASS_SYSTEM: &str = "systemStyle";
pub const CLASS_CONTAINER: &str = "containerStyle";
pub const CLASS_DATABASE: &str = "databaseStyle";
pub const CLASS_QUEUE: &str = "queueStyle";
pub const CLASS_EXTERNAL: &str = "externalStyle";
pub const CLASS_COMPONENT: &str = "componentStyle";

// Formatting
pub const INDENT2: &str = "  ";
pub const INDENT4: &str = "    ";

// Label length limits (best practice: keep labels short so Mermaid can wrap/display without cutoff)
/// Max characters per line in node labels; Mermaid wrappingWidth works with this.
pub const MAX_NODE_LINE_CHARS: usize = 48;
/// Max lines in a node label (title + description + technology).
pub const MAX_NODE_LINES: usize = 3;
/// Max characters for edge/relation labels (single line).
pub const MAX_EDGE_LABEL_CHARS: usize = 40;
/// Max characters for subgraph titles.
pub const MAX_SUBGRAPH_TITLE_CHARS: usize = 42;
