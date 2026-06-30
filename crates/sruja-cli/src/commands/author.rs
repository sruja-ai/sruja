use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use super::discover::discover_context_json_from_graph;
use super::CliError;
use crate::integrations::{run_cmd_enrichment, EnrichmentLimits};
use crate::utils::architecture_path;
use sruja_diff::{Proposal, ProposalStatus};
use sruja_scan::graph::compute_all_centrality;
use sruja_scan::scan_scope::ScanScope;
use sruja_scan::{detect_communities, summarize_communities, Edge, Graph};

const AUTHOR_EVIDENCE_SCHEMA_VERSION: &str = "author_evidence/v1";
const AUTHOR_EVIDENCE_DEFAULT_PATH: &str = ".sruja/author_evidence.json";
const AUTHOR_EVIDENCE_MAX_BYTES: usize = 80 * 1024;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorEvidence {
    pub schema_version: String,
    pub repo: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    pub truth_status: String,
    pub scan_scope: ScanScope,
    pub summary: AuthorEvidenceSummary,
    pub workspace_units: Vec<AuthorWorkspaceUnit>,
    pub communities: Vec<AuthorCommunity>,
    pub entrypoints: Vec<AuthorEntrypoint>,
    pub data_stores: Vec<AuthorDataStore>,
    pub repomap_files: Vec<AuthorRepoMapFile>,
    pub manifest_edges: Vec<AuthorManifestEdge>,
    pub open_questions_seed: Vec<String>,
    pub excluded_from_default_context: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorEvidenceSummary {
    pub primary_language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    pub architecture_style: String,
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorWorkspaceUnit {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorCommunity {
    pub id: u32,
    pub suggested_label: String,
    pub member_count: usize,
    pub cohesion: f64,
    pub top_member_paths: Vec<String>,
    pub edge_basis: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorEntrypoint {
    pub id: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorDataStore {
    pub id: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorRepoMapFile {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_preview: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorManifestEdge {
    pub from: String,
    pub to: String,
    pub basis: String,
}

fn iso8601_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn is_workspace_unit_id(id: &str) -> bool {
    id.starts_with("crate:") || id.starts_with("npm:")
}

fn workspace_unit_key(id: &str) -> Option<&str> {
    id.strip_prefix("crate:")
        .or_else(|| id.strip_prefix("npm:"))
}

fn is_manifest_edge(edge: &Edge) -> Option<&'static str> {
    for e in &edge.evidence {
        match e.rule.as_str() {
            "cargo_metadata_dep" => return Some("cargo_metadata_dep"),
            "package_json_dep" => return Some("package_json_dep"),
            _ => {}
        }
    }
    None
}

fn relative_graph_path(repo_root: &str, repo_path: &Path, raw: &str) -> String {
    let normalized = raw.replace('\\', "/");
    let repo_prefix = repo_path
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));
    let repo_arg_norm = repo_root
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string();

    let rel = if let Some(ref prefix) = repo_prefix {
        normalized
            .strip_prefix(prefix)
            .or_else(|| normalized.strip_prefix(&format!("{}/", prefix)))
            .unwrap_or(normalized.as_str())
            .trim_start_matches('/')
            .to_string()
    } else if !repo_arg_norm.is_empty() {
        normalized
            .strip_prefix(&format!("{}/", repo_arg_norm))
            .or_else(|| normalized.strip_prefix(&repo_arg_norm))
            .unwrap_or(normalized.as_str())
            .trim_start_matches('/')
            .to_string()
    } else {
        normalized.trim_start_matches('/').to_string()
    };

    normalize_relative_path(&rel)
}

/// Collapse `..` / `.` segments so community sample paths are stable and dedupe-friendly.
fn normalize_relative_path(path: &str) -> String {
    let mut out = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            Component::Normal(seg) => out.push(seg),
            Component::RootDir | Component::Prefix(_) => out.push(component.as_os_str()),
        }
    }
    out.to_string_lossy().replace('\\', "/")
}

pub fn build_author_evidence_from_graph(
    repo_root: &str,
    repo_path: &Path,
    graph: &Graph,
    truth_status: &str,
    git_commit: Option<String>,
) -> Result<AuthorEvidence, CliError> {
    let context = discover_context_json_from_graph(repo_root, repo_path, graph)?;

    let mut workspace_units: Vec<AuthorWorkspaceUnit> = graph
        .nodes
        .iter()
        .filter(|n| is_workspace_unit_id(&n.id))
        .map(|n| {
            let mut hints: Vec<String> = n
                .metadata
                .iter()
                .filter_map(|(k, v)| {
                    if k.starts_with("hint:") && v == "true" {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();
            hints.sort();
            AuthorWorkspaceUnit {
                id: n.id.clone(),
                label: if n.label.is_empty() {
                    workspace_unit_key(&n.id).unwrap_or(&n.id).to_string()
                } else {
                    n.label.clone()
                },
                technology: n.technology.clone(),
                hints,
            }
        })
        .collect();
    workspace_units.sort_by(|a, b| a.id.cmp(&b.id));
    workspace_units.truncate(25);

    let raw_communities = detect_communities(graph);
    let community_infos = summarize_communities(graph, &raw_communities);
    let mut communities: Vec<AuthorCommunity> = community_infos
        .into_iter()
        .map(|c| {
            let mut seen_paths = HashSet::new();
            let mut top_member_paths = Vec::new();
            for member_id in c.members {
                let path = graph
                    .nodes
                    .iter()
                    .find(|n| n.id == member_id)
                    .and_then(|n| n.path.as_deref())
                    .map(|p| relative_graph_path(repo_root, repo_path, p))
                    .unwrap_or_else(|| member_id.clone());
                if seen_paths.insert(path.clone()) {
                    top_member_paths.push(path);
                }
                if top_member_paths.len() >= 5 {
                    break;
                }
            }
            top_member_paths.sort();
            AuthorCommunity {
                id: c.id,
                suggested_label: c.suggested_label,
                member_count: c.member_count,
                cohesion: c.cohesion,
                top_member_paths,
                edge_basis: "import_graph".to_string(),
            }
        })
        .collect();
    communities.sort_by(|a, b| {
        b.member_count
            .cmp(&a.member_count)
            .then_with(|| a.suggested_label.cmp(&b.suggested_label))
    });
    communities.truncate(12);

    let entrypoints = super::intent_domain::onboard::discover_entrypoints(repo_path, graph, 8)
        .into_iter()
        .map(|e| AuthorEntrypoint {
            id: e.id,
            kind: e.kind,
            path: e.path,
        })
        .collect::<Vec<_>>();
    let data_stores = super::intent_domain::onboard::discover_data_stores(repo_path, graph, 8)
        .into_iter()
        .map(|s| AuthorDataStore {
            id: s.id,
            kind: s.kind,
            technology: s.technology,
            path: s.path,
        })
        .collect::<Vec<_>>();

    let mut manifest_edges: Vec<AuthorManifestEdge> = graph
        .edges
        .iter()
        .filter_map(|e| {
            let basis = is_manifest_edge(e)?;
            Some(AuthorManifestEdge {
                from: e.source.clone(),
                to: e.target.clone(),
                basis: basis.to_string(),
            })
        })
        .collect();
    manifest_edges.sort_by(|a, b| a.from.cmp(&b.from).then_with(|| a.to.cmp(&b.to)));
    manifest_edges.dedup_by(|a, b| a.from == b.from && a.to == b.to && a.basis == b.basis);
    manifest_edges.truncate(80);

    let repomap_files = build_repomap_files(repo_root, repo_path, graph, 12)?;

    let mut open_questions_seed = Vec::new();
    if !communities.is_empty() {
        open_questions_seed.push(
            "Which of these import clusters correspond to real domain boundaries (and which are just implementation adjacency)?"
                .to_string(),
        );
    }
    if entrypoints.is_empty() {
        open_questions_seed.push(
            "What are the true runtime entrypoints (CLI, HTTP server, background jobs) that the scan might miss?"
                .to_string(),
        );
    }
    if data_stores.is_empty() {
        open_questions_seed.push(
            "What persistent stores exist (DBs, queues, caches) that are not detected via direct library imports?"
                .to_string(),
        );
    }

    let mut evidence = AuthorEvidence {
        schema_version: AUTHOR_EVIDENCE_SCHEMA_VERSION.to_string(),
        repo: repo_root.to_string(),
        updated_at: iso8601_now(),
        git_commit,
        truth_status: truth_status.to_string(),
        scan_scope: context.scan_scope,
        summary: AuthorEvidenceSummary {
            primary_language: context.primary_language,
            framework: context.framework,
            architecture_style: context.architecture_style,
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
        },
        workspace_units,
        communities,
        entrypoints,
        data_stores,
        repomap_files,
        manifest_edges,
        open_questions_seed,
        excluded_from_default_context: vec![
            "full_graph".to_string(),
            "raw_call_edges".to_string(),
            "surprising_connections".to_string(),
            "key_relationships".to_string(),
            "violations".to_string(),
        ],
    };

    shrink_author_evidence_to_budget(&mut evidence)?;
    Ok(evidence)
}

fn build_repomap_files(
    repo_root: &str,
    repo_path: &Path,
    graph: &Graph,
    max_files: usize,
) -> Result<Vec<AuthorRepoMapFile>, CliError> {
    let centralities = compute_all_centrality(graph);
    let mut best_by_path: HashMap<String, f64> = HashMap::new();

    for node in &graph.nodes {
        let Some(ref raw_path) = node.path else {
            continue;
        };
        let rel = relative_graph_path(repo_root, repo_path, raw_path);
        if rel.is_empty() {
            continue;
        }
        let score = centralities
            .get(&node.id)
            .map(|c| {
                (c.pagerank * 0.4) + (c.betweenness_centrality * 0.4) + (c.degree_centrality * 0.2)
            })
            .unwrap_or(0.0);
        let slot = best_by_path.entry(rel).or_insert(score);
        if score > *slot {
            *slot = score;
        }
    }

    let mut ranked: Vec<(String, f64)> = best_by_path.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    ranked.truncate(max_files);

    let mut out = Vec::new();
    for (rel_path, _) in ranked {
        let abs = repo_path.join(&rel_path);
        if !abs.exists() || !abs.is_file() {
            out.push(AuthorRepoMapFile {
                path: rel_path,
                signature_preview: None,
            });
            continue;
        }
        let meta_len = abs.metadata().ok().map(|m| m.len()).unwrap_or(0);
        if meta_len > 256 * 1024 {
            out.push(AuthorRepoMapFile {
                path: rel_path,
                signature_preview: None,
            });
            continue;
        }
        let content = std::fs::read_to_string(&abs).unwrap_or_default();
        let language = sruja_scan::tree_sitter::detect_language(&abs);
        let signature_preview = language
            .and_then(|lang| sruja_scan::tree_sitter::parse_file(&abs, &content, lang))
            .and_then(|parsed| {
                if parsed.definitions.is_empty() {
                    return None;
                }
                let lines = parsed
                    .definitions
                    .iter()
                    .take(8)
                    .map(|d| format!("{} {} (L{})", definition_kind_str(&d.kind), d.name, d.line))
                    .collect::<Vec<_>>()
                    .join("\n");
                if lines.trim().is_empty() {
                    None
                } else {
                    Some(lines)
                }
            });
        out.push(AuthorRepoMapFile {
            path: rel_path,
            signature_preview,
        });
    }

    Ok(out)
}

fn definition_kind_str(kind: &sruja_scan::tree_sitter::DefinitionKind) -> &'static str {
    use sruja_scan::tree_sitter::DefinitionKind;
    match kind {
        DefinitionKind::Function => "fn",
        DefinitionKind::Class => "class",
        DefinitionKind::Interface => "interface",
        DefinitionKind::Struct => "struct",
        DefinitionKind::Enum => "enum",
        DefinitionKind::Constant => "const",
        DefinitionKind::Variable => "var",
    }
}

fn shrink_author_evidence_to_budget(evidence: &mut AuthorEvidence) -> Result<(), CliError> {
    let mut serialized =
        serde_json::to_vec(evidence).map_err(|e| CliError::validation(e.to_string()))?;
    if serialized.len() <= AUTHOR_EVIDENCE_MAX_BYTES {
        return Ok(());
    }

    for step in 0..8 {
        match step {
            0 => {
                for f in &mut evidence.repomap_files {
                    f.signature_preview = None;
                }
            }
            1 => {
                evidence.repomap_files.truncate(8);
            }
            2 => {
                for c in &mut evidence.communities {
                    c.top_member_paths.truncate(3);
                }
            }
            3 => {
                evidence.communities.truncate(8);
            }
            4 => {
                evidence.manifest_edges.truncate(40);
            }
            5 => {
                evidence.workspace_units.truncate(15);
            }
            6 => {
                evidence.open_questions_seed.truncate(5);
            }
            7 => {
                evidence.scan_scope.included.truncate(12);
                evidence.scan_scope.excluded.truncate(8);
                evidence.scan_scope.exclude_patterns.truncate(20);
                evidence.scan_scope.user_patterns.truncate(20);
            }
            _ => {}
        }

        serialized =
            serde_json::to_vec(evidence).map_err(|e| CliError::validation(e.to_string()))?;
        if serialized.len() <= AUTHOR_EVIDENCE_MAX_BYTES {
            return Ok(());
        }
    }

    Err(CliError::validation(format!(
        "Author evidence exceeds size cap ({} bytes) after truncation ({} bytes). Reduce scan scope or adjust caps.",
        AUTHOR_EVIDENCE_MAX_BYTES,
        serialized.len()
    )))
}

pub async fn author_evidence(
    repo_root: &str,
    format: &str,
    output: Option<&str>,
    quiet: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = super::scan_repo_cached(repo_path)?;
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let truth_status = compute_truth_status(&graph, baseline_path.as_deref())?;

    let evidence = build_author_evidence_from_graph(
        repo_root,
        repo_path,
        &graph,
        &truth_status,
        crate::commands::git_commit_short(repo_path),
    )?;

    match format {
        "json" | "text" => {
            let rendered = serde_json::to_string_pretty(&evidence)
                .map_err(|e| CliError::validation(e.to_string()))?;
            if let Some(out_path) = output {
                let p = repo_path.join(out_path);
                super::sync_cmd::atomic_write_file(&p, rendered.as_bytes())?;
                eprintln!("Wrote {}", p.display());
                if !quiet {
                    println!("{}", rendered);
                }
            } else {
                let default_path = repo_path.join(AUTHOR_EVIDENCE_DEFAULT_PATH);
                super::sync_cmd::atomic_write_file(&default_path, rendered.as_bytes())?;
                eprintln!("Wrote {}", default_path.display());
                if !quiet {
                    println!("{}", rendered);
                }
            }
        }
        other => {
            return Err(CliError::validation(format!(
                "Unsupported format for author evidence: {} (expected json)",
                other
            )));
        }
    }

    Ok(())
}

pub async fn author_propose(
    repo_root: &str,
    enrich_cmd: &str,
    enrich_timeout_ms: u64,
    enrich_max_bytes: usize,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo_root),
        )));
    }

    let graph = super::scan_repo_cached(repo_path)?;
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let truth_status = compute_truth_status(&graph, baseline_path.as_deref())?;
    let evidence = build_author_evidence_from_graph(
        repo_root,
        repo_path,
        &graph,
        &truth_status,
        crate::commands::git_commit_short(repo_path),
    )?;

    let stdin_payload = serde_json::to_vec(&serde_json::json!({
        "schema_version": "author_propose_input/v1",
        "instructions": "Return a single JSON object that matches the sruja-diff Proposal schema. Do not output markdown. Include open_questions and evidence_refs when making non-trivial claims.",
        "evidence": evidence
    }))
    .map_err(|e| CliError::validation(e.to_string()))?;

    let output = run_cmd_enrichment(
        enrich_cmd,
        &stdin_payload,
        EnrichmentLimits::with_defaults(enrich_timeout_ms, enrich_max_bytes),
    )
    .map_err(CliError::validation)?;

    let json_text = extract_json_object(&output).ok_or_else(|| {
        CliError::validation("enrich-cmd did not return a JSON object".to_string())
    })?;

    let mut proposal: Proposal =
        serde_json::from_str(&json_text).map_err(|e| CliError::validation(e.to_string()))?;
    proposal.status = ProposalStatus::Draft;
    if proposal.created_at.trim().is_empty() {
        proposal.created_at = iso8601_now();
    }
    if proposal.id.trim().is_empty() {
        proposal.id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    }

    let intent_model = sruja_intent::IntentModel::default();
    proposal.validate(&graph, &intent_model);

    let saved = proposal
        .save(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    println!(
        "{}",
        serde_json::to_string_pretty(&proposal).unwrap_or(output)
    );
    eprintln!("Wrote {}", saved.display());
    Ok(())
}

fn compute_truth_status(graph: &Graph, baseline_path: Option<&Path>) -> Result<String, CliError> {
    if let Some(baseline_file) = baseline_path {
        let content = std::fs::read_to_string(baseline_file)?;
        let parser = sruja_language::Parser::new(baseline_file.to_string_lossy().as_ref());
        let program = parser.parse(&content).map_err(|diags| {
            CliError::parse_with_diagnostics(baseline_file.to_string_lossy().to_string(), diags)
        })?;
        let proposed_graph = sruja_diff::program_to_graph(&program);
        let diff = sruja_diff::compare_graphs(graph, &proposed_graph);
        Ok(match diff.truth_status {
            sruja_diff::TruthStatus::Reviewed => "reviewed",
            sruja_diff::TruthStatus::Drifted => "drifted",
            sruja_diff::TruthStatus::Unknown => "unknown",
        }
        .to_string())
    } else {
        Ok("unknown".to_string())
    }
}

pub fn load_or_build_author_evidence(repo_root: &str) -> Result<AuthorEvidence, CliError> {
    let repo_path = Path::new(repo_root);
    let p = repo_path.join(AUTHOR_EVIDENCE_DEFAULT_PATH);
    let current_commit = crate::commands::git_commit_short(repo_path);
    if let Ok(txt) = std::fs::read_to_string(&p) {
        if let Ok(existing) = serde_json::from_str::<AuthorEvidence>(&txt) {
            if existing.schema_version == AUTHOR_EVIDENCE_SCHEMA_VERSION
                && existing.git_commit == current_commit
            {
                return Ok(existing);
            }
        }
    }

    let graph = super::scan_repo_cached(repo_path)?;
    let baseline_path = architecture_path::resolve_architecture_path(repo_path);
    let truth_status = compute_truth_status(&graph, baseline_path.as_deref())?;
    let evidence = build_author_evidence_from_graph(
        repo_root,
        repo_path,
        &graph,
        &truth_status,
        crate::commands::git_commit_short(repo_path),
    )?;

    let rendered =
        serde_json::to_string_pretty(&evidence).map_err(|e| CliError::validation(e.to_string()))?;
    super::sync_cmd::atomic_write_file(&p, rendered.as_bytes())?;
    Ok(evidence)
}

pub fn author_evidence_default_path(repo_root: &str) -> PathBuf {
    Path::new(repo_root).join(AUTHOR_EVIDENCE_DEFAULT_PATH)
}

/// Pull the first top-level JSON object from enrich-cmd stdout (handles optional markdown fences).
fn extract_json_object(output: &str) -> Option<String> {
    let trimmed = output.trim();
    if trimmed.starts_with('{') && serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
        return Some(trimmed.to_string());
    }
    if let Some(start) = trimmed.find("```json") {
        let rest = &trimmed[start + 7..];
        if let Some(end) = rest.find("```") {
            let inner = rest[..end].trim();
            if serde_json::from_str::<serde_json::Value>(inner).is_ok() {
                return Some(inner.to_string());
            }
        }
    }
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    if end <= start {
        return None;
    }
    let candidate = trimmed[start..=end].trim();
    if serde_json::from_str::<serde_json::Value>(candidate).is_ok() {
        Some(candidate.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_relative_path_collapses_parent_dirs() {
        assert_eq!(
            normalize_relative_path("crates/foo/../../bar/baz.rs"),
            "bar/baz.rs"
        );
    }

    #[test]
    fn extract_json_object_from_fenced_markdown() {
        let raw = r#"Here is the proposal:
```json
{"id":"x","title":"t","description":"d","status":"draft","changes":[]}
```
"#;
        let json = extract_json_object(raw).expect("json");
        assert!(json.contains("\"id\":\"x\""));
    }
}
