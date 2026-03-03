//! View-based analysis system
//!
//! Provides configurable views for architecture analysis with LLM integration

use crate::config::{get_builtin_views, AnalysisDepth, SrujaConfig, ViewDefinition};
use serde::{Deserialize, Serialize};
use sruja_scan::Graph;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewReport {
    pub view_name: String,
    pub view_display_name: String,
    pub summary: ViewSummary,
    pub sections: HashMap<String, serde_json::Value>,
    pub health_score: f32,
    pub recommendations: Vec<ViewRecommendation>,
    pub llm_insights: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewSummary {
    pub total_components: usize,
    pub services_count: usize,
    pub databases_count: usize,
    pub external_apis_count: usize,
    pub coupling_score: f32,
    pub complexity_score: f32,
    pub orphan_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewRecommendation {
    pub priority: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub impact: String,
    pub effort: String,
}

#[derive(Debug, Clone)]
pub struct ViewContext {
    pub view: ViewDefinition,
    pub graph: Graph,
    pub config: SrujaConfig,
    pub repo_path: std::path::PathBuf,
}

impl ViewContext {
    pub fn new(view_name: &str, graph: Graph, repo_path: &Path, config: SrujaConfig) -> Result<Self, String> {
        let view = if let Some(v) = config.get_view(view_name) {
            v
        } else {
            get_builtin_views()
                .get(view_name)
                .cloned()
                .ok_or_else(|| format!("View '{}' not found", view_name))?
        };

        Ok(Self {
            view,
            graph,
            config,
            repo_path: repo_path.to_path_buf(),
        })
    }

    pub async fn analyze(&self) -> Result<ViewReport, String> {
        let summary = self.generate_summary();
        let mut sections = HashMap::new();

        for section_name in &self.view.sections {
            if self.view.exclude.contains(section_name) {
                continue;
            }

            let section_data = self.analyze_section(section_name)?;
            sections.insert(section_name.clone(), section_data);
        }

        let health_score = self.calculate_health_score(&summary);
        let recommendations = self.generate_recommendations(&summary, &sections);
        let llm_insights = self.get_llm_insights(&summary, &sections).await?;

        Ok(ViewReport {
            view_name: self.view.name.clone(),
            view_display_name: self.view.name.clone(),
            summary,
            sections,
            health_score,
            recommendations,
            llm_insights,
        })
    }

    fn generate_summary(&self) -> ViewSummary {
        use sruja_scan::NodeKind;

        let total_components = self.graph.nodes.len();
        let services_count = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Service)
            .count();
        let databases_count = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Database)
            .count();
        let external_apis_count = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::ExternalApi)
            .count();

        let coupling_score = if total_components > 0 {
            self.graph.edges.len() as f32 / total_components as f32
        } else {
            0.0
        };

        let orphan_count = self.count_orphans();
        let complexity_score = self.calculate_complexity_score();

        ViewSummary {
            total_components,
            services_count,
            databases_count,
            external_apis_count,
            coupling_score,
            complexity_score,
            orphan_count,
        }
    }

    fn count_orphans(&self) -> usize {
        let mut has_incoming: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut has_outgoing: std::collections::HashSet<String> = std::collections::HashSet::new();

        for edge in &self.graph.edges {
            has_incoming.insert(edge.target.clone());
            has_outgoing.insert(edge.source.clone());
        }

        self.graph
            .nodes
            .iter()
            .filter(|n| !has_incoming.contains(&n.id) && !has_outgoing.contains(&n.id))
            .count()
    }

    fn calculate_complexity_score(&self) -> f32 {
        let mut scores = Vec::new();

        let mut incoming: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut outgoing: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for edge in &self.graph.edges {
            *incoming.entry(edge.target.clone()).or_insert(0) += 1;
            *outgoing.entry(edge.source.clone()).or_insert(0) += 1;
        }

        for node in &self.graph.nodes {
            let in_degree = incoming.get(&node.id).copied().unwrap_or(0);
            let out_degree = outgoing.get(&node.id).copied().unwrap_or(0);
            let complexity = (in_degree + out_degree) as f32 / 2.0;
            scores.push(complexity);
        }

        if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f32>() / scores.len() as f32
        }
    }

    fn calculate_health_score(&self, summary: &ViewSummary) -> f32 {
        let mut score = 100.0;

        if summary.coupling_score > self.view.thresholds.max_coupling {
            score -= (summary.coupling_score - self.view.thresholds.max_coupling) * 5.0;
        }

        if summary.orphan_count > self.view.thresholds.max_orphans {
            score -= (summary.orphan_count - self.view.thresholds.max_orphans) as f32 * 3.0;
        }

        if summary.complexity_score > self.view.thresholds.max_complexity {
            score -= (summary.complexity_score - self.view.thresholds.max_complexity) * 2.0;
        }

        score.max(0.0).min(100.0)
    }

    fn analyze_section(&self, section_name: &str) -> Result<serde_json::Value, String> {
        match section_name {
            "executive_summary" => self.analyze_executive_summary(),
            "tech_stack" => self.analyze_tech_stack(),
            "tech_debt" => self.analyze_tech_debt(),
            "risks" => self.analyze_risks(),
            "recommendations" => self.analyze_recommendations_section(),
            "infrastructure" => self.analyze_infrastructure(),
            "reliability" => self.analyze_reliability(),
            "dependencies" => self.analyze_dependencies(),
            "single_points_of_failure" => self.analyze_spofs(),
            "cost_optimization" => self.analyze_cost_optimization(),
            "deployment_readiness" => self.analyze_deployment_readiness(),
            "cicd" => self.analyze_cicd(),
            "attack_surface" => self.analyze_attack_surface(),
            "vulnerabilities" => self.analyze_vulnerabilities(),
            "features" => self.analyze_features(),
            "team_impact" => self.analyze_team_impact(),
            "incident_patterns" => self.analyze_incident_patterns(),
            "code_quality" => self.analyze_code_quality(),
            "resource_efficiency" => self.analyze_resource_efficiency(),
            _ => Ok(serde_json::json!({
                "message": format!("Section '{}' analysis not yet implemented", section_name)
            })),
        }
    }

    fn analyze_executive_summary(&self) -> Result<serde_json::Value, String> {
        use sruja_scan::NodeKind;

        let services = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Service)
            .count();
        let databases = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Database)
            .count();
        let external = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::ExternalApi)
            .count();

        let technologies: std::collections::HashSet<String> = self
            .graph
            .nodes
            .iter()
            .filter_map(|n| n.technology.clone())
            .collect();

        Ok(serde_json::json!({
            "architecture_scope": {
                "services": services,
                "databases": databases,
                "external_integrations": external,
                "unique_technologies": technologies.len(),
            },
            "health_indicators": {
                "coupling": if self.graph.edges.len() > self.graph.nodes.len() * 2 { "high" } else { "moderate" },
                "complexity": if self.count_orphans() > 5 { "needs_attention" } else { "acceptable" },
            },
            "key_findings": [
                format!("{} services identified", services),
                format!("{} database dependencies", databases),
                format!("{} external API integrations", external),
            ],
        }))
    }

    fn analyze_tech_stack(&self) -> Result<serde_json::Value, String> {
        let mut tech_counts: HashMap<String, usize> = HashMap::new();

        for node in &self.graph.nodes {
            if let Some(ref tech) = node.technology {
                let tech_name = self.apply_terminology(tech);
                *tech_counts.entry(tech_name).or_insert(0) += 1;
            }
        }

        let mut tech_list: Vec<_> = tech_counts
            .into_iter()
            .map(|(name, count)| {
                let risk = self.assess_tech_risk(&name);
                serde_json::json!({
                    "name": name,
                    "usage_count": count,
                    "risk_level": risk,
                })
            })
            .collect();
        tech_list.sort_by(|a, b| {
            let count_a = a.get("usage_count").and_then(|c| c.as_u64()).unwrap_or(0);
            let count_b = b.get("usage_count").and_then(|c| c.as_u64()).unwrap_or(0);
            count_b.cmp(&count_a)
        });

        Ok(serde_json::json!({
            "technologies": tech_list,
            "total_unique": tech_list.len(),
        }))
    }

    fn analyze_tech_debt(&self) -> Result<serde_json::Value, String> {
        let orphans = self.count_orphans();
        let coupling = if self.graph.nodes.is_empty() {
            0.0
        } else {
            self.graph.edges.len() as f32 / self.graph.nodes.len() as f32
        };

        let mut hotspots = Vec::new();

        if coupling > self.view.thresholds.max_coupling {
            hotspots.push(serde_json::json!({
                "area": "High Coupling",
                "description": "Components have excessive dependencies",
                "score": coupling,
                "threshold": self.view.thresholds.max_coupling,
            }));
        }

        if orphans > self.view.thresholds.max_orphans {
            hotspots.push(serde_json::json!({
                "area": "Orphaned Code",
                "description": format!("{} components appear unused", orphans),
                "score": orphans,
                "threshold": self.view.thresholds.max_orphans,
            }));
        }

        let debt_score = (orphans as f32 * 0.5 + coupling * 3.0).min(100.0);

        Ok(serde_json::json!({
            "debt_score": debt_score,
            "hotspots": hotspots,
            "estimated_remediation_weeks": (debt_score / 10.0).ceil() as i32,
        }))
    }

    fn analyze_risks(&self) -> Result<serde_json::Value, String> {
        use sruja_scan::NodeKind;

        let mut risks = Vec::new();

        let db_count = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Database)
            .count();
        if db_count > 5 {
            risks.push(serde_json::json!({
                "category": "Data Architecture",
                "severity": "Medium",
                "description": format!("High number of {} data stores may indicate data silos", db_count),
            }));
        }

        let external_count = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::ExternalApi)
            .count();
        if external_count > 10 {
            risks.push(serde_json::json!({
                "category": "External Dependencies",
                "severity": "High",
                "description": format!("Heavy reliance on {} external services", external_count),
            }));
        }

        let orphans = self.count_orphans();
        if orphans > 5 {
            risks.push(serde_json::json!({
                "category": "Code Quality",
                "severity": "Low",
                "description": format!("{} orphaned components detected", orphans),
            }));
        }

        Ok(serde_json::json!({
            "risks": risks,
            "risk_count": risks.len(),
        }))
    }

    fn analyze_recommendations_section(&self) -> Result<serde_json::Value, String> {
        let recs = self.generate_recommendations(&self.generate_summary(), &HashMap::new());
        Ok(serde_json::json!({
            "recommendations": recs,
        }))
    }

    fn analyze_infrastructure(&self) -> Result<serde_json::Value, String> {
        use sruja_scan::NodeKind;

        let has_docker = self.repo_path.join("Dockerfile").exists()
            || self.repo_path.join("docker-compose.yml").exists();
        let has_iac = self.check_for_iac();

        Ok(serde_json::json!({
            "containerization": has_docker,
            "infrastructure_as_code": has_iac,
            "databases": self.graph.nodes.iter().filter(|n| n.kind == NodeKind::Database).count(),
            "external_services": self.graph.nodes.iter().filter(|n| n.kind == NodeKind::ExternalApi).count(),
        }))
    }

    fn analyze_reliability(&self) -> Result<serde_json::Value, String> {
        let spofs = self.identify_spofs();
        let coupling = if self.graph.nodes.is_empty() {
            0.0
        } else {
            self.graph.edges.len() as f32 / self.graph.nodes.len() as f32
        };

        let blast_radius = if coupling > 5.0 {
            "high"
        } else if coupling > 2.0 {
            "medium"
        } else {
            "low"
        };

        Ok(serde_json::json!({
            "coupling_score": coupling,
            "blast_radius": blast_radius,
            "single_points_of_failure": spofs.len(),
            "availability_risk": if spofs.len() > 5 { "high" } else if spofs.len() > 2 { "medium" } else { "low" },
        }))
    }

    fn analyze_dependencies(&self) -> Result<serde_json::Value, String> {
        let high_coupling = self.find_high_coupling_components();

        Ok(serde_json::json!({
            "total_dependencies": self.graph.edges.len(),
            "high_coupling_components": high_coupling,
            "dependency_density": if self.graph.nodes.is_empty() {
                0.0
            } else {
                self.graph.edges.len() as f32 / self.graph.nodes.len() as f32
            },
        }))
    }

    fn analyze_spofs(&self) -> Result<serde_json::Value, String> {
        let spofs = self.identify_spofs();
        Ok(serde_json::json!({
            "single_points_of_failure": spofs,
            "total_spofs": spofs.len(),
        }))
    }

    fn analyze_cost_optimization(&self) -> Result<serde_json::Value, String> {
        let mut opportunities = Vec::new();

        let orphans = self.count_orphans();
        if orphans > 0 {
            opportunities.push(serde_json::json!({
                "category": "Unused Resources",
                "description": format!("{} orphaned components could be removed", orphans),
                "estimated_savings": "low",
            }));
        }

        let coupling = if self.graph.nodes.is_empty() {
            0.0
        } else {
            self.graph.edges.len() as f32 / self.graph.nodes.len() as f32
        };

        if coupling > 5.0 {
            opportunities.push(serde_json::json!({
                "category": "Over-provisioned Dependencies",
                "description": "High coupling suggests potential for service consolidation",
                "estimated_savings": "medium",
            }));
        }

        Ok(serde_json::json!({
            "optimization_opportunities": opportunities,
        }))
    }

    fn analyze_deployment_readiness(&self) -> Result<serde_json::Value, String> {
        let has_docker = self.repo_path.join("Dockerfile").exists();
        let has_ci = self.repo_path.join(".github/workflows").exists();
        let has_tests = self.check_for_tests();

        let mut blockers = Vec::new();
        if !has_docker {
            blockers.push("No containerization detected");
        }
        if !has_ci {
            blockers.push("No CI/CD configuration found");
        }
        if !has_tests {
            blockers.push("No test infrastructure detected");
        }

        let score = (if has_docker { 30.0 } else { 0.0 })
            + (if has_ci { 30.0 } else { 0.0 })
            + (if has_tests { 25.0 } else { 0.0 })
            + 15.0;

        Ok(serde_json::json!({
            "readiness_score": score,
            "blockers": blockers,
            "has_containers": has_docker,
            "has_ci": has_ci,
            "has_tests": has_tests,
        }))
    }

    fn analyze_cicd(&self) -> Result<serde_json::Value, String> {
        let has_github_actions = self.repo_path.join(".github/workflows").exists();
        let has_gitlab_ci = self.repo_path.join(".gitlab-ci.yml").exists();
        let has_jenkins = self.repo_path.join("Jenkinsfile").exists();

        Ok(serde_json::json!({
            "github_actions": has_github_actions,
            "gitlab_ci": has_gitlab_ci,
            "jenkins": has_jenkins,
            "has_ci": has_github_actions || has_gitlab_ci || has_jenkins,
        }))
    }

    fn analyze_attack_surface(&self) -> Result<serde_json::Value, String> {
        use sruja_scan::NodeKind;

        let external_apis = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::ExternalApi)
            .count();
        let services = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Service)
            .count();
        let databases = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Database)
            .count();

        let risk_level = if external_apis > 10 {
            "high"
        } else if external_apis > 5 {
            "medium"
        } else {
            "low"
        };

        Ok(serde_json::json!({
            "entry_points": services,
            "external_apis": external_apis,
            "databases": databases,
            "risk_level": risk_level,
        }))
    }

    fn analyze_vulnerabilities(&self) -> Result<serde_json::Value, String> {
        let mut vulnerabilities = Vec::new();

        let has_auth = self.check_for_auth();
        if !has_auth {
            vulnerabilities.push(serde_json::json!({
                "category": "Authentication",
                "severity": "High",
                "description": "No authentication mechanisms detected",
            }));
        }

        let has_encryption = self.check_for_encryption();
        if !has_encryption {
            vulnerabilities.push(serde_json::json!({
                "category": "Encryption",
                "severity": "Medium",
                "description": "No encryption configuration detected",
            }));
        }

        Ok(serde_json::json!({
            "vulnerabilities": vulnerabilities,
            "total": vulnerabilities.len(),
        }))
    }

    fn analyze_features(&self) -> Result<serde_json::Value, String> {
        let features = self.detect_features();

        Ok(serde_json::json!({
            "features_detected": features,
            "total_modules": self.graph.nodes.len(),
        }))
    }

    fn analyze_team_impact(&self) -> Result<serde_json::Value, String> {
        let shared_components = self.identify_shared_components();

        Ok(serde_json::json!({
            "shared_components": shared_components,
            "coordination_needed": shared_components.len() > 3,
        }))
    }

    fn analyze_incident_patterns(&self) -> Result<serde_json::Value, String> {
        let spofs = self.identify_spofs();

        Ok(serde_json::json!({
            "potential_incident_sources": spofs.len(),
            "blast_radius": if spofs.len() > 5 { "high" } else { "moderate" },
        }))
    }

    fn analyze_code_quality(&self) -> Result<serde_json::Value, String> {
        let orphans = self.count_orphans();
        let coupling = if self.graph.nodes.is_empty() {
            0.0
        } else {
            self.graph.edges.len() as f32 / self.graph.nodes.len() as f32
        };

        Ok(serde_json::json!({
            "orphaned_components": orphans,
            "coupling_score": coupling,
            "maintainability_index": (100.0 - coupling * 5.0 - orphans as f32 * 2.0).max(0.0),
        }))
    }

    fn analyze_resource_efficiency(&self) -> Result<serde_json::Value, String> {
        let orphans = self.count_orphans();

        Ok(serde_json::json!({
            "unused_components": orphans,
            "efficiency_score": if self.graph.nodes.is_empty() {
                100.0
            } else {
                ((self.graph.nodes.len() - orphans) as f32 / self.graph.nodes.len() as f32) * 100.0
            },
        }))
    }

    fn generate_recommendations(
        &self,
        summary: &ViewSummary,
        _sections: &HashMap<String, serde_json::Value>,
    ) -> Vec<ViewRecommendation> {
        let mut recs = Vec::new();

        if summary.orphan_count > self.view.thresholds.max_orphans {
            recs.push(ViewRecommendation {
                priority: "High".to_string(),
                category: "Code Quality".to_string(),
                title: "Remove Orphaned Components".to_string(),
                description: format!(
                    "Review and remove {} unused components",
                    summary.orphan_count
                ),
                impact: "Reduces maintenance burden".to_string(),
                effort: "1-2 weeks".to_string(),
            });
        }

        if summary.coupling_score > self.view.thresholds.max_coupling {
            recs.push(ViewRecommendation {
                priority: "High".to_string(),
                category: "Architecture".to_string(),
                title: "Reduce Coupling".to_string(),
                description: "Introduce abstraction layers to reduce tight coupling".to_string(),
                impact: "Enables independent deployment".to_string(),
                effort: "3-6 weeks".to_string(),
            });
        }

        if summary.complexity_score > self.view.thresholds.max_complexity {
            recs.push(ViewRecommendation {
                priority: "Medium".to_string(),
                category: "Refactoring".to_string(),
                title: "Simplify Complex Components".to_string(),
                description: "Break down highly complex components".to_string(),
                impact: "Improves maintainability".to_string(),
                effort: "2-4 weeks".to_string(),
            });
        }

        recs
    }

    async fn get_llm_insights(
        &self,
        summary: &ViewSummary,
        sections: &HashMap<String, serde_json::Value>,
    ) -> Result<Option<String>, String> {
        if !self.config.defaults.enable_llm {
            return Ok(None);
        }

        if let Some(ref prompt_template) = self.view.llm_prompt {
            let context = self.build_llm_context(summary, sections);
            let full_prompt = format!("{}\n\n{}", prompt_template, context);

            let system_prompt = "You are an expert Software Architect analyzing a codebase structure and providing insights tailored for specific stakeholder views. Produce a highly insightful and concise architectural summary, identifying real hotspots and structural bottlenecks based strictly on the metrics provided.";
            match crate::commands::llm::call_llm(system_prompt, &full_prompt).await {
                Ok(response) => Ok(Some(response)),
                Err(_) => {
                    match self.view.analysis_depth {
                        AnalysisDepth::Quick => Ok(Some(self.generate_quick_insight(&full_prompt))),
                        AnalysisDepth::Standard => Ok(Some(self.generate_standard_insight(&full_prompt))),
                        AnalysisDepth::Deep => Ok(Some(self.generate_deep_insight(&full_prompt))),
                        AnalysisDepth::Comprehensive => {
                            Ok(Some(self.generate_comprehensive_insight(&full_prompt)))
                        }
                    }
                }
            }
        } else {
            Ok(None)
        }
    }

    fn build_llm_context(
        &self,
        summary: &ViewSummary,
        sections: &HashMap<String, serde_json::Value>,
    ) -> String {
        let mut context = String::new();

        context.push_str(&"Architecture Summary:\n".to_string());
        context.push_str(&format!(
            "- Total components: {}\n",
            summary.total_components
        ));
        context.push_str(&format!("- Services: {}\n", summary.services_count));
        context.push_str(&format!("- Databases: {}\n", summary.databases_count));
        context.push_str(&format!(
            "- External APIs: {}\n",
            summary.external_apis_count
        ));
        context.push_str(&format!(
            "- Coupling score: {:.2}\n",
            summary.coupling_score
        ));
        context.push_str(&format!(
            "- Orphaned components: {}\n",
            summary.orphan_count
        ));
        context.push('\n');

        for (section_name, section_data) in sections {
            context.push_str(&format!("Section: {}\n", section_name));
            context.push_str(&serde_json::to_string_pretty(section_data).unwrap_or_default());
            context.push_str("\n\n");
        }

        context
    }

    fn generate_quick_insight(&self, _prompt: &str) -> String {
        format!(
            "Quick analysis complete. Health score: {:.0}%. Focus on {} high-priority items.",
            self.calculate_health_score(&self.generate_summary()),
            if self.count_orphans() > 5 { 2 } else { 1 }
        )
    }

    fn generate_standard_insight(&self, _prompt: &str) -> String {
        let summary = self.generate_summary();
        let health = self.calculate_health_score(&summary);

        format!(
            "Standard analysis identifies {} components with {:.0}% health score. \
             Key concerns: coupling ({:.1}), orphans ({}). \
             Recommendations: {} items requiring attention.",
            summary.total_components,
            health,
            summary.coupling_score,
            summary.orphan_count,
            if health < 70.0 {
                3
            } else if health < 85.0 {
                2
            } else {
                1
            }
        )
    }

    fn generate_deep_insight(&self, _prompt: &str) -> String {
        let summary = self.generate_summary();
        let health = self.calculate_health_score(&summary);

        format!(
            "Deep analysis of {} components reveals {:.0}% architectural health. \
             Coupling analysis: {:.1} (threshold: {:.1}). \
             Complexity distribution suggests {} hotspots. \
             Recommended focus areas: {}, {}. \
             Estimated remediation: {} weeks.",
            summary.total_components,
            health,
            summary.coupling_score,
            self.view.thresholds.max_coupling,
            if summary.complexity_score > 10.0 {
                3
            } else {
                1
            },
            self.view
                .focus_areas
                .first()
                .unwrap_or(&"coupling".to_string()),
            self.view
                .focus_areas
                .get(1)
                .unwrap_or(&"quality".to_string()),
            ((100.0 - health) / 10.0).ceil() as i32
        )
    }

    fn generate_comprehensive_insight(&self, _prompt: &str) -> String {
        let summary = self.generate_summary();
        let health = self.calculate_health_score(&summary);
        let spofs = self.identify_spofs();

        format!(
            "Comprehensive architectural analysis:\n\n\
             Overall Health: {:.0}%\n\n\
             Component Distribution:\n\
             - {} services across {} total components\n\
             - {} data stores with {} external integrations\n\n\
             Quality Metrics:\n\
             - Coupling Index: {:.1} (threshold: {:.1})\n\
             - Complexity Score: {:.1}\n\
             - Orphaned Components: {}\n\
             - Single Points of Failure: {}\n\n\
             Strategic Recommendations:\n\
             1. Address {} high-priority coupling issues\n\
             2. Review {} orphaned components for removal\n\
             3. Implement redundancy for {} critical components\n\n\
             Focus Areas: {}\n\n\
             Estimated Remediation Timeline: {} weeks",
            health,
            summary.services_count,
            summary.total_components,
            summary.databases_count,
            summary.external_apis_count,
            summary.coupling_score,
            self.view.thresholds.max_coupling,
            summary.complexity_score,
            summary.orphan_count,
            spofs.len(),
            if summary.coupling_score > self.view.thresholds.max_coupling {
                2
            } else {
                0
            },
            summary.orphan_count,
            spofs.len(),
            self.view.focus_areas.join(", "),
            ((100.0 - health) / 10.0).ceil() as i32
        )
    }

    fn apply_terminology(&self, term: &str) -> String {
        self.view
            .terminology
            .get(term)
            .cloned()
            .unwrap_or_else(|| term.to_string())
    }

    fn assess_tech_risk(&self, tech: &str) -> String {
        let tech_lower = tech.to_lowercase();
        if ["cobol", "fortran", "perl", "vb"]
            .iter()
            .any(|t| tech_lower.contains(t))
        {
            "High".to_string()
        } else if ["php", "ruby", "java"]
            .iter()
            .any(|t| tech_lower.contains(t))
        {
            "Medium".to_string()
        } else {
            "Low".to_string()
        }
    }

    fn check_for_iac(&self) -> bool {
        self.repo_path.join("terraform").exists()
            || self.repo_path.join("kubernetes").exists()
            || self.repo_path.join("helm").exists()
            || self.repo_path.join("ansible").exists()
    }

    fn check_for_tests(&self) -> bool {
        self.repo_path.join("tests").exists()
            || self.repo_path.join("test").exists()
            || self.repo_path.join("__tests__").exists()
    }

    fn check_for_auth(&self) -> bool {
        self.repo_path.join("auth").exists()
            || self.repo_path.join("middleware").exists()
            || std::fs::read_dir(&self.repo_path)
                .map(|entries| {
                    entries.filter_map(Result::ok).any(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .to_lowercase()
                            .contains("auth")
                    })
                })
                .unwrap_or(false)
    }

    fn check_for_encryption(&self) -> bool {
        std::fs::read_dir(&self.repo_path)
            .map(|entries| {
                entries.filter_map(Result::ok).any(|e| {
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    name.contains("ssl") || name.contains("tls") || name.contains("cert")
                })
            })
            .unwrap_or(false)
    }

    fn identify_spofs(&self) -> Vec<serde_json::Value> {
        let mut incoming: HashMap<String, usize> = HashMap::new();

        for edge in &self.graph.edges {
            *incoming.entry(edge.target.clone()).or_insert(0) += 1;
        }

        self.graph
            .nodes
            .iter()
            .filter(|n| incoming.get(&n.id).copied().unwrap_or(0) > 3)
            .map(|n| {
                serde_json::json!({
                    "component": n.label,
                    "kind": format!("{:?}", n.kind),
                    "downstream_impact": incoming.get(&n.id).copied().unwrap_or(0),
                })
            })
            .collect()
    }

    fn find_high_coupling_components(&self) -> Vec<String> {
        let mut incoming: HashMap<String, usize> = HashMap::new();
        let mut outgoing: HashMap<String, usize> = HashMap::new();

        for edge in &self.graph.edges {
            *incoming.entry(edge.target.clone()).or_insert(0) += 1;
            *outgoing.entry(edge.source.clone()).or_insert(0) += 1;
        }

        self.graph
            .nodes
            .iter()
            .filter(|n| {
                incoming.get(&n.id).copied().unwrap_or(0) > 5
                    || outgoing.get(&n.id).copied().unwrap_or(0) > 5
            })
            .map(|n| n.label.clone())
            .collect()
    }

    fn detect_features(&self) -> Vec<String> {
        let mut features = std::collections::HashSet::new();

        for node in &self.graph.nodes {
            let label_lower = node.label.to_lowercase();
            if label_lower.contains("user") {
                features.insert("User Management".to_string());
            }
            if label_lower.contains("auth") || label_lower.contains("login") {
                features.insert("Authentication".to_string());
            }
            if label_lower.contains("payment") || label_lower.contains("checkout") {
                features.insert("Payment Processing".to_string());
            }
            if label_lower.contains("order") {
                features.insert("Order Management".to_string());
            }
            if label_lower.contains("notification") || label_lower.contains("email") {
                features.insert("Notifications".to_string());
            }
        }

        features.into_iter().collect()
    }

    fn identify_shared_components(&self) -> Vec<String> {
        let mut usage_count: HashMap<String, usize> = HashMap::new();

        for edge in &self.graph.edges {
            *usage_count.entry(edge.target.clone()).or_insert(0) += 1;
        }

        self.graph
            .nodes
            .iter()
            .filter(|n| usage_count.get(&n.id).copied().unwrap_or(0) > 2)
            .map(|n| n.label.clone())
            .collect()
    }
}

pub fn print_view_report(report: &ViewReport, format: &str) {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(report).unwrap());
        }
        _ => {
            print_text_report(report);
        }
    }
}

fn print_text_report(report: &ViewReport) {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║ {:^64} ║", report.view_display_name);
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ 📊 SUMMARY                                                       │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!(
        "│ Total Components:  {:>44} │",
        report.summary.total_components
    );
    println!(
        "│ Services:          {:>44} │",
        report.summary.services_count
    );
    println!(
        "│ Databases:         {:>44} │",
        report.summary.databases_count
    );
    println!(
        "│ External APIs:     {:>44} │",
        report.summary.external_apis_count
    );
    println!(
        "│ Coupling Score:    {:>44.2} │",
        report.summary.coupling_score
    );
    println!("│ Health Score:      {:>43.0}% │", report.health_score);
    println!("└──────────────────────────────────────────────────────────────────┘");
    println!();

    for (section_name, section_data) in &report.sections {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ 📋 {} {:>58} │", to_title_case(section_name), "");
        println!("├──────────────────────────────────────────────────────────────────┤");
        print_section_data(section_data);
        println!("└──────────────────────────────────────────────────────────────────┘");
        println!();
    }

    if !report.recommendations.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ 💡 RECOMMENDATIONS                                               │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for rec in &report.recommendations {
            println!("│ [{}] {}", rec.priority, rec.title);
            println!("│     {}", rec.description);
            println!("│     Effort: {} | Impact: {}", rec.effort, rec.impact);
            println!("│");
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
        println!();
    }

    if let Some(ref insights) = report.llm_insights {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ 🤖 AI INSIGHTS                                                   │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for line in insights.lines() {
            println!("│ {}", line);
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
    }
}

fn print_section_data(data: &serde_json::Value) {
    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            match value {
                serde_json::Value::Number(n) => {
                    println!("│ {}: {:>48} │", to_title_case(key), n);
                }
                serde_json::Value::String(s) => {
                    println!("│ {}: {:>48} │", to_title_case(key), s);
                }
                serde_json::Value::Bool(b) => {
                    println!(
                        "│ {}: {:>48} │",
                        to_title_case(key),
                        if *b { "✅" } else { "❌" }
                    );
                }
                serde_json::Value::Array(arr) => {
                    println!(
                        "│ {}: {:>48} │",
                        to_title_case(key),
                        format!("{} items", arr.len())
                    );
                }
                _ => {
                    println!("│ {}: {:>48} │", to_title_case(key), "-");
                }
            }
        }
    }
}

fn to_title_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
