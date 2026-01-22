//! SLO validation rule
//!
//! Mirrors Go `SLOValidationRule`:
//! - Validates formats inside `slo { ... }` blocks for availability/latency/errorRate/throughput.
//! - Emits CODE_VALIDATION_RULE_ERROR diagnostics with severity aligned to Go.

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{collect_elements, Program, SloBlock};

use crate::validator::Rule;

pub struct SloValidationRule;

impl Rule for SloValidationRule {
    fn name(&self) -> &str {
        "SLO Validation"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        let (elements, _relations) = collect_elements(program);
        let mut diags: Vec<Diagnostic> = Vec::with_capacity(10);

        for (_fqn, elem) in &elements {
            if let Some(body) = &elem.assignment.body {
                if let Some(slo) = &body.slo {
                    diags.extend(validate_slo_block(slo, &elem.location));
                }
            }
        }

        diags
    }
}

fn validate_slo_block(slo: &SloBlock, fallback_loc: &sruja_diagnostics::SourceLocation) -> Vec<Diagnostic> {
    let mut diags: Vec<Diagnostic> = Vec::with_capacity(5);

    // At least one SLO type present
    if slo.availability.is_none()
        && slo.latency.is_none()
        && slo.error_rate.is_none()
        && slo.throughput.is_none()
    {
        diags.push(Diagnostic::new(
            sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
            Severity::Warning,
            "SLO block should define at least one SLO type (availability, latency, errorRate, or throughput)",
            fallback_loc.clone(),
        ));
        return diags;
    }

    if let Some(av) = &slo.availability {
        if let Some(target) = &av.target {
            if !is_valid_percentage(target) {
                diags.push(Diagnostic::new(
                    sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                    Severity::Error,
                    format!("Availability target '{}' must be a percentage (e.g., '99.9%')", target),
                    fallback_loc.clone(),
                ));
            }
        }
        if let Some(window) = &av.window {
            if !is_valid_time_window(window) {
                diags.push(Diagnostic::new(
                    sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                    Severity::Warning,
                    format!(
                        "Availability window '{}' should be a time period (e.g., '30 days', '7 days')",
                        window
                    ),
                    fallback_loc.clone(),
                ));
            }
        }
        if let Some(current) = &av.current {
            if !is_valid_percentage(current) {
                diags.push(Diagnostic::new(
                    sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                    Severity::Error,
                    format!("Availability current '{}' must be a percentage (e.g., '99.95%')", current),
                    fallback_loc.clone(),
                ));
            }
        }
    }

    if let Some(lat) = &slo.latency {
        if let Some(p95) = &lat.p95 {
            if !is_valid_duration(p95) {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                        Severity::Error,
                        format!("Latency p95 '{}' must be a duration (e.g., '200ms', '1s')", p95),
                        fallback_loc.clone(),
                    )
                    .with_suggestions(vec![
                        "Use duration format: '200ms', '1s', '500ms', '2s'".to_string(),
                        "Common values: '100ms' (fast), '200ms' (good), '500ms' (acceptable)".to_string(),
                    ]),
                );
            }
        }
        if let Some(p99) = &lat.p99 {
            if !is_valid_duration(p99) {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                        Severity::Error,
                        format!("Latency p99 '{}' must be a duration (e.g., '500ms', '2s')", p99),
                        fallback_loc.clone(),
                    )
                    .with_suggestions(vec![
                        "Use duration format: '500ms', '2s', '1s', '3s'".to_string(),
                        "p99 should typically be 2-5x higher than p95".to_string(),
                    ]),
                );
            }
        }
        if let Some(window) = &lat.window {
            if !is_valid_time_window(window) {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                        Severity::Warning,
                        format!(
                            "Latency window '{}' should be a time period (e.g., '7 days', '30 days')",
                            window
                        ),
                        fallback_loc.clone(),
                    )
                    .with_suggestions(vec![
                        "Use time period format: '7 days', '30 days', '1 week', '1 month'".to_string(),
                        "Common windows: '7 days' (weekly), '30 days' (monthly)".to_string(),
                    ]),
                );
            }
        }
        if let Some(cur) = &lat.current {
            if let Some(p95) = &cur.p95 {
                if !is_valid_duration(p95) {
                    diags.push(
                        Diagnostic::new(
                            sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                            Severity::Error,
                            format!("Latency current p95 '{}' must be a duration", p95),
                            fallback_loc.clone(),
                        )
                        .with_suggestions(vec!["Use duration format: '200ms', '1s', '500ms'".to_string()]),
                    );
                }
            }
            if let Some(p99) = &cur.p99 {
                if !is_valid_duration(p99) {
                    diags.push(
                        Diagnostic::new(
                            sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                            Severity::Error,
                            format!("Latency current p99 '{}' must be a duration", p99),
                            fallback_loc.clone(),
                        )
                        .with_suggestions(vec!["Use duration format: '500ms', '2s', '1s'".to_string()]),
                    );
                }
            }
        }
    }

    if let Some(er) = &slo.error_rate {
        if let Some(target) = &er.target {
            if !is_valid_percentage(target) {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                        Severity::Error,
                        format!("Error rate target '{}' must be a percentage (e.g., '0.1%')", target),
                        fallback_loc.clone(),
                    )
                    .with_suggestions(vec![
                        "Use percentage format: '0.1%', '0.01%', '1%'".to_string(),
                        "Common targets: '0.1%' (99.9% success), '0.01%' (99.99% success)".to_string(),
                    ]),
                );
            }
        }
        if let Some(window) = &er.window {
            if !is_valid_time_window(window) {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                        Severity::Warning,
                        format!("Error rate window '{}' should be a time period", window),
                        fallback_loc.clone(),
                    )
                    .with_suggestions(vec!["Use time period format: '7 days', '30 days', '1 week'".to_string()]),
                );
            }
        }
        if let Some(current) = &er.current {
            if !is_valid_percentage(current) {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                        Severity::Error,
                        format!("Error rate current '{}' must be a percentage", current),
                        fallback_loc.clone(),
                    )
                    .with_suggestions(vec!["Use percentage format: '0.1%', '0.5%', '1%'".to_string()]),
                );
            }
        }
    }

    if let Some(tp) = &slo.throughput {
        if let Some(target) = &tp.target {
            if !is_valid_rate(target) {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                        Severity::Warning,
                        format!(
                            "Throughput target '{}' should be a rate (e.g., '10000 req/s', '1000/s')",
                            target
                        ),
                        fallback_loc.clone(),
                    )
                    .with_suggestions(vec![
                        "Use rate format: '10000 req/s', '1000/s', '500 ops/min'".to_string(),
                        "Specify units: 'req/s', '/s', 'ops/min', 'requests/hour'".to_string(),
                    ]),
                );
            }
        }
        if let Some(window) = &tp.window {
            if !is_valid_time_window(window) {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                        Severity::Warning,
                        format!("Throughput window '{}' should be a time period", window),
                        fallback_loc.clone(),
                    )
                    .with_suggestions(vec!["Use time period format: '1 hour', '1 day', '1 week'".to_string()]),
                );
            }
        }
        if let Some(current) = &tp.current {
            if !is_valid_rate(current) {
                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_VALIDATION_RULE_ERROR,
                        Severity::Warning,
                        format!("Throughput current '{}' should be a rate", current),
                        fallback_loc.clone(),
                    )
                    .with_suggestions(vec!["Use rate format: '5000 req/s', '1000/s', '200 ops/min'".to_string()]),
                );
            }
        }
    }

    diags
}

fn is_valid_percentage(s: &str) -> bool {
    let s = s.trim();
    let Some(num) = s.strip_suffix('%') else { return false };
    is_number(num)
}

fn is_valid_duration(s: &str) -> bool {
    let s = s.trim();
    for unit in ["ms", "s", "m", "h"] {
        if let Some(num) = s.strip_suffix(unit) {
            return is_number(num.trim());
        }
    }
    false
}

fn is_valid_time_window(s: &str) -> bool {
    let lower = s.trim().to_lowercase();
    let parts: Vec<&str> = lower.split_whitespace().collect();
    if parts.len() != 2 {
        return false;
    }
    if parts[0].parse::<u64>().is_err() {
        return false;
    }
    matches!(
        parts[1],
        "day" | "days" | "hour" | "hours" | "week" | "weeks" | "month" | "months"
    )
}

fn is_valid_rate(s: &str) -> bool {
    // Very small parser matching Go's regex: ^\d+(\s+\w+)?/\w+$
    let s = s.trim();
    let Some((lhs, rhs)) = s.split_once('/') else { return false };
    if rhs.trim().is_empty() {
        return false;
    }
    let lhs = lhs.trim();
    if lhs.is_empty() {
        return false;
    }
    // lhs: "<num>" or "<num> <word>"
    let mut it = lhs.split_whitespace();
    let Some(num) = it.next() else { return false };
    if !num.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return false;
    }
    // optional unit word
    if let Some(extra) = it.next() {
        if !extra.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return false;
        }
    }
    // no more parts
    it.next().is_none()
}

fn is_number(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    // digits or digits '.' digits
    let mut seen_dot = false;
    let mut seen_digit = false;
    for c in s.chars() {
        if c.is_ascii_digit() {
            seen_digit = true;
            continue;
        }
        if c == '.' && !seen_dot {
            seen_dot = true;
            continue;
        }
        return false;
    }
    seen_digit
}

