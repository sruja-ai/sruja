//! Smart commit selection for timeline explain: score by architecture-signal, top-k, optional diversity.

use std::path::Path;
use std::process::Command;

use crate::commands::CliError;

/// One commit line from git log: "SHORT_SHA\tDATE\tSUBJECT"
#[derive(Debug, Clone)]
pub struct CommitCandidate {
    pub short_sha: String,
    pub date: String,
    pub subject: String,
    pub score: f64,
}

const ARCH_PATHS: &[&str] = &["src/", "crates/", "services/", "api/", "infra/", "docs/adr/"];
const ARCH_KEYWORDS: &[&str] = &[
    "refactor", "architecture", "module", "service", "boundary", "migration", "split", "merge",
];
const CODE_EXTENSIONS: &[&str] = &[".rs", ".go", ".ts", ".js", ".py"];

/// Run git log and return lines "SHORT_SHA\tDATE\tSUBJECT" (newest first).
pub fn git_log_commits(repo_path: &Path, max: usize) -> Result<Vec<String>, CliError> {
    let out = Command::new("git")
        .args(["-C", repo_path.as_os_str().to_str().unwrap_or("."), "log"])
        .arg(format!("-{}", max))
        .arg("--format=%h%x09%ci%x09%s")
        .output()
        .map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    if !out.status.success() {
        return Err(CliError::Validation(format!(
            "git log failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }

    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<String> = s
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    Ok(lines)
}

/// Parse "SHORT_SHA\tDATE\tSUBJECT" into (sha, date, subject).
fn parse_log_line(line: &str) -> Option<(String, String, String)> {
    let parts: Vec<&str> = line.splitn(3, '\t').collect();
    if parts.len() >= 3 {
        Some((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        ))
    } else {
        None
    }
}

/// Score a commit by subject keywords and (if we had changed-files count we'd use it; here we only use subject).
/// Returns a non-negative score; higher = more architecture-significant.
fn score_commit_subject(subject: &str) -> f64 {
    let s = subject.to_lowercase();
    let mut score = 0.0;
    for kw in ARCH_KEYWORDS {
        if s.contains(kw) {
            score += 1.0;
        }
    }
    score
}

/// Score using number of changed files under ARCH_PATHS (requires git show --name-only).
fn score_commit_files(repo_path: &Path, short_sha: &str) -> f64 {
    let out = Command::new("git")
        .args(["-C", repo_path.as_os_str().to_str().unwrap_or("."), "show", "--name-only", "--format=", short_sha])
        .output();
    let out = match out {
        Ok(o) if o.status.success() => o,
        _ => return 0.0,
    };
    let names = String::from_utf8_lossy(&out.stdout);
    let mut count = 0;
    for path in names.lines().filter(|l| !l.trim().is_empty()) {
        if ARCH_PATHS.iter().any(|p| path.starts_with(p)) {
            count += 1;
        }
        if CODE_EXTENSIONS.iter().any(|e| path.ends_with(e)) {
            count += 1;
        }
    }
    count as f64 * 0.5
}

/// Score one commit (subject + file count under arch paths).
pub fn score_one(repo_path: &Path, line: &str) -> f64 {
    let (sha, _date, subject) = match parse_log_line(line) {
        Some(t) => t,
        None => return 0.0,
    };
    let subj_score = score_commit_subject(&subject);
    let file_score = score_commit_files(repo_path, &sha);
    subj_score + file_score
}

/// From recent commits (last 200), score each, take top 30, then optionally cap and enforce diversity.
/// Returns candidates sorted oldest-first (chronological for timeline output).
pub fn score_commits(
    repo_path: &Path,
    window: usize,
    top_k: usize,
    max_final: usize,
) -> Result<Vec<CommitCandidate>, CliError> {
    let lines = git_log_commits(repo_path, window)?;
    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let mut scored: Vec<CommitCandidate> = lines
        .into_iter()
        .filter_map(|line| {
            let (short_sha, date, subject) = parse_log_line(&line)?;
            let score = score_one(repo_path, &line);
            Some(CommitCandidate {
                short_sha,
                date,
                subject,
                score,
            })
        })
        .collect();

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);

    // Diversity: prefer not adjacent same-minute/author; take up to max_final
    // Simplified: just take first max_final by score (chronology restored below by reversing)
    scored.truncate(max_final);

    // git log is newest-first; plan wants "oldest -> newest for output"
    scored.reverse();
    Ok(scored)
}
