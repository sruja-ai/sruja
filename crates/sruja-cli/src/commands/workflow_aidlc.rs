//! AI-DLC bridge: parse aidlc-docs state, profile checklists, install rules (no top-level `aidlc` CLI).

use crate::commands::CliError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Manifest `aidlc` section (`workflow/v2`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidlcConfig {
    #[serde(default)]
    pub enabled: bool,
    /// `minimal` or `full`
    #[serde(default = "default_aidlc_profile")]
    pub profile: String,
    /// Relative to each workflow phase dir (default: `aidlc-docs`)
    #[serde(default = "default_docs_root")]
    pub docs_root: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inception_required: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub construction_required: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enabled_extensions: Vec<String>,
}

fn default_aidlc_profile() -> String {
    "minimal".to_string()
}

fn default_docs_root() -> String {
    "aidlc-docs".to_string()
}

/// Public for manifest init (distinct from serde default fn).
pub fn default_docs_root_for_manifest() -> String {
    default_docs_root()
}

impl Default for AidlcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            profile: default_aidlc_profile(),
            docs_root: default_docs_root(),
            inception_required: Vec::new(),
            construction_required: Vec::new(),
            rules_source: None,
            rules_ref: None,
            enabled_extensions: Vec::new(),
        }
    }
}

/// Parsed summary from `aidlc-state.md` (best-effort).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AidlcStateSummary {
    pub state_file: Option<String>,
    pub current_phase: Option<String>,
    pub current_stage: Option<String>,
    pub next_stage: Option<String>,
    pub project_type: Option<String>,
}

/// Status block included in `workflow status` JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidlcStatus {
    pub enabled: bool,
    pub profile: String,
    pub docs_root: String,
    pub state: AidlcStateSummary,
    pub missing: Vec<String>,
    pub next_stage_hint: Option<String>,
}

pub fn aidlc_docs_dir(phase_dir: &Path, config: &AidlcConfig) -> PathBuf {
    phase_dir.join(&config.docs_root)
}

pub fn find_aidlc_state_file(
    repo: &Path,
    workflow_id: &str,
    phase: &str,
    config: &AidlcConfig,
) -> Option<PathBuf> {
    let phase_base = repo
        .join(".sruja")
        .join("workflows")
        .join(workflow_id)
        .join(phase);
    let docs = aidlc_docs_dir(&phase_base, config);
    let candidates = [
        docs.join("aidlc-state.md"),
        docs.join("aidlc-docs").join("aidlc-state.md"),
        repo.join("aidlc-docs").join("aidlc-state.md"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

pub fn parse_aidlc_state(path: &Path) -> AidlcStateSummary {
    let Ok(text) = std::fs::read_to_string(path) else {
        return AidlcStateSummary::default();
    };
    let mut summary = AidlcStateSummary {
        state_file: Some(path.to_string_lossy().to_string()),
        ..Default::default()
    };
    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("- **Current Phase**:") {
            summary.current_phase = Some(v.trim().trim_matches('*').to_string());
        } else if let Some(v) = line.strip_prefix("- **Current Stage**:") {
            summary.current_stage = Some(v.trim().trim_matches('*').to_string());
        } else if let Some(v) = line.strip_prefix("- **Next Stage**:") {
            summary.next_stage = Some(v.trim().trim_matches('*').to_string());
        } else if let Some(v) = line.strip_prefix("- **Project Type**:") {
            summary.project_type = Some(v.trim().to_string());
        }
    }
    summary
}

fn default_inception_paths(config: &AidlcConfig) -> Vec<String> {
    if !config.inception_required.is_empty() {
        return config.inception_required.clone();
    }
    match config.profile.as_str() {
        "full" | "e2e" => vec![
            "aidlc-state.md".into(),
            "inception/requirements/requirements.md".into(),
            "inception/application-design/components.md".into(),
        ],
        _ => vec!["aidlc-state.md".into()],
    }
}

fn default_construction_paths(config: &AidlcConfig) -> Vec<String> {
    if !config.construction_required.is_empty() {
        return config.construction_required.clone();
    }
    match config.profile.as_str() {
        "full" | "e2e" => vec!["construction/build-and-test/build-and-test-summary.md".into()],
        _ => vec![],
    }
}

/// Missing artifact paths (relative to phase `aidlc-docs` root) for the workflow phase gate.
pub fn aidlc_missing_for_phase(
    repo: &Path,
    workflow_id: &str,
    workflow_phase: &str,
    config: &AidlcConfig,
) -> Vec<String> {
    if !config.enabled {
        return Vec::new();
    }
    let phase_base = repo
        .join(".sruja")
        .join("workflows")
        .join(workflow_id)
        .join(workflow_phase);
    let docs_root = aidlc_docs_dir(&phase_base, config);
    if !docs_root.exists() && workflow_phase == "inception" {
        return vec![format!(
            "{}/ (aidlc docs root missing)",
            docs_root.display()
        )];
    }

    let rel_paths: Vec<String> = match workflow_phase {
        "inception" => default_inception_paths(config),
        "construction" => default_construction_paths(config),
        "operations" => Vec::new(),
        _ => Vec::new(),
    };

    let mut missing = Vec::new();
    for rel in rel_paths {
        let p = docs_root.join(&rel);
        if !p.is_file() {
            missing.push(format!("aidlc:{}", docs_root.join(rel).display()));
        }
    }
    missing
}

pub fn build_aidlc_status(
    repo: &Path,
    workflow_id: &str,
    workflow_phase: &str,
    config: &AidlcConfig,
) -> AidlcStatus {
    let state_path = find_aidlc_state_file(repo, workflow_id, workflow_phase, config);
    let state = state_path
        .as_ref()
        .map(|p| parse_aidlc_state(p))
        .unwrap_or_default();
    let missing = aidlc_missing_for_phase(repo, workflow_id, workflow_phase, config);
    let next_stage_hint = state.next_stage.clone().or(state.current_stage.clone());
    AidlcStatus {
        enabled: config.enabled,
        profile: config.profile.clone(),
        docs_root: config.docs_root.clone(),
        state,
        missing: missing.clone(),
        next_stage_hint,
    }
}

/// Locate vendored `aidlc-rules` (env `SRUJA_AIDLC_RULES` or walk parents for `aidlc-workflows/aidlc-rules`).
pub fn resolve_vendored_aidlc_rules(start: &Path) -> Option<PathBuf> {
    if let Ok(env) = std::env::var("SRUJA_AIDLC_RULES") {
        let p = PathBuf::from(env);
        if p.join("aws-aidlc-rules").is_dir() || p.join("core-workflow.md").exists() {
            return Some(p);
        }
    }
    let mut cur = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    for _ in 0..12 {
        let candidate = cur.join("aidlc-workflows").join("aidlc-rules");
        if candidate.join("aws-aidlc-rules").is_dir() {
            return Some(candidate);
        }
        if !cur.pop() {
            break;
        }
    }
    None
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}

/// Install AIDLC rules into `.aidlc/aidlc-rules/` and `.aidlc-rule-details/` for editor consumption.
pub fn install_aidlc_rules(repo: &Path) -> Result<Vec<String>, CliError> {
    let rules_src = resolve_vendored_aidlc_rules(repo).ok_or_else(|| {
        CliError::validation(
            "Could not find aidlc-rules. Set SRUJA_AIDLC_RULES or vendor aidlc-workflows/aidlc-rules.".to_string(),
        )
    })?;

    let mut installed = Vec::new();
    let aidlc_rules_dst = repo.join(".aidlc").join("aidlc-rules");
    if aidlc_rules_dst.exists() {
        std::fs::remove_dir_all(&aidlc_rules_dst).map_err(CliError::Io)?;
    }
    copy_dir_all(&rules_src, &aidlc_rules_dst).map_err(CliError::Io)?;
    installed.push(aidlc_rules_dst.display().to_string());

    let details_src = rules_src.join("aws-aidlc-rule-details");
    if details_src.is_dir() {
        let details_dst = repo.join(".aidlc-rule-details");
        if details_dst.exists() {
            std::fs::remove_dir_all(&details_dst).map_err(CliError::Io)?;
        }
        copy_dir_all(&details_src, &details_dst).map_err(CliError::Io)?;
        installed.push(details_dst.display().to_string());
    }

    Ok(installed)
}

pub fn append_workflow_audit(
    repo: &Path,
    workflow_id: &str,
    event: &str,
    actor: &str,
) -> Result<PathBuf, CliError> {
    let path = repo
        .join(".sruja")
        .join("workflows")
        .join(workflow_id)
        .join("audit.jsonl");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let line = serde_json::json!({
        "at": chrono::Utc::now().to_rfc3339(),
        "actor": actor,
        "event": event,
    });
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "{}", line)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_state_extracts_phase() {
        let dir = std::env::temp_dir().join(format!("aidlc-state-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("aidlc-state.md");
        std::fs::write(
            &path,
            r#"# State
- **Current Phase**: INCEPTION
- **Current Stage**: Requirements
- **Next Stage**: Application Design
"#,
        )
        .unwrap();
        let s = parse_aidlc_state(&path);
        assert_eq!(s.current_phase.as_deref(), Some("INCEPTION"));
        assert_eq!(s.current_stage.as_deref(), Some("Requirements"));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn aidlc_missing_for_phase_disabled_returns_empty() {
        let dir =
            std::env::temp_dir().join(format!("aidlc-missing-disabled-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = AidlcConfig {
            enabled: false,
            ..Default::default()
        };
        let missing = aidlc_missing_for_phase(&dir, "wf-1", "inception", &cfg);
        assert!(missing.is_empty());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn aidlc_missing_for_phase_inception_minimal_reports_missing_docs_root() {
        let dir =
            std::env::temp_dir().join(format!("aidlc-missing-minimal-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = AidlcConfig {
            enabled: true,
            profile: "minimal".to_string(),
            ..Default::default()
        };
        let missing = aidlc_missing_for_phase(&dir, "wf-1", "inception", &cfg);
        assert!(!missing.is_empty());
        assert!(missing
            .iter()
            .any(|m| m.contains("aidlc docs root missing")));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn aidlc_missing_for_phase_inception_full_reports_missing_required_files() {
        let dir = std::env::temp_dir().join(format!("aidlc-missing-full-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = AidlcConfig {
            enabled: true,
            profile: "full".to_string(),
            ..Default::default()
        };

        // Create the docs root so we test missing-file reporting, not only root-missing.
        let docs_root = dir
            .join(".sruja")
            .join("workflows")
            .join("wf-1")
            .join("inception")
            .join(&cfg.docs_root);
        std::fs::create_dir_all(&docs_root).unwrap();
        // Minimal artifact exists, others missing.
        std::fs::write(docs_root.join("aidlc-state.md"), "# State\n").unwrap();

        let missing = aidlc_missing_for_phase(&dir, "wf-1", "inception", &cfg);
        assert!(!missing.is_empty());
        assert!(missing
            .iter()
            .any(|m| m.contains("inception/requirements/requirements.md")));
        assert!(missing
            .iter()
            .any(|m| m.contains("inception/application-design/components.md")));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn install_rules_missing_source_is_error() {
        let dir =
            std::env::temp_dir().join(format!("aidlc-install-missing-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        // Ensure env doesn't point to rules.
        std::env::remove_var("SRUJA_AIDLC_RULES");
        let err = install_aidlc_rules(&dir).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Could not find aidlc-rules"));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn build_aidlc_status_includes_state_and_missing() {
        let dir = std::env::temp_dir().join(format!("aidlc-status-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = AidlcConfig {
            enabled: true,
            profile: "minimal".to_string(),
            ..Default::default()
        };
        let docs_root = dir
            .join(".sruja")
            .join("workflows")
            .join("wf-1")
            .join("inception")
            .join(&cfg.docs_root);
        std::fs::create_dir_all(&docs_root).unwrap();
        std::fs::write(
            docs_root.join("aidlc-state.md"),
            r#"# State
- **Current Phase**: INCEPTION
- **Current Stage**: Requirements
"#,
        )
        .unwrap();

        let status = build_aidlc_status(&dir, "wf-1", "inception", &cfg);
        assert!(status.enabled);
        assert_eq!(status.profile, "minimal");
        assert_eq!(status.state.current_phase.as_deref(), Some("INCEPTION"));
        assert!(
            status.missing.is_empty(),
            "minimal inception requires only aidlc-state.md"
        );
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn append_workflow_audit_appends_jsonl() {
        let dir = std::env::temp_dir().join(format!("aidlc-audit-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = append_workflow_audit(&dir, "wf-1", "stage complete", "tester").unwrap();
        let txt = std::fs::read_to_string(&path).unwrap();
        assert!(txt.contains("\"actor\":\"tester\""));
        assert!(txt.contains("\"event\":\"stage complete\""));
        std::fs::remove_dir_all(dir).ok();
    }
}
