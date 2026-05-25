//! `sruja explore` — produce the ExplorerModel JSON consumed by the
//! VS Code Architecture Explorer webview.

use std::collections::HashMap;
use std::path::Path;

use sruja_export::explorer::{DriftOverlay, ExplorerBuilder, NodeDriftInfo};
use sruja_graph::{CentralityAnalyzer, CouplingAnalyzer, SccAnalyzer};
use sruja_scan::graph::community::{detect_communities, summarize_communities};

use crate::commands::CliError;
use crate::graph_store;
use crate::utils::progress;

pub async fn explore(repo_root: &str) -> Result<(), CliError> {
    let json = explore_json(repo_root)?;
    println!("{json}");
    Ok(())
}

pub fn explore_json(repo_root: &str) -> Result<String, CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {repo_root}"),
        )));
    }

    let pb = progress::spinner("Building architecture explorer model…");

    // 1. Knowledge graph (scan + decisions) — read-only, no cache side-effects
    let mut kg = graph_store::load_or_build_graph(repo_path)?;
    // Fresh scan for community detection and drift (avoids persisting cache)
    let scan_graph = sruja_scan::scan_repo(repo_path)
        .map_err(|e| CliError::scan(e.to_string()))?;

    // Overlay declared architecture if a baseline exists
    let baseline_path =
        crate::utils::architecture_path::resolve_architecture_path(repo_path);
    if let Some(ref path) = baseline_path {
        let content = std::fs::read_to_string(path)?;
        let parser =
            sruja_language::Parser::new(path.to_string_lossy().to_string());
        if let Ok(program) = parser.parse(&content) {
            let scan_graph = sruja_diff::program_to_graph(&program);
            sruja_graph::scan_merge::merge_scan_into_graph(
                &mut kg,
                &scan_graph,
                &repo_path.display().to_string(),
            );
            sruja_graph::scan_merge::merge_program_into_graph(
                &mut kg,
                &program,
                &repo_path.display().to_string(),
            );
        }
    }

    let mut builder = ExplorerBuilder::new(kg.clone());

    // 2. Coupling analysis
    let coupling = CouplingAnalyzer.analyze_graph(&kg);
    builder = builder.coupling(coupling);

    // 3. Centrality analysis
    let centrality = CentralityAnalyzer::default().analyze_graph(&kg);
    builder = builder.centrality(centrality);

    // 4. SCC / cycle detection
    let scc = SccAnalyzer::default().analyze_graph(&kg);
    builder = builder.scc(scc);

    // 5. Community detection (reuses the inline scan)
    {
        let community_map = detect_communities(&scan_graph);
        let communities = summarize_communities(&scan_graph, &community_map);
        builder = builder.communities(communities);
    }

    // 6. Drift overlay (optional — requires baseline architecture file)
    if let Some(ref arch_path) = baseline_path {
        if let Ok(overlay) = build_drift_overlay(&scan_graph, arch_path) {
            builder = builder.drift(overlay);
        }
    }

    pb.finish_and_clear();

    let model = builder.build();
    serde_json::to_string(&model).map_err(|e| CliError::validation(e.to_string()))
}

fn build_drift_overlay(
    scan_graph: &sruja_scan::Graph,
    arch_path: &Path,
) -> Result<DriftOverlay, CliError> {
    let intent_model =
        sruja_intent::model::IntentModel::from_sruja_file(arch_path).map_err(|e| {
            CliError::validation(format!("Failed to build intent model: {e}"))
        })?;
    let schema = sruja_language::DomainSchema::architecture();
    let drift_report =
        sruja_intent::compare::DriftDetector::default()
            .detect(&intent_model, scan_graph, &schema);

    let mut node_counts: HashMap<String, (usize, Option<String>)> = HashMap::new();
    for d in &drift_report.drifts {
        if let Some(ref intent_ref) = d.intent_ref {
            let entry = node_counts.entry(intent_ref.clone()).or_default();
            entry.0 += 1;
            let sev = format!("{:?}", d.severity).to_lowercase();
            if entry.1.is_none()
                || severity_rank(&sev)
                    > severity_rank(entry.1.as_deref().unwrap_or("info"))
            {
                entry.1 = Some(sev);
            }
        }
    }

    let health_str = format!("{:?}", drift_report.health).to_lowercase();

    let mut nodes = HashMap::new();
    for (id, (count, sev)) in node_counts {
        nodes.insert(
            id,
            NodeDriftInfo {
                count,
                severity_max: sev,
                health: health_str.clone(),
            },
        );
    }

    Ok(DriftOverlay {
        score: drift_report.drift_score,
        health: health_str,
        nodes,
        edges: Vec::new(),
    })
}

fn severity_rank(s: &str) -> u8 {
    match s {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}
