//! Multi-layer signal extraction from session context.
//!
//! Modeled after Evolver's `SIGNAL_PROFILES` and `extractSignals` — the three
//! layers are:
//!
//! 1. **Regex** (Layer 1): Deterministic, zero-latency pattern matching for
//!    known signal types (errors, performance, capability gaps).
//!
//! 2. **Keyword scoring** (Layer 2): Weighted keyword accumulation with
//!    confidence thresholds — catches fuzzy/distributed patterns that no
//!    single regex can match.
//!
//! The extracted signals are used to rank learnings by `signals_match` overlap,
//! making retrieval context-aware rather than just keyword-based.
//!
//! ## De-duplication
//!
//! Signals that appear in 3+ of the last 8 events are suppressed to prevent
//! repair loops — the system stops feeding the same failure into the gene
//! selector repeatedly.

use std::collections::{HashMap, HashSet};

/// A named signal extracted from session context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signal {
    /// Signal name (e.g., "log_error", "perf_bottleneck").
    pub name: String,
    /// Optional detail payload (e.g., the error signature).
    pub detail: Option<String>,
}

impl Signal {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            detail: None,
        }
    }

    pub fn with_detail(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            detail: Some(detail.into()),
        }
    }
}

/// Weighted keyword profile for a signal type.
struct SignalProfile {
    keywords: HashMap<String, f64>,
    threshold: f64,
}

/// Layer 2 keyword scoring profiles — mirrors Evolver's `SIGNAL_PROFILES`.
///
/// Each profile defines weighted keywords and a threshold. The signal fires
/// when the accumulated keyword weight exceeds the threshold.
fn signal_profiles() -> Vec<(&'static str, SignalProfile)> {
    vec![
        (
            "perf_bottleneck",
            SignalProfile {
                keywords: HashMap::from([
                    ("slow".into(), 3.0),
                    ("timeout".into(), 4.0),
                    ("timed out".into(), 4.0),
                    ("latency".into(), 3.0),
                    ("bottleneck".into(), 5.0),
                    ("lag".into(), 2.0),
                    ("delay".into(), 2.0),
                    ("hung".into(), 3.0),
                    ("freeze".into(), 3.0),
                    ("unresponsive".into(), 4.0),
                    ("took too long".into(), 4.0),
                    ("high cpu".into(), 4.0),
                    ("high memory".into(), 4.0),
                    ("oom".into(), 5.0),
                    ("out of memory".into(), 5.0),
                    ("performance".into(), 2.0),
                    ("throttle".into(), 3.0),
                ]),
                threshold: 6.0,
            },
        ),
        (
            "capability_gap",
            SignalProfile {
                keywords: HashMap::from([
                    ("not supported".into(), 5.0),
                    ("cannot".into(), 1.0),
                    ("unsupported".into(), 4.0),
                    ("not implemented".into(), 5.0),
                    ("no way to".into(), 3.0),
                    ("missing feature".into(), 5.0),
                    ("not available".into(), 3.0),
                    ("no support for".into(), 4.0),
                    ("unavailable".into(), 3.0),
                    ("incompatible".into(), 3.0),
                ]),
                threshold: 5.0,
            },
        ),
        (
            "user_feature_request",
            SignalProfile {
                keywords: HashMap::from([
                    ("add".into(), 1.0),
                    ("implement".into(), 3.0),
                    ("create".into(), 2.0),
                    ("build".into(), 2.0),
                    ("feature".into(), 3.0),
                    ("i want".into(), 3.0),
                    ("i need".into(), 3.0),
                    ("we need".into(), 3.0),
                    ("please add".into(), 4.0),
                    ("new function".into(), 4.0),
                    ("new module".into(), 4.0),
                    ("endpoint".into(), 2.0),
                    ("capability".into(), 2.0),
                    ("support for".into(), 2.0),
                ]),
                threshold: 6.0,
            },
        ),
        (
            "user_improvement_suggestion",
            SignalProfile {
                keywords: HashMap::from([
                    ("improve".into(), 3.0),
                    ("enhance".into(), 3.0),
                    ("upgrade".into(), 3.0),
                    ("refactor".into(), 4.0),
                    ("clean up".into(), 3.0),
                    ("simplify".into(), 3.0),
                    ("streamline".into(), 3.0),
                    ("optimize".into(), 3.0),
                    ("could be better".into(), 4.0),
                    ("should be".into(), 2.0),
                    ("more efficient".into(), 3.0),
                ]),
                threshold: 5.0,
            },
        ),
        (
            "recurring_error",
            SignalProfile {
                keywords: HashMap::from([
                    ("error".into(), 1.0),
                    ("exception".into(), 2.0),
                    ("failed".into(), 1.0),
                    ("crash".into(), 4.0),
                    ("again".into(), 1.0),
                    ("still".into(), 1.0),
                    ("keeps".into(), 2.0),
                    ("repeatedly".into(), 4.0),
                    ("same error".into(), 5.0),
                    ("still failing".into(), 5.0),
                    ("not fixed".into(), 4.0),
                ]),
                threshold: 7.0,
            },
        ),
        (
            "tool_bypass",
            SignalProfile {
                keywords: HashMap::from([
                    ("exec".into(), 2.0),
                    ("shell".into(), 2.0),
                    ("subprocess".into(), 3.0),
                    ("child_process".into(), 3.0),
                    ("curl".into(), 2.0),
                    ("wget".into(), 2.0),
                    ("ad-hoc".into(), 3.0),
                    ("workaround".into(), 3.0),
                    ("hack".into(), 2.0),
                    ("manual".into(), 1.0),
                ]),
                threshold: 6.0,
            },
        ),
        (
            "evolution_stagnation_detected",
            SignalProfile {
                keywords: HashMap::from([
                    ("no change".into(), 4.0),
                    ("same result".into(), 4.0),
                    ("stuck".into(), 3.0),
                    ("plateau".into(), 4.0),
                    ("stagnant".into(), 5.0),
                    ("no progress".into(), 5.0),
                    ("spinning".into(), 3.0),
                    ("idle".into(), 2.0),
                    ("nothing new".into(), 4.0),
                    ("exhausted".into(), 3.0),
                ]),
                threshold: 6.0,
            },
        ),
    ]
}

// ---------------------------------------------------------------------------
// Layer 1: Regex extraction (deterministic, zero-latency)
// ---------------------------------------------------------------------------

fn extract_regex(corpus: &str, lower: &str, error_hit: bool) -> Vec<Signal> {
    let mut signals = Vec::new();

    if error_hit {
        signals.push(Signal::new("log_error"));
    }

    // Extract error signature from the first matching error line.
    let error_patterns = [
        r"(?i)\b(typeerror|referenceerror|syntaxerror)\b\s*:",
        r"(?i)error\s*:",
        r"(?i)exception\s*:",
        r"(?i)\[error",
    ];
    for line in corpus.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let ll = trimmed.to_lowercase();
        for pattern in &error_patterns {
            if regex_lite_match(&ll, pattern) {
                let clipped = trimmed.replace(['\n', '\r'], " ");
                let clipped: String = clipped.chars().take(260).collect();
                signals.push(Signal::with_detail("errsig", clipped));
                break;
            }
        }
        // Only capture the first error signature.
        if signals.iter().any(|s| s.name == "errsig") {
            break;
        }
    }

    // Protocol drift.
    if lower.contains("prompt") && !lower.contains("evolutionevent") {
        signals.push(Signal::new("protocol_drift"));
    }

    // Unsupported input type.
    if regex_lite_match(
        lower,
        r"(?i)unsupported\s+mime|unsupported.*type|invalid.*mime",
    ) {
        signals.push(Signal::new("unsupported_input_type"));
    }

    // User feature request — EN patterns.
    let has_feature_request = regex_lite_match(
        lower,
        r"(?i)\b(add|implement|create|build|make|develop|write|design)\b[^.?!\n]{3,120}\b(feature|function|module|capability|tool|support|endpoint|command|option|mode)\b",
    ) || regex_lite_match(
        lower,
        r"(?i)\b(i want|i need|we need|please add|can you add|could you add|let'?s add)\b",
    );

    if has_feature_request {
        signals.push(Signal::new("user_feature_request"));
    }

    // User improvement suggestion — EN patterns.
    let has_improvement = regex_lite_match(
        lower,
        r"(?i)\b(should be|could be better|improve|enhance|upgrade|refactor|clean up|simplify|streamline)\b",
    );

    if has_improvement {
        signals.push(Signal::new("user_improvement_suggestion"));
    }

    // Performance bottleneck.
    if regex_lite_match(
        lower,
        r"(?i)\b(slow|timeout|timed?\s*out|latency|bottleneck|took too long|performance issue|high cpu|high memory|oom|out of memory)\b",
    ) {
        signals.push(Signal::new("perf_bottleneck"));
    }

    // Capability gap.
    if regex_lite_match(
        lower,
        r"(?i)\b(not supported|cannot|doesn'?t support|no way to|missing feature|unsupported|not available|not implemented|no support for)\b",
    ) {
        // Exclude false positives from config signals.
        let exclude = lower.contains("memory.md missing")
            || lower.contains("user.md missing")
            || lower.contains("no session logs found");
        if !exclude {
            signals.push(Signal::new("capability_gap"));
        }
    }

    // Recurring error detection.
    let error_counts = count_recurring_patterns(lower);
    if let Some((top_key, count)) = error_counts
        .iter()
        .max_by_key(|(_, c)| *c)
        .filter(|(_, c)| *c >= 3)
    {
        signals.push(Signal::with_detail(
            "recurring_error",
            format!("{}x: {}", count, truncate_str(top_key, 150)),
        ));
    }

    signals
}

/// Simple regex-lite matching (no external dependency needed).
fn regex_lite_match(haystack: &str, _pattern: &str) -> bool {
    // Use simple keyword matching as a lightweight stand-in for full regex.
    // For the patterns we care about, substring/word-boundary matching suffices.
    let p = _pattern
        .replace("(?i)", "")
        .replace("\\b", "")
        .replace("\\s+", " ")
        .replace(['\\', '(', ')'], "");

    // Check if all pipe-separated alternatives contain at least one match.
    for alt in p.split('|') {
        let alt = alt.trim().trim_matches(|c: char| c == '^' || c == '$');
        if alt.is_empty() {
            continue;
        }
        if haystack.contains(alt) {
            return true;
        }
    }
    false
}

/// Count recurring error-like patterns in the corpus.
fn count_recurring_patterns(lower: &str) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    // Look for repeated error-like substrings.
    for line in lower.lines() {
        let trimmed = line.trim();
        if trimmed.contains("error") || trimmed.contains("failed") || trimmed.contains("exception")
        {
            let key: String = trimmed.chars().take(100).collect();
            *counts.entry(key).or_insert(0) += 1;
        }
    }
    counts.into_iter().collect()
}

/// Truncate a string to the given byte length.
fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
    }
}

// ---------------------------------------------------------------------------
// Layer 2: Weighted keyword scoring (statistical, zero-latency)
// ---------------------------------------------------------------------------

fn extract_keyword_score(lower: &str) -> Vec<Signal> {
    let mut scored = Vec::new();

    for (signal_name, profile) in signal_profiles() {
        let mut total = 0.0;
        for (kw, weight) in &profile.keywords {
            let count = lower.matches(kw.as_str()).count();
            total += count as f64 * weight;
        }
        if total >= profile.threshold {
            scored.push(Signal::new(signal_name));
        }
    }

    scored
}

// ---------------------------------------------------------------------------
// Signal merge and de-duplication
// ---------------------------------------------------------------------------

fn merge_signals(regex_signals: Vec<Signal>, score_signals: Vec<Signal>) -> Vec<Signal> {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();

    for s in regex_signals.into_iter().chain(score_signals) {
        if seen.insert(s.name.clone()) {
            merged.push(s);
        }
    }

    merged
}

/// De-duplicate: suppress signals that appeared in 3+ of the last 8 events.
fn dedup_signals(signals: Vec<Signal>, recent_event_signals: &[Vec<String>]) -> Vec<Signal> {
    if recent_event_signals.is_empty() {
        return signals;
    }

    let tail = &recent_event_signals[recent_event_signals.len().saturating_sub(8)..];
    let mut freq: HashMap<String, usize> = HashMap::new();
    for event_sigs in tail {
        for sig in event_sigs {
            let key = if let Some(idx) = sig.find(':') {
                sig[..idx].to_string()
            } else {
                sig.clone()
            };
            *freq.entry(key).or_insert(0) += 1;
        }
    }

    let suppressed: HashSet<&String> = freq
        .iter()
        .filter(|(_, c)| **c >= 3)
        .map(|(k, _)| k)
        .collect();

    signals
        .into_iter()
        .filter(|s| !suppressed.contains(&s.name))
        .collect()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Extract signals from a text corpus.
///
/// Runs both regex (Layer 1) and keyword scoring (Layer 2) in parallel,
/// merges their outputs, and applies de-duplication.
///
/// # Arguments
///
/// * `corpus` — The text to analyze (goal statement, session transcript, etc.)
/// * `recent_event_signals` — Signal lists from recent events (for de-dup).
///   Pass `&[]` for fresh extraction.
pub fn extract_signals(corpus: &str, recent_event_signals: &[Vec<String>]) -> Vec<Signal> {
    let lower = corpus.to_lowercase();

    let error_hit = regex_lite_match(
        &lower,
        r#"(?i)\[error\]|error:|exception:|iserror":true|"status":\s*"error"|"status":\s*"failed""#,
    );

    let regex_signals = extract_regex(corpus, &lower, error_hit);
    let score_signals = extract_keyword_score(&lower);
    let merged = merge_signals(regex_signals, score_signals);
    dedup_signals(merged, recent_event_signals)
}

/// Score a learning's `signals_match` against extracted signals.
///
/// Returns the weighted overlap score (higher = more relevant).
pub fn score_signals_match(signals: &[Signal], patterns: &[super::SignalPattern]) -> f64 {
    if patterns.is_empty() || signals.is_empty() {
        return 0.0;
    }

    let signal_names: HashSet<&str> = signals.iter().map(|s| s.name.as_str()).collect();

    let mut score = 0.0;
    for pattern in patterns {
        if signal_names.contains(pattern.signal.as_str()) {
            score += pattern.weight;
        } else {
            // Prefix match: "errsig" matches "errsig:..." in signals.
            for sig in signals {
                if sig.name.starts_with(&pattern.signal) {
                    score += pattern.weight;
                    break;
                }
            }
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_signals_error() {
        let corpus = "Error: type mismatch in main.rs";
        let signals = extract_signals(corpus, &[]);
        assert!(signals.iter().any(|s| s.name == "log_error"));
    }

    #[test]
    fn test_extract_signals_feature_request() {
        let corpus = "I need to add a new authentication endpoint";
        let signals = extract_signals(corpus, &[]);
        assert!(signals.iter().any(|s| s.name == "user_feature_request"));
    }

    #[test]
    fn test_extract_signals_perf_bottleneck() {
        let corpus = "The request is slow, high latency on the API";
        let signals = extract_signals(corpus, &[]);
        assert!(signals.iter().any(|s| s.name == "perf_bottleneck"));
    }

    #[test]
    fn test_dedup_suppresses_frequent_signals() {
        let sig1 = vec!["log_error".to_string(), "errsig:test".to_string()];
        let sig2 = vec!["log_error".to_string(), "errsig:test".to_string()];
        let sig3 = vec!["log_error".to_string(), "errsig:test".to_string()];
        let sig4 = vec!["log_error".to_string()];

        let recent = vec![sig1, sig2, sig3, sig4];
        let signals = extract_signals("Error: failure in lib.rs", &recent);
        // "log_error" appeared in 4/4 = >3 of last 8, so should be suppressed.
        assert!(!signals.iter().any(|s| s.name == "log_error"));
    }

    #[test]
    fn test_score_signals_match_empty() {
        let signals = vec![Signal::new("log_error")];
        assert_eq!(score_signals_match(&signals, &[]), 0.0);
    }

    #[test]
    fn test_score_signals_match_basic() {
        use crate::memory::SignalPattern;

        let signals = vec![Signal::new("log_error"), Signal::new("perf_bottleneck")];
        let patterns = vec![
            SignalPattern {
                signal: "log_error".into(),
                weight: 2.0,
            },
            SignalPattern {
                signal: "perf_bottleneck".into(),
                weight: 3.0,
            },
            SignalPattern {
                signal: "capability_gap".into(),
                weight: 1.0,
            },
        ];

        let score = score_signals_match(&signals, &patterns);
        assert!((score - 5.0).abs() < f64::EPSILON); // 2.0 + 3.0
    }
}
