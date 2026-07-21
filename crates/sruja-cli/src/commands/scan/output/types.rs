use sruja_scan::scan_scope::ScanScope;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct QuickstartResult {
    pub repo: String,
    pub scan_scope: ScanScope,
    pub health_score: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_breakdown: Option<sruja_diff::HealthScoreBreakdown>,
    pub inventory: InventorySummary,
    pub top_findings: Vec<Finding>,
    pub actionable_fixes: Vec<ActionableFix>,
    pub truth_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_quality: Option<ScanQuality>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScanQuality {
    pub confidence_score: u8,
    pub coverage_percent: u8,
    pub manifest_discoveries: usize,
    pub entry_point_count: usize,
    pub leaf_node_count: usize,
    pub orphan_count: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InventorySummary {
    pub modules: usize,
    pub services: usize,
    pub databases: usize,
    pub external_apis: usize,
    pub total_dependencies: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    pub severity: String,
    pub kind: String,
    pub message: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ActionableFix {
    pub priority: String,
    pub description: String,
    pub impact: String,
    pub affected_components: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StatusOutput {
    pub baseline: Option<String>,
    pub truth_status: String,
    pub violations_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_score: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_score: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub health_history: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub velocity: Option<sruja_diff::ArchitecturalVelocity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_memory: Option<crate::utils::agent_memory_signal::AgentMemorySignal>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PrDriftResult {
    pub base_ref: String,
    pub head_ref: String,
    pub changed_files: Vec<String>,
    pub base_health: u8,
    pub head_health: u8,
    pub new_violations: Vec<PrViolation>,
    pub base_violations_count: usize,
    pub head_violations_count: usize,
    pub component_diffs: Vec<sruja_diff::ComponentDiff>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PrViolation {
    pub severity: String,
    pub kind: String,
    pub message: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftBaselineSkip {
    ReviewedBaselineExists,
    DraftExists,
}
