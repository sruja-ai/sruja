//! Markdown export options (ported from Go)

use crate::mermaid::MermaidConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextType {
    Default,
    CodeGeneration,
    Review,
    Analysis,
}

impl Default for ContextType {
    fn default() -> Self {
        Self::Default
    }
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
    pub include_toc: bool,
    pub include_overview: bool,
    pub include_systems: bool,
    pub include_deployments: bool,
    pub include_persons: bool,
    pub include_requirements: bool,
    pub include_adrs: bool,
    pub include_scenarios: bool,
    pub include_metadata: bool,
    pub include_glossary: bool,
    pub include_recommendations: bool,
    pub mermaid_config: MermaidConfig,
    pub heading_level: u32,
    pub scope: Scope,
    pub token_limit: usize,
    pub context: ContextType,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            include_toc: true,
            include_overview: true,
            include_systems: true,
            include_deployments: true,
            include_persons: true,
            include_requirements: true,
            include_adrs: true,
            include_scenarios: true,
            include_metadata: true,
            include_glossary: true,
            include_recommendations: true,
            mermaid_config: MermaidConfig::default(),
            heading_level: 1,
            scope: Scope::full(),
            token_limit: 0,
            context: ContextType::Default,
        }
    }
}
