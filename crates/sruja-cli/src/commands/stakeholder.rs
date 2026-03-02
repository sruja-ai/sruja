//! Stakeholder-specific commands: cto, sre, devops, security, product.
//!
//! These commands provide tailored views of architecture analysis for different
//! stakeholders in an organization.

use std::path::Path;

use sruja_scan::{scan_repo, Graph, NodeKind};
use serde::{Deserialize, Serialize};

use super::CliError;

#[derive(Debug, Serialize, Deserialize)]
pub struct CtoReport {
    pub summary: CtoSummary,
    pub tech_stack: Vec<TechStackItem>,
    pub tech_debt: TechDebtAssessment,
    pub risks: Vec<RiskItem>,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CtoSummary {
    pub total_components: usize,
    pub services_count: usize,
    pub databases_count: usize,
    pub external_apis_count: usize,
    pub health_score: f32,
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TechStackItem {
    pub name: String,
    pub category: String,
    pub usage_count: usize,
    pub risk_level: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TechDebtAssessment {
    pub score: f32,
    pub estimated_remediation_weeks: i32,
    pub hotspots: Vec<TechDebtHotspot>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TechDebtHotspot {
    pub area: String,
    pub description: String,
    pub impact: String,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskItem {
    pub category: String,
    pub severity: String,
    pub description: String,
    pub affected_components: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: String,
    pub title: String,
    pub description: String,
    pub estimated_effort: String,
    pub impact: String,
}

pub async fn cto(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;

    let report = generate_cto_report(&graph);

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        _ => {
            print_cto_report(&report);
        }
    }

    Ok(())
}

fn generate_cto_report(graph: &Graph) -> CtoReport {
    let total_components = graph.nodes.len();
    let services_count = graph.nodes.iter().filter(|n| n.kind == NodeKind::Service).count();
    let databases_count = graph.nodes.iter().filter(|n| n.kind == NodeKind::Database).count();
    let external_apis_count = graph.nodes.iter().filter(|n| n.kind == NodeKind::ExternalApi).count();

    let languages = extract_technologies(&graph.nodes, "language");
    let frameworks = extract_technologies(&graph.nodes, "framework");

    let health_score = calculate_health_score(graph);
    let tech_debt = assess_tech_debt(graph);
    let risks = identify_risks(graph);
    let recommendations = generate_recommendations(graph);

    CtoReport {
        summary: CtoSummary {
            total_components,
            services_count,
            databases_count,
            external_apis_count,
            health_score,
            languages,
            frameworks,
        },
        tech_stack: extract_tech_stack(graph),
        tech_debt,
        risks,
        recommendations,
    }
}

fn extract_technologies(nodes: &[sruja_scan::Node], _category: &str) -> Vec<String> {
    let mut techs: std::collections::HashSet<String> = std::collections::HashSet::new();
    for node in nodes {
        if let Some(ref tech) = node.technology {
            techs.insert(tech.clone());
        }
    }
    let mut v: Vec<_> = techs.into_iter().collect();
    v.sort();
    v
}

fn extract_tech_stack(graph: &Graph) -> Vec<TechStackItem> {
    let mut tech_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for node in &graph.nodes {
        if let Some(ref tech) = node.technology {
            *tech_counts.entry(tech.clone()).or_insert(0) += 1;
        }
    }

    let mut items: Vec<TechStackItem> = tech_counts
        .into_iter()
        .map(|(name, count)| TechStackItem {
            category: categorize_tech(&name),
            name: name.clone(),
            usage_count: count,
            risk_level: assess_tech_risk(&name),
        })
        .collect();
    items.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
    items.truncate(20);
    items
}

fn categorize_tech(tech: &str) -> String {
    let tech_lower = tech.to_lowercase();
    if ["javascript", "typescript", "python", "go", "rust", "java", "c#", "ruby", "php", "kotlin", "scala", "c", "c++"].iter().any(|l| tech_lower.contains(l)) {
        return "Language".to_string();
    }
    if ["react", "vue", "angular", "svelte", "next", "nuxt"].iter().any(|l| tech_lower.contains(l)) {
        return "Frontend Framework".to_string();
    }
    if ["express", "fastapi", "django", "rails", "spring", "aspnet", "actix", "axum", "gin", "echo"].iter().any(|l| tech_lower.contains(l)) {
        return "Backend Framework".to_string();
    }
    if ["postgres", "mysql", "mongodb", "redis", "elasticsearch", "kafka", "rabbitmq", "cassandra", "dynamodb"].iter().any(|l| tech_lower.contains(l)) {
        return "Data Store".to_string();
    }
    if ["kubernetes", "docker", "terraform", "helm", "aws", "gcp", "azure"].iter().any(|l| tech_lower.contains(l)) {
        return "Infrastructure".to_string();
    }
    "Other".to_string()
}

fn assess_tech_risk(tech: &str) -> String {
    let tech_lower = tech.to_lowercase();
    if ["cobol", "fortran", "perl", "vb"].iter().any(|l| tech_lower.contains(l)) {
        return "High".to_string();
    }
    if ["php", "ruby", "java"].iter().any(|l| tech_lower.contains(l)) {
        return "Medium".to_string();
    }
    "Low".to_string()
}

fn calculate_health_score(graph: &Graph) -> f32 {
    let total = graph.nodes.len() as f32;
    if total == 0.0 {
        return 100.0;
    }

    let edges = graph.edges.len() as f32;
    let coupling_ratio = edges / total;

    let cycles = detect_cycles_count(graph);
    let orphans = count_orphans(graph);

    let base_score = 100.0;
    let coupling_penalty = (coupling_ratio * 2.0).min(20.0);
    let cycle_penalty = (cycles as f32 * 5.0).min(30.0);
    let orphan_penalty = (orphans as f32 * 2.0).min(20.0);

    (base_score - coupling_penalty - cycle_penalty - orphan_penalty).max(0.0)
}

fn detect_cycles_count(_graph: &Graph) -> usize {
    0
}

fn count_orphans(graph: &Graph) -> usize {
    let mut has_incoming: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut has_outgoing: std::collections::HashSet<String> = std::collections::HashSet::new();

    for edge in &graph.edges {
        has_incoming.insert(edge.target.clone());
        has_outgoing.insert(edge.source.clone());
    }

    graph
        .nodes
        .iter()
        .filter(|n| !has_incoming.contains(&n.id) && !has_outgoing.contains(&n.id))
        .count()
}

fn assess_tech_debt(graph: &Graph) -> TechDebtAssessment {
    let orphans = count_orphans(graph);
    let cycles = detect_cycles_count(graph);
    let coupling = graph.edges.len() as f32 / graph.nodes.len().max(1) as f32;

    let debt_score = (orphans as f32 * 0.5 + cycles as f32 * 2.0 + coupling * 3.0).min(100.0);
    let remediation_weeks = ((debt_score / 10.0).ceil() as i32).max(1);

    let mut hotspots = Vec::new();

    if coupling > 5.0 {
        hotspots.push(TechDebtHotspot {
            area: "High Coupling".to_string(),
            description: "Components have excessive dependencies".to_string(),
            impact: "High".to_string(),
            files: vec![],
        });
    }

    if orphans > 5 {
        hotspots.push(TechDebtHotspot {
            area: "Orphaned Code".to_string(),
            description: format!("{} modules appear unused", orphans),
            impact: "Medium".to_string(),
            files: vec![],
        });
    }

    TechDebtAssessment {
        score: debt_score,
        estimated_remediation_weeks: remediation_weeks,
        hotspots,
    }
}

fn identify_risks(graph: &Graph) -> Vec<RiskItem> {
    let mut risks = Vec::new();

    let db_count = graph.nodes.iter().filter(|n| n.kind == NodeKind::Database).count();
    if db_count > 5 {
        risks.push(RiskItem {
            category: "Data Architecture".to_string(),
            severity: "Medium".to_string(),
            description: format!("High number of databases ({}) may indicate data silos", db_count),
            affected_components: db_count,
        });
    }

    let external_count = graph.nodes.iter().filter(|n| n.kind == NodeKind::ExternalApi).count();
    if external_count > 10 {
        risks.push(RiskItem {
            category: "External Dependencies".to_string(),
            severity: "High".to_string(),
            description: format!("Heavy reliance on {} external services", external_count),
            affected_components: external_count,
        });
    }

    let cycle_count = detect_cycles_count(graph);
    if cycle_count > 0 {
        risks.push(RiskItem {
            category: "Architecture".to_string(),
            severity: "High".to_string(),
            description: format!("{} circular dependencies detected", cycle_count),
            affected_components: cycle_count,
        });
    }

    risks
}

fn generate_recommendations(graph: &Graph) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    let orphans = count_orphans(graph);
    if orphans > 3 {
        recs.push(Recommendation {
            priority: "High".to_string(),
            title: "Remove Orphaned Modules".to_string(),
            description: format!("Review and remove {} unused modules", orphans),
            estimated_effort: "1-2 weeks".to_string(),
            impact: "Reduces maintenance burden and technical debt".to_string(),
        });
    }

    let cycles = detect_cycles_count(graph);
    if cycles > 0 {
        recs.push(Recommendation {
            priority: "High".to_string(),
            title: "Break Circular Dependencies".to_string(),
            description: format!("Resolve {} circular dependency chains", cycles),
            estimated_effort: "2-4 weeks".to_string(),
            impact: "Improves maintainability and testability".to_string(),
        });
    }

    let coupling = graph.edges.len() as f32 / graph.nodes.len().max(1) as f32;
    if coupling > 3.0 {
        recs.push(Recommendation {
            priority: "Medium".to_string(),
            title: "Reduce Coupling".to_string(),
            description: "Introduce abstraction layers to reduce tight coupling".to_string(),
            estimated_effort: "3-6 weeks".to_string(),
            impact: "Enables independent deployment and testing".to_string(),
        });
    }

    recs
}

fn print_cto_report(report: &CtoReport) {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                 📊 EXECUTIVE ARCHITECTURE REPORT                  ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ 📈 SUMMARY                                                       │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Total Components: {:>46} │", report.summary.total_components);
    println!("│ Services:         {:>46} │", report.summary.services_count);
    println!("│ Databases:        {:>46} │", report.summary.databases_count);
    println!("│ External APIs:    {:>46} │", report.summary.external_apis_count);
    println!("│ Health Score:     {:>45.1}% │", report.summary.health_score);
    println!("└──────────────────────────────────────────────────────────────────┘");
    println!();

    if !report.summary.languages.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ 💻 TECHNOLOGY STACK                                              │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        println!("│ Languages: {}", report.summary.languages.join(", "));
        println!("│ Frameworks: {}", report.summary.frameworks.join(", "));
        println!("└──────────────────────────────────────────────────────────────────┘");
        println!();
    }

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ 🔧 TECH DEBT ASSESSMENT                                          │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Debt Score:        {:>42.1}/100 │", report.tech_debt.score);
    println!("│ Remediation Time:  {:>36} weeks │", report.tech_debt.estimated_remediation_weeks);
    for hotspot in &report.tech_debt.hotspots {
        println!("│ ⚠ {} - {}", hotspot.area, hotspot.description);
    }
    println!("└──────────────────────────────────────────────────────────────────┘");
    println!();

    if !report.risks.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ ⚠️ RISKS                                                         │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for risk in &report.risks {
            println!("│ [{}] {} - {}", risk.severity, risk.category, risk.description);
        }
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
            println!("│     Effort: {} | Impact: {}", rec.estimated_effort, rec.impact);
            println!("│");
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SreReport {
    pub reliability: ReliabilityMetrics,
    pub dependencies: DependencyAnalysis,
    pub single_points_of_failure: Vec<SpofItem>,
    pub recommendations: Vec<SreRecommendation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReliabilityMetrics {
    pub total_dependencies: usize,
    pub external_dependencies: usize,
    pub database_dependencies: usize,
    pub coupling_score: f32,
    pub blast_radius: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub critical_paths: Vec<CriticalPath>,
    pub high_coupling_components: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CriticalPath {
    pub path: Vec<String>,
    pub risk: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpofItem {
    pub component: String,
    pub kind: String,
    pub downstream_impact: usize,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SreRecommendation {
    pub category: String,
    pub priority: String,
    pub description: String,
}

pub async fn sre(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;

    let report = generate_sre_report(&graph);

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        _ => {
            print_sre_report(&report);
        }
    }

    Ok(())
}

fn generate_sre_report(graph: &Graph) -> SreReport {
    let total_deps = graph.edges.len();
    let external_deps = graph.edges.iter().filter(|e| {
        graph.nodes.iter().any(|n| n.id == e.target && n.kind == NodeKind::ExternalApi)
    }).count();
    let db_deps = graph.edges.iter().filter(|e| {
        graph.nodes.iter().any(|n| n.id == e.target && n.kind == NodeKind::Database)
    }).count();

    let coupling_score = total_deps as f32 / graph.nodes.len().max(1) as f32;
    let blast_radius = if coupling_score > 5.0 { "High" } else if coupling_score > 2.0 { "Medium" } else { "Low" };

    let critical_paths = find_critical_paths(graph);
    let high_coupling = find_high_coupling_components(graph);
    let spofs = identify_spofs(graph);
    let recommendations = generate_sre_recommendations(graph, &spofs);

    SreReport {
        reliability: ReliabilityMetrics {
            total_dependencies: total_deps,
            external_dependencies: external_deps,
            database_dependencies: db_deps,
            coupling_score,
            blast_radius: blast_radius.to_string(),
        },
        dependencies: DependencyAnalysis {
            critical_paths,
            high_coupling_components: high_coupling,
        },
        single_points_of_failure: spofs,
        recommendations,
    }
}

fn find_critical_paths(_graph: &Graph) -> Vec<CriticalPath> {
    vec![]
}

fn find_high_coupling_components(graph: &Graph) -> Vec<String> {
    let mut incoming: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut outgoing: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for edge in &graph.edges {
        *incoming.entry(edge.target.clone()).or_insert(0) += 1;
        *outgoing.entry(edge.source.clone()).or_insert(0) += 1;
    }

    let threshold = 5;
    graph
        .nodes
        .iter()
        .filter(|n| {
            incoming.get(&n.id).copied().unwrap_or(0) > threshold
                || outgoing.get(&n.id).copied().unwrap_or(0) > threshold
        })
        .map(|n| n.label.clone())
        .collect()
}

fn identify_spofs(graph: &Graph) -> Vec<SpofItem> {
    let mut incoming: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for edge in &graph.edges {
        *incoming.entry(edge.target.clone()).or_insert(0) += 1;
    }

    graph
        .nodes
        .iter()
        .filter(|n| incoming.get(&n.id).copied().unwrap_or(0) > 3)
        .map(|n| SpofItem {
            component: n.label.clone(),
            kind: format!("{:?}", n.kind),
            downstream_impact: incoming.get(&n.id).copied().unwrap_or(0),
            recommendation: "Consider adding redundancy or circuit breakers".to_string(),
        })
        .collect()
}

fn generate_sre_recommendations(graph: &Graph, spofs: &[SpofItem]) -> Vec<SreRecommendation> {
    let mut recs = Vec::new();

    if !spofs.is_empty() {
        recs.push(SreRecommendation {
            category: "Reliability".to_string(),
            priority: "High".to_string(),
            description: format!("{} single points of failure identified. Add redundancy.", spofs.len()),
        });
    }

    let db_count = graph.nodes.iter().filter(|n| n.kind == NodeKind::Database).count();
    if db_count > 0 {
        recs.push(SreRecommendation {
            category: "Data".to_string(),
            priority: "Medium".to_string(),
            description: "Ensure database backups and failover mechanisms are in place".to_string(),
        });
    }

    let external_count = graph.nodes.iter().filter(|n| n.kind == NodeKind::ExternalApi).count();
    if external_count > 0 {
        recs.push(SreRecommendation {
            category: "Resilience".to_string(),
            priority: "Medium".to_string(),
            description: "Implement circuit breakers for external API calls".to_string(),
        });
    }

    recs
}

fn print_sre_report(report: &SreReport) {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                 🔧 SRE RELIABILITY REPORT                         ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ 📊 RELIABILITY METRICS                                           │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Total Dependencies:      {:>36} │", report.reliability.total_dependencies);
    println!("│ External Dependencies:   {:>36} │", report.reliability.external_dependencies);
    println!("│ Database Dependencies:   {:>36} │", report.reliability.database_dependencies);
    println!("│ Coupling Score:          {:>36.2} │", report.reliability.coupling_score);
    println!("│ Blast Radius:            {:>36} │", report.reliability.blast_radius);
    println!("└──────────────────────────────────────────────────────────────────┘");
    println!();

    if !report.single_points_of_failure.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ ⚠️ SINGLE POINTS OF FAILURE                                       │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for spof in &report.single_points_of_failure {
            println!("│ {} ({})", spof.component, spof.kind);
            println!("│   Downstream Impact: {} components", spof.downstream_impact);
            println!("│   Recommendation: {}", spof.recommendation);
            println!("│");
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
        println!();
    }

    if !report.recommendations.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ 💡 RECOMMENDATIONS                                               │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for rec in &report.recommendations {
            println!("│ [{}] [{}] {}", rec.category, rec.priority, rec.description);
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DevopsReport {
    pub readiness: DeploymentReadiness,
    pub infrastructure: InfrastructureAnalysis,
    pub cicd_indicators: CICDIndicators,
    pub recommendations: Vec<DevopsRecommendation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentReadiness {
    pub score: f32,
    pub services_ready: usize,
    pub services_total: usize,
    pub blockers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfrastructureAnalysis {
    pub has_containers: bool,
    pub has_iac: bool,
    pub databases: usize,
    pub external_services: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CICDIndicators {
    pub has_tests: bool,
    pub has_linting: bool,
    pub has_dockerfile: bool,
    pub has_ci_config: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DevopsRecommendation {
    pub category: String,
    pub description: String,
    pub priority: String,
}

pub async fn devops(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;

    let report = generate_devops_report(&graph, repo_path);

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        _ => {
            print_devops_report(&report);
        }
    }

    Ok(())
}

fn generate_devops_report(graph: &Graph, repo_path: &Path) -> DevopsReport {
    let services_total = graph.nodes.iter().filter(|n| n.kind == NodeKind::Service).count();
    let databases = graph.nodes.iter().filter(|n| n.kind == NodeKind::Database).count();
    let external_services = graph.nodes.iter().filter(|n| n.kind == NodeKind::ExternalApi).count();

    let has_dockerfile = repo_path.join("Dockerfile").exists() || repo_path.join("docker-compose.yml").exists();
    let has_ci_config = repo_path.join(".github/workflows").exists() 
        || repo_path.join(".gitlab-ci.yml").exists()
        || repo_path.join("Jenkinsfile").exists();
    let has_tests = check_for_tests(repo_path);
    let has_linting = check_for_linting(repo_path);
    let has_iac = check_for_iac(repo_path);

    let mut blockers = Vec::new();
    if !has_dockerfile {
        blockers.push("No containerization detected".to_string());
    }
    if !has_ci_config {
        blockers.push("No CI/CD configuration found".to_string());
    }
    if !has_tests {
        blockers.push("No test infrastructure detected".to_string());
    }

    let readiness_score = calculate_readiness_score(has_dockerfile, has_ci_config, has_tests, has_linting);

    let recommendations = generate_devops_recommendations(&blockers, has_iac, has_dockerfile);

    DevopsReport {
        readiness: DeploymentReadiness {
            score: readiness_score,
            services_ready: if has_dockerfile && has_ci_config { services_total } else { 0 },
            services_total,
            blockers,
        },
        infrastructure: InfrastructureAnalysis {
            has_containers: has_dockerfile,
            has_iac,
            databases,
            external_services,
        },
        cicd_indicators: CICDIndicators {
            has_tests,
            has_linting,
            has_dockerfile,
            has_ci_config,
        },
        recommendations,
    }
}

fn check_for_tests(repo_path: &Path) -> bool {
    repo_path.join("tests").exists()
        || repo_path.join("test").exists()
        || repo_path.join("__tests__").exists()
        || repo_path.join("spec").exists()
}

fn check_for_linting(repo_path: &Path) -> bool {
    repo_path.join(".eslintrc").exists()
        || repo_path.join(".eslintrc.json").exists()
        || repo_path.join(".prettierrc").exists()
        || repo_path.join("pyproject.toml").exists()
        || repo_path.join(".rubocop.yml").exists()
}

fn check_for_iac(repo_path: &Path) -> bool {
    repo_path.join("terraform").exists()
        || repo_path.join("kubernetes").exists()
        || repo_path.join("k8s").exists()
        || repo_path.join("helm").exists()
        || repo_path.join("charts").exists()
        || repo_path.join("ansible").exists()
}

fn calculate_readiness_score(
    has_dockerfile: bool,
    has_ci_config: bool,
    has_tests: bool,
    has_linting: bool,
) -> f32 {
    let mut score = 0.0;
    if has_dockerfile { score += 30.0; }
    if has_ci_config { score += 30.0; }
    if has_tests { score += 25.0; }
    if has_linting { score += 15.0; }
    score
}

fn generate_devops_recommendations(blockers: &[String], has_iac: bool, has_dockerfile: bool) -> Vec<DevopsRecommendation> {
    let mut recs = Vec::new();

    for blocker in blockers {
        recs.push(DevopsRecommendation {
            category: "Deployment Readiness".to_string(),
            description: blocker.clone(),
            priority: "High".to_string(),
        });
    }

    if !has_iac {
        recs.push(DevopsRecommendation {
            category: "Infrastructure".to_string(),
            description: "Consider Infrastructure as Code (Terraform, Pulumi)".to_string(),
            priority: "Medium".to_string(),
        });
    }

    if !has_dockerfile {
        recs.push(DevopsRecommendation {
            category: "Containerization".to_string(),
            description: "Add Dockerfile for consistent deployment environments".to_string(),
            priority: "High".to_string(),
        });
    }

    recs
}

fn print_devops_report(report: &DevopsReport) {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                 🚀 DEVOPS DEPLOYMENT READINESS                    ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ 📊 DEPLOYMENT READINESS SCORE                                    │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Score:          {:>46.0}/100 │", report.readiness.score);
    println!("│ Services Ready: {:>46} │", format!("{}/{}", report.readiness.services_ready, report.readiness.services_total));
    println!("└──────────────────────────────────────────────────────────────────┘");
    println!();

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ 🔍 CI/CD INDICATORS                                              │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Tests:       {:>48} │", if report.cicd_indicators.has_tests { "✅" } else { "❌" });
    println!("│ Linting:     {:>48} │", if report.cicd_indicators.has_linting { "✅" } else { "❌" });
    println!("│ Containers:  {:>48} │", if report.cicd_indicators.has_dockerfile { "✅" } else { "❌" });
    println!("│ CI Config:   {:>48} │", if report.cicd_indicators.has_ci_config { "✅" } else { "❌" });
    println!("└──────────────────────────────────────────────────────────────────┘");
    println!();

    if !report.readiness.blockers.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ ⚠️ DEPLOYMENT BLOCKERS                                            │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for blocker in &report.readiness.blockers {
            println!("│ • {}", blocker);
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
        println!();
    }

    if !report.recommendations.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ 💡 RECOMMENDATIONS                                               │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for rec in &report.recommendations {
            println!("│ [{}] [{}] {}", rec.category, rec.priority, rec.description);
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityReport {
    pub overview: SecurityOverview,
    pub attack_surface: AttackSurface,
    pub data_flows: Vec<DataFlow>,
    pub vulnerabilities: Vec<SecurityVulnerability>,
    pub recommendations: Vec<SecurityRecommendation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityOverview {
    pub external_exposure: usize,
    pub databases_exposed: usize,
    pub auth_indicators: bool,
    pub encryption_indicators: bool,
    pub risk_level: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttackSurface {
    pub entry_points: usize,
    pub external_apis: usize,
    pub databases: usize,
    pub high_risk_components: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataFlow {
    pub source: String,
    pub destination: String,
    pub data_type: String,
    pub risk: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub category: String,
    pub severity: String,
    pub description: String,
    pub affected_component: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityRecommendation {
    pub category: String,
    pub priority: String,
    pub description: String,
}

pub async fn security(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;

    let report = generate_security_report(&graph, repo_path);

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        _ => {
            print_security_report(&report);
        }
    }

    Ok(())
}

fn generate_security_report(graph: &Graph, repo_path: &Path) -> SecurityReport {
    let external_apis = graph.nodes.iter().filter(|n| n.kind == NodeKind::ExternalApi).count();
    let databases = graph.nodes.iter().filter(|n| n.kind == NodeKind::Database).count();
    let services = graph.nodes.iter().filter(|n| n.kind == NodeKind::Service).count();

    let auth_indicators = check_for_auth(repo_path);
    let encryption_indicators = check_for_encryption(repo_path);

    let risk_level = if external_apis > 10 || !auth_indicators {
        "High"
    } else if external_apis > 5 || databases > 3 {
        "Medium"
    } else {
        "Low"
    };

    let mut vulnerabilities = Vec::new();

    if !auth_indicators {
        vulnerabilities.push(SecurityVulnerability {
            category: "Authentication".to_string(),
            severity: "High".to_string(),
            description: "No authentication mechanisms detected".to_string(),
            affected_component: "All services".to_string(),
        });
    }

    if !encryption_indicators {
        vulnerabilities.push(SecurityVulnerability {
            category: "Encryption".to_string(),
            severity: "Medium".to_string(),
            description: "No encryption configuration detected".to_string(),
            affected_component: "Data in transit/rest".to_string(),
        });
    }

    let high_risk: Vec<String> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::ExternalApi || n.kind == NodeKind::Database)
        .map(|n| n.label.clone())
        .collect();

    let recommendations = generate_security_recommendations(&vulnerabilities, external_apis, databases);

    SecurityReport {
        overview: SecurityOverview {
            external_exposure: external_apis,
            databases_exposed: databases,
            auth_indicators,
            encryption_indicators,
            risk_level: risk_level.to_string(),
        },
        attack_surface: AttackSurface {
            entry_points: services,
            external_apis,
            databases,
            high_risk_components: high_risk,
        },
        data_flows: vec![],
        vulnerabilities,
        recommendations,
    }
}

fn check_for_auth(repo_path: &Path) -> bool {
    let auth_indicators = ["auth", "jwt", "oauth", "session", "passport", "cognito", "auth0", "keycloak"];
    if let Ok(entries) = std::fs::read_dir(repo_path) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                let name_lower = name.to_lowercase();
                if auth_indicators.iter().any(|i| name_lower.contains(i)) {
                    return true;
                }
            }
        }
    }
    repo_path.join("middleware").exists() || repo_path.join("auth").exists()
}

fn check_for_encryption(repo_path: &Path) -> bool {
    let enc_indicators = ["ssl", "tls", "https", "encrypt", "crypto", "cert", "key"];
    if let Ok(entries) = std::fs::read_dir(repo_path) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                let name_lower = name.to_lowercase();
                if enc_indicators.iter().any(|i| name_lower.contains(i)) {
                    return true;
                }
            }
        }
    }
    false
}

fn generate_security_recommendations(vulnerabilities: &[SecurityVulnerability], external_apis: usize, databases: usize) -> Vec<SecurityRecommendation> {
    let mut recs = Vec::new();

    for vuln in vulnerabilities {
        recs.push(SecurityRecommendation {
            category: vuln.category.clone(),
            priority: vuln.severity.clone(),
            description: vuln.description.clone(),
        });
    }

    if external_apis > 5 {
        recs.push(SecurityRecommendation {
            category: "API Security".to_string(),
            priority: "High".to_string(),
            description: "Implement API rate limiting and input validation".to_string(),
        });
    }

    if databases > 0 {
        recs.push(SecurityRecommendation {
            category: "Data Protection".to_string(),
            priority: "High".to_string(),
            description: "Enable encryption at rest for all databases".to_string(),
        });
    }

    recs
}

fn print_security_report(report: &SecurityReport) {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                 🔒 SECURITY ANALYSIS REPORT                       ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ 📊 SECURITY OVERVIEW                                             │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Risk Level:         {:>40} │", report.overview.risk_level);
    println!("│ External Exposure:  {:>40} │", report.overview.external_exposure);
    println!("│ Databases Exposed:  {:>40} │", report.overview.databases_exposed);
    println!("│ Auth Detected:      {:>40} │", if report.overview.auth_indicators { "✅" } else { "❌" });
    println!("│ Encryption:         {:>40} │", if report.overview.encryption_indicators { "✅" } else { "❌" });
    println!("└──────────────────────────────────────────────────────────────────┘");
    println!();

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ 🎯 ATTACK SURFACE                                                │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Entry Points:    {:>44} │", report.attack_surface.entry_points);
    println!("│ External APIs:   {:>44} │", report.attack_surface.external_apis);
    println!("│ Databases:       {:>44} │", report.attack_surface.databases);
    println!("└──────────────────────────────────────────────────────────────────┘");
    println!();

    if !report.vulnerabilities.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ ⚠️ VULNERABILITIES                                                │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for vuln in &report.vulnerabilities {
            println!("│ [{}] {} - {}", vuln.severity, vuln.category, vuln.description);
            println!("│   Affected: {}", vuln.affected_component);
            println!("│");
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
        println!();
    }

    if !report.recommendations.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ 💡 SECURITY RECOMMENDATIONS                                      │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for rec in &report.recommendations {
            println!("│ [{}] [{}] {}", rec.category, rec.priority, rec.description);
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductReport {
    pub features: FeatureOverview,
    pub dependencies: FeatureDependencies,
    pub impact_analysis: ImpactAnalysis,
    pub recommendations: Vec<ProductRecommendation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureOverview {
    pub total_modules: usize,
    pub api_endpoints: usize,
    pub services: usize,
    pub features_detected: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureDependencies {
    pub critical_features: Vec<CriticalFeature>,
    pub shared_components: Vec<SharedComponent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CriticalFeature {
    pub name: String,
    pub dependent_features: usize,
    pub risk_if_changed: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SharedComponent {
    pub name: String,
    pub used_by_count: usize,
    pub coordination_needed: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub high_impact_changes: Vec<String>,
    pub isolated_changes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductRecommendation {
    pub category: String,
    pub description: String,
    pub priority: String,
}

pub async fn product(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let graph = scan_repo(repo_path).map_err(|e| CliError::Scan(e.to_string()))?;

    let report = generate_product_report(&graph);

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        _ => {
            print_product_report(&report);
        }
    }

    Ok(())
}

fn generate_product_report(graph: &Graph) -> ProductReport {
    let total_modules = graph.nodes.len();
    let services = graph.nodes.iter().filter(|n| n.kind == NodeKind::Service).count();
    let api_endpoints = count_api_endpoints(graph);

    let features_detected = detect_features(graph);
    let critical_features = identify_critical_features(graph);
    let shared_components = identify_shared_components(graph);

    let high_impact_changes = identify_high_impact_changes(graph);
    let isolated_changes = identify_isolated_changes(graph);

    let recommendations = generate_product_recommendations(&critical_features, &shared_components);

    ProductReport {
        features: FeatureOverview {
            total_modules,
            api_endpoints,
            services,
            features_detected,
        },
        dependencies: FeatureDependencies {
            critical_features,
            shared_components,
        },
        impact_analysis: ImpactAnalysis {
            high_impact_changes,
            isolated_changes,
        },
        recommendations,
    }
}

fn count_api_endpoints(graph: &Graph) -> usize {
    graph
        .nodes
        .iter()
        .filter(|n| {
            n.label.to_lowercase().contains("api")
                || n.label.to_lowercase().contains("controller")
                || n.label.to_lowercase().contains("route")
                || n.label.to_lowercase().contains("handler")
        })
        .count()
}

fn detect_features(graph: &Graph) -> Vec<String> {
    let mut features = std::collections::HashSet::new();

    for node in &graph.nodes {
        let label_lower = node.label.to_lowercase();
        if label_lower.contains("user") {
            features.insert("User Management".to_string());
        }
        if label_lower.contains("auth") || label_lower.contains("login") {
            features.insert("Authentication".to_string());
        }
        if label_lower.contains("payment") || label_lower.contains("checkout") {
            features.insert("Payments".to_string());
        }
        if label_lower.contains("order") {
            features.insert("Order Management".to_string());
        }
        if label_lower.contains("product") || label_lower.contains("catalog") {
            features.insert("Product Catalog".to_string());
        }
        if label_lower.contains("notification") || label_lower.contains("email") {
            features.insert("Notifications".to_string());
        }
        if label_lower.contains("search") {
            features.insert("Search".to_string());
        }
        if label_lower.contains("analytics") || label_lower.contains("report") {
            features.insert("Analytics".to_string());
        }
    }

    let mut v: Vec<_> = features.into_iter().collect();
    v.sort();
    v
}

fn identify_critical_features(graph: &Graph) -> Vec<CriticalFeature> {
    let mut incoming: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for edge in &graph.edges {
        *incoming.entry(edge.target.clone()).or_insert(0) += 1;
    }

    graph
        .nodes
        .iter()
        .filter(|n| incoming.get(&n.id).copied().unwrap_or(0) > 3)
        .map(|n| CriticalFeature {
            name: n.label.clone(),
            dependent_features: incoming.get(&n.id).copied().unwrap_or(0),
            risk_if_changed: "High".to_string(),
        })
        .collect()
}

fn identify_shared_components(graph: &Graph) -> Vec<SharedComponent> {
    let mut incoming: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for edge in &graph.edges {
        *incoming.entry(edge.target.clone()).or_insert(0) += 1;
    }

    graph
        .nodes
        .iter()
        .filter(|n| incoming.get(&n.id).copied().unwrap_or(0) > 2)
        .map(|n| SharedComponent {
            name: n.label.clone(),
            used_by_count: incoming.get(&n.id).copied().unwrap_or(0),
            coordination_needed: "Yes - Changes affect multiple teams".to_string(),
        })
        .collect()
}

fn identify_high_impact_changes(_graph: &Graph) -> Vec<String> {
    vec!["Core services with high fan-out".to_string()]
}

fn identify_isolated_changes(graph: &Graph) -> Vec<String> {
    let orphans = count_orphans(graph);
    if orphans > 0 {
        vec![format!("{} isolated modules with no dependencies", orphans)]
    } else {
        vec![]
    }
}

fn generate_product_recommendations(
    critical_features: &[CriticalFeature],
    shared_components: &[SharedComponent],
) -> Vec<ProductRecommendation> {
    let mut recs = Vec::new();

    if !critical_features.is_empty() {
        recs.push(ProductRecommendation {
            category: "Planning".to_string(),
            description: "Schedule coordination meetings before changing critical features".to_string(),
            priority: "High".to_string(),
        });
    }

    if !shared_components.is_empty() {
        recs.push(ProductRecommendation {
            category: "Architecture".to_string(),
            description: "Consider extracting shared components into dedicated libraries".to_string(),
            priority: "Medium".to_string(),
        });
    }

    recs
}

fn print_product_report(report: &ProductReport) {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                 📦 PRODUCT FEATURE ANALYSIS                       ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ 📊 FEATURE OVERVIEW                                              │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Total Modules:     {:>40} │", report.features.total_modules);
    println!("│ Services:          {:>40} │", report.features.services);
    println!("│ API Endpoints:     {:>40} │", report.features.api_endpoints);
    println!("│ Features Detected: {}", report.features.features_detected.join(", "));
    println!("└──────────────────────────────────────────────────────────────────┘");
    println!();

    if !report.dependencies.critical_features.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ ⚠️ CRITICAL FEATURES (High Impact)                                │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for feature in &report.dependencies.critical_features {
            println!("│ {} - {} dependent features", feature.name, feature.dependent_features);
            println!("│   Risk if changed: {}", feature.risk_if_changed);
            println!("│");
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
        println!();
    }

    if !report.dependencies.shared_components.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ 🔗 SHARED COMPONENTS                                             │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for component in &report.dependencies.shared_components {
            println!("│ {} - Used by {} features", component.name, component.used_by_count);
            println!("│   Coordination: {}", component.coordination_needed);
            println!("│");
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
        println!();
    }

    if !report.recommendations.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────┐");
        println!("│ 💡 PRODUCT RECOMMENDATIONS                                       │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        for rec in &report.recommendations {
            println!("│ [{}] [{}] {}", rec.category, rec.priority, rec.description);
        }
        println!("└──────────────────────────────────────────────────────────────────┘");
    }
}
