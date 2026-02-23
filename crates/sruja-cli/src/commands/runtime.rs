//! Runtime trace commands.

use std::fs;
use std::path::Path;

use sruja_runtime::{build_report, ExecutionTrace};

use super::CliError;

/// Load traces from JSON file. Accepts array of ExecutionTrace or single ExecutionTrace.
pub(crate) fn load_traces(path: &Path) -> Result<Vec<ExecutionTrace>, CliError> {
    let content = fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&content)?;
    let traces = match value {
        serde_json::Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for v in arr {
                let t: ExecutionTrace = serde_json::from_value(v).map_err(|e| CliError::Parse {
                    file: "traces".to_string(),
                    message: format!("Invalid trace: {}", e),
                })?;
                out.push(t);
            }
            out
        }
        serde_json::Value::Object(_) => {
            let t: ExecutionTrace = serde_json::from_value(value).map_err(|e| CliError::Parse {
                file: "traces".to_string(),
                message: format!("Invalid trace: {}", e),
            })?;
            vec![t]
        }
        _ => {
            return Err(CliError::Parse {
                file: "traces".to_string(),
                message: "Expected JSON array or object".to_string(),
            })
        }
    };
    Ok(traces)
}

pub async fn runtime_analyze(traces_path: &str, format: &str) -> Result<(), CliError> {
    let path = Path::new(traces_path);
    if !path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Traces file not found: {}", traces_path),
        )));
    }

    let traces = load_traces(path)?;
    let report = build_report(&traces);

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    eprintln!("{}", "═".repeat(70));
    eprintln!("⏱ Sruja Runtime Analysis");
    eprintln!("{}", "═".repeat(70));
    eprintln!();
    eprintln!("📊 Summary");
    eprintln!("   Root traces: {}", report.trace_count);
    eprintln!("   Total spans: {}", report.total_spans);
    eprintln!("   Max depth: {}", report.max_depth);
    eprintln!("   Total duration: {} ms", report.total_duration_ms);
    if !report.hotspots.is_empty() {
        eprintln!();
        eprintln!("🔥 Hotspots (by duration)");
        for h in report.hotspots.iter().take(5) {
            eprintln!(
                "   {} ({} ms, {}x)",
                h.span_name, h.total_duration_ms, h.count
            );
        }
        if report.hotspots.len() > 5 {
            eprintln!("   ... and {} more", report.hotspots.len() - 5);
        }
    }
    if !report.emergent_cycles.is_empty() {
        eprintln!();
        eprintln!("🔄 Emergent Cycles");
        for c in report.emergent_cycles.iter().take(5) {
            eprintln!(
                "   {} (occurrences: {}, {:?})",
                c.pattern.join(" → "),
                c.occurrences,
                c.severity
            );
        }
        if report.emergent_cycles.len() > 5 {
            eprintln!("   ... and {} more", report.emergent_cycles.len() - 5);
        }
    }
    eprintln!();
    eprintln!("{}", "═".repeat(70));

    Ok(())
}
