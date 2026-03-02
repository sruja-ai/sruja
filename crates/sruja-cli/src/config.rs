//! Configuration management for Sruja CLI
//!
//! Handles loading and parsing .sruja.yaml configuration files

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Config file not found: {0}")]
    NotFound(PathBuf),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SrujaConfig {
    #[serde(default)]
    pub views: HashMap<String, ViewDefinition>,

    #[serde(default)]
    pub defaults: DefaultSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
    #[serde(default)]
    pub extends: Option<String>,

    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub sections: Vec<String>,

    #[serde(default)]
    pub exclude: Vec<String>,

    #[serde(default)]
    pub thresholds: ThresholdConfig,

    #[serde(default)]
    pub terminology: HashMap<String, String>,

    #[serde(default)]
    pub llm_prompt: Option<String>,

    #[serde(default)]
    pub focus_areas: Vec<String>,

    #[serde(default)]
    pub analysis_depth: AnalysisDepth,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThresholdConfig {
    #[serde(default = "default_max_coupling")]
    pub max_coupling: f32,

    #[serde(default = "default_max_orphans")]
    pub max_orphans: usize,

    #[serde(default = "default_max_complexity")]
    pub max_complexity: f32,

    #[serde(default = "default_min_health")]
    pub min_health: f32,

    #[serde(default)]
    pub custom: HashMap<String, f32>,
}

fn default_max_coupling() -> f32 {
    10.0
}
fn default_max_orphans() -> usize {
    5
}
fn default_max_complexity() -> f32 {
    15.0
}
fn default_min_health() -> f32 {
    70.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum AnalysisDepth {
    Quick,
    #[default]
    Standard,
    Deep,
    Comprehensive,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultSettings {
    #[serde(default)]
    pub default_view: Option<String>,

    #[serde(default)]
    pub output_format: Option<String>,

    #[serde(default)]
    pub enable_llm: bool,
}

impl SrujaConfig {
    pub fn load(repo_path: &Path) -> Result<Self, ConfigError> {
        let config_paths = vec![
            repo_path.join(".sruja.yaml"),
            repo_path.join(".sruja.yml"),
            repo_path.join("sruja.yaml"),
            repo_path.join("sruja.yml"),
        ];

        for config_path in config_paths {
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let config: SrujaConfig = serde_yaml::from_str(&content)?;
                return Ok(config);
            }
        }

        Ok(SrujaConfig::default())
    }

    pub fn get_view(&self, view_name: &str) -> Option<ViewDefinition> {
        self.views.get(view_name).map(|v| self.resolve_view(v))
    }

    fn resolve_view(&self, view: &ViewDefinition) -> ViewDefinition {
        if let Some(ref extends) = view.extends {
            if let Some(parent) = self.views.get(extends) {
                let resolved_parent = self.resolve_view(parent);
                return self.merge_views(&resolved_parent, view);
            }
        }
        view.clone()
    }

    fn merge_views(&self, parent: &ViewDefinition, child: &ViewDefinition) -> ViewDefinition {
        let mut merged = parent.clone();

        if !child.name.is_empty() {
            merged.name = child.name.clone();
        }

        if child.description.is_some() {
            merged.description = child.description.clone();
        }

        if !child.sections.is_empty() {
            let mut sections = parent.sections.clone();
            for section in &child.sections {
                if !sections.contains(section) {
                    sections.push(section.clone());
                }
            }
            merged.sections = sections;
        }

        for excluded in &child.exclude {
            merged.sections.retain(|s| s != excluded);
        }

        merged.exclude.extend(child.exclude.clone());
        merged.exclude.sort();
        merged.exclude.dedup();

        merged.thresholds.max_coupling = child.thresholds.max_coupling;
        merged.thresholds.max_orphans = child.thresholds.max_orphans;
        merged.thresholds.max_complexity = child.thresholds.max_complexity;
        merged.thresholds.min_health = child.thresholds.min_health;
        merged
            .thresholds
            .custom
            .extend(child.thresholds.custom.clone());

        merged.terminology.extend(child.terminology.clone());

        if child.llm_prompt.is_some() {
            merged.llm_prompt = child.llm_prompt.clone();
        }

        merged.focus_areas.extend(child.focus_areas.clone());
        merged.focus_areas.sort();
        merged.focus_areas.dedup();

        if child.analysis_depth != AnalysisDepth::Standard {
            merged.analysis_depth = child.analysis_depth.clone();
        }

        merged
    }
}

pub fn get_builtin_views() -> HashMap<String, ViewDefinition> {
    let mut views = HashMap::new();

    views.insert("cto".to_string(), ViewDefinition {
        name: "CTO Report".to_string(),
        description: Some("Executive architecture summary for technology leaders".to_string()),
        sections: vec![
            "executive_summary".to_string(),
            "tech_stack".to_string(),
            "tech_debt".to_string(),
            "risks".to_string(),
            "recommendations".to_string(),
            "cost_optimization".to_string(),
            "team_impact".to_string(),
        ],
        thresholds: ThresholdConfig {
            max_coupling: 8.0,
            max_orphans: 3,
            max_complexity: 12.0,
            min_health: 80.0,
            custom: HashMap::new(),
        },
        terminology: [
            ("service".to_string(), "microservice".to_string()),
            ("database".to_string(), "data store".to_string()),
            ("module".to_string(), "component".to_string()),
        ].into_iter().collect(),
        llm_prompt: Some("Analyze this architecture from an executive perspective. Focus on business impact, strategic risks, and high-level recommendations suitable for CTO review.".to_string()),
        focus_areas: vec![
            "business_alignment".to_string(),
            "strategic_risks".to_string(),
            "vendor_lock_in".to_string(),
            "scalability_constraints".to_string(),
        ],
        analysis_depth: AnalysisDepth::Standard,
        extends: None,
        exclude: vec![],
    });

    views.insert("sre".to_string(), ViewDefinition {
        name: "SRE Report".to_string(),
        description: Some("Reliability and incident analysis for site reliability engineers".to_string()),
        sections: vec![
            "infrastructure".to_string(),
            "reliability".to_string(),
            "dependencies".to_string(),
            "single_points_of_failure".to_string(),
            "incident_patterns".to_string(),
            "cost_optimization".to_string(),
        ],
        thresholds: ThresholdConfig {
            max_coupling: 5.0,
            max_orphans: 2,
            max_complexity: 10.0,
            min_health: 90.0,
            custom: [
                ("max_sprinkle_effect".to_string(), 3.0),
                ("min_availability".to_string(), 99.9),
            ].into_iter().collect(),
        },
        terminology: [
            ("service".to_string(), "application".to_string()),
            ("database".to_string(), "persistence layer".to_string()),
            ("error".to_string(), "incident".to_string()),
        ].into_iter().collect(),
        llm_prompt: Some("Analyze this architecture for reliability and operational concerns. Identify potential failure modes, single points of failure, and suggest improvements for maintaining SLOs.".to_string()),
        focus_areas: vec![
            "availability".to_string(),
            "latency".to_string(),
            "error_budgets".to_string(),
            "disaster_recovery".to_string(),
            "monitoring".to_string(),
        ],
        analysis_depth: AnalysisDepth::Deep,
        extends: None,
        exclude: vec![],
    });

    views.insert("devops".to_string(), ViewDefinition {
        name: "DevOps Report".to_string(),
        description: Some("Deployment readiness and infrastructure analysis".to_string()),
        sections: vec![
            "deployment_readiness".to_string(),
            "infrastructure".to_string(),
            "cicd".to_string(),
            "containerization".to_string(),
            "blockers".to_string(),
            "recommendations".to_string(),
        ],
        thresholds: ThresholdConfig {
            max_coupling: 6.0,
            max_orphans: 5,
            max_complexity: 12.0,
            min_health: 75.0,
            custom: HashMap::new(),
        },
        terminology: [
            ("service".to_string(), "deployable unit".to_string()),
            ("module".to_string(), "artifact".to_string()),
        ].into_iter().collect(),
        llm_prompt: Some("Analyze this architecture for deployment and operational concerns. Identify deployment blockers, CI/CD gaps, and infrastructure improvements.".to_string()),
        focus_areas: vec![
            "deployment_velocity".to_string(),
            "build_times".to_string(),
            "infrastructure_as_code".to_string(),
            "containerization".to_string(),
        ],
        analysis_depth: AnalysisDepth::Quick,
        extends: None,
        exclude: vec![],
    });

    views.insert("security".to_string(), ViewDefinition {
        name: "Security Report".to_string(),
        description: Some("Security vulnerability and compliance analysis".to_string()),
        sections: vec![
            "attack_surface".to_string(),
            "vulnerabilities".to_string(),
            "data_flows".to_string(),
            "compliance".to_string(),
            "recommendations".to_string(),
        ],
        thresholds: ThresholdConfig {
            max_coupling: 8.0,
            max_orphans: 3,
            max_complexity: 10.0,
            min_health: 95.0,
            custom: [
                ("max_external_deps".to_string(), 5.0),
            ].into_iter().collect(),
        },
        terminology: [
            ("service".to_string(), "attack vector".to_string()),
            ("database".to_string(), "data store".to_string()),
            ("error".to_string(), "vulnerability".to_string()),
        ].into_iter().collect(),
        llm_prompt: Some("Analyze this architecture from a security perspective. Identify attack vectors, potential vulnerabilities, data flow risks, and compliance concerns.".to_string()),
        focus_areas: vec![
            "authentication".to_string(),
            "authorization".to_string(),
            "data_encryption".to_string(),
            "network_security".to_string(),
            "compliance".to_string(),
        ],
        analysis_depth: AnalysisDepth::Comprehensive,
        extends: None,
        exclude: vec![],
    });

    views.insert("product".to_string(), ViewDefinition {
        name: "Product Report".to_string(),
        description: Some("Feature dependency and impact analysis for product teams".to_string()),
        sections: vec![
            "features".to_string(),
            "dependencies".to_string(),
            "impact_analysis".to_string(),
            "team_coordination".to_string(),
            "recommendations".to_string(),
        ],
        thresholds: ThresholdConfig {
            max_coupling: 10.0,
            max_orphans: 5,
            max_complexity: 15.0,
            min_health: 70.0,
            custom: HashMap::new(),
        },
        terminology: [
            ("service".to_string(), "feature".to_string()),
            ("module".to_string(), "capability".to_string()),
        ].into_iter().collect(),
        llm_prompt: Some("Analyze this architecture from a product perspective. Identify feature dependencies, coordination needs between teams, and impact of potential changes.".to_string()),
        focus_areas: vec![
            "feature_dependencies".to_string(),
            "team_boundaries".to_string(),
            "change_impact".to_string(),
            "release_coordination".to_string(),
        ],
        analysis_depth: AnalysisDepth::Standard,
        extends: None,
        exclude: vec![],
    });

    views.insert("platform-engineer".to_string(), ViewDefinition {
        name: "Platform Engineer Report".to_string(),
        description: Some("Infrastructure and platform concerns for platform engineers".to_string()),
        extends: Some("sre".to_string()),
        sections: vec![
            "cost_optimization".to_string(),
            "resource_efficiency".to_string(),
        ],
        thresholds: ThresholdConfig {
            max_coupling: 8.0,
            max_orphans: 3,
            max_complexity: 12.0,
            min_health: 85.0,
            custom: HashMap::new(),
        },
        terminology: [
            ("service".to_string(), "microservice".to_string()),
            ("database".to_string(), "data store".to_string()),
        ].into_iter().collect(),
        llm_prompt: Some("Analyze from a platform engineering perspective. Focus on infrastructure efficiency, cost optimization, and platform capabilities.".to_string()),
        focus_areas: vec![
            "resource_utilization".to_string(),
            "cost_efficiency".to_string(),
            "platform_abstractions".to_string(),
        ],
        analysis_depth: AnalysisDepth::Deep,
        exclude: vec!["incident_patterns".to_string()],
    });

    views.insert("tech-lead".to_string(), ViewDefinition {
        name: "Tech Lead Summary".to_string(),
        description: Some("Technical debt and team impact analysis for tech leads".to_string()),
        extends: Some("cto".to_string()),
        sections: vec![
            "tech_debt".to_string(),
            "team_impact".to_string(),
            "code_quality".to_string(),
        ],
        exclude: vec![
            "executive_summary".to_string(),
            "cost_optimization".to_string(),
        ],
        thresholds: ThresholdConfig {
            max_coupling: 6.0,
            max_orphans: 2,
            max_complexity: 10.0,
            min_health: 80.0,
            custom: HashMap::new(),
        },
        terminology: HashMap::new(),
        llm_prompt: Some("Analyze from a tech lead perspective. Focus on code quality, technical debt, and team productivity impact.".to_string()),
        focus_areas: vec![
            "code_maintainability".to_string(),
            "testing_coverage".to_string(),
            "documentation".to_string(),
            "developer_experience".to_string(),
        ],
        analysis_depth: AnalysisDepth::Standard,
    });

    views
}
