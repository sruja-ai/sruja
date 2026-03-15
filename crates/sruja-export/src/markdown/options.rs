//! Markdown export options (ported from Go)

use crate::mermaid::exporter::MermaidConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContextType {
    #[default]
    Default,
    CodeGeneration,
    Review,
    Analysis,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub r#type: String, // "system", "container", "component", "full"
    pub id: String,
}

impl Scope {
    pub fn full() -> Self {
        Self {
            r#type: "full".to_string(),
            id: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarkdownOptions {
    /// Top-level document title (e.g. "Architecture Overview"). When None, derived from overview summary or default.
    pub document_title: Option<String>,
    pub include_toc: bool,
    pub include_overview: bool,
    pub include_mermaid_diagrams: bool,
    pub include_systems: bool,
    pub include_deployments: bool,
    pub include_persons: bool,
    pub include_requirements: bool,
    pub include_adrs: bool,
    pub include_scenarios: bool,
    pub include_metadata: bool,
    pub include_glossary: bool,
    pub include_recommendations: bool,
    /// When true, add a "Relations" section listing all relations (from → to "label").
    pub include_relations: bool,
    pub mermaid_config: MermaidConfig,
    pub heading_level: u32,
    pub scope: Scope,
    pub token_limit: usize,
    pub context: ContextType,
    pub use_views: bool,
    pub view_name: Option<String>,
    /// When true and use_views is true (and view_name is None), emit a "Custom views" section with all defined views.
    pub include_all_views: bool,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            document_title: None,
            include_toc: true,
            include_overview: true,
            include_mermaid_diagrams: true,
            include_systems: true,
            include_deployments: true,
            include_persons: true,
            include_requirements: true,
            include_adrs: true,
            include_scenarios: true,
            include_metadata: true,
            include_glossary: true,
            include_recommendations: true,
            include_relations: false,
            mermaid_config: MermaidConfig::default(),
            heading_level: 1,
            scope: Scope::full(),
            token_limit: 0,
            context: ContextType::Default,
            use_views: false,
            view_name: None,
            include_all_views: false,
        }
    }
}
