use std::fs;
use std::path::Path;

use sruja_diagnostics::{format_diagnostic, format_github_actions_annotation};
use sruja_engine::Validator;

use crate::modules::validation::enrich_diagnostics_with_source;
use crate::commands::CliError;

pub async fn lint(
    file: &str,
    format: &str,
    baseline: Option<&str>,
    write_baseline: Option<&str>,
) -> Result<(), CliError> {
    let github = matches!(format, "github" | "github-actions");
    let content = fs::read_to_string(file)?;
    let parser = sruja_language::Parser::new(file.to_string());

    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(mut diagnostics) => {
            enrich_diagnostics_with_source(&content, &mut diagnostics);
            if format == "json" {
                let out = super::lint_diagnostics_to_json(Path::new(file), &diagnostics, false)?;
                println!(
                    "{}",
                    serde_json::to_string(&out).map_err(|e| CliError::validation(e.to_string()))?
                );
                return Err(CliError::Parse {
                    file: file.to_string(),
                    message: format!("Parsing failed with {} errors", diagnostics.len()),
                    diagnostics,
                    help: Some("Fix the syntax errors reported by the parser.".into()),
                    fix: Some(
                        "After fixing syntax, run 'sruja fmt' (optional) and re-run 'sruja lint'."
                            .into(),
                    ),
                });
            }
            if github {
                for diag in &diagnostics {
                    println!("{}", format_github_actions_annotation(diag));
                }
                return Err(CliError::Parse {
                    file: file.to_string(),
                    message: format!("Parsing failed with {} errors", diagnostics.len()),
                    diagnostics,
                    help: Some(
                        "Check the GitHub Actions annotations for the exact location.".into(),
                    ),
                    fix: Some("Fix the syntax errors in the file, then re-run the check.".into()),
                });
            }
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag));
            }
            return Err(CliError::Parse {
                file: file.to_string(),
                message: format!("Parsing failed with {} errors", diagnostics.len()),
                diagnostics,
                help: Some("Review the diagnostics printed above for the exact location.".into()),
                fix: Some("Fix the syntax errors in the file, then re-run 'sruja lint'.".into()),
            });
        }
    };

    let validator = Validator::with_default_rules();
    let mut diagnostics = validator.validate_sync(&program);
    enrich_diagnostics_with_source(&content, &mut diagnostics);
    super::sort_diagnostics(&mut diagnostics);

    if let Some(out_path) = write_baseline {
        super::write_lint_baseline(out_path, &diagnostics)?;
        if format == "json" {
            let mut out = super::lint_diagnostics_to_json(Path::new(file), &diagnostics, true)?;
            out.baseline = Some(out_path.to_string());
            out.total_error_count = Some(out.error_count);
            out.total_warning_count = Some(out.warning_count);
            out.error_count = 0;
            out.warning_count = 0;
            out.diagnostics.clear();
            println!("{}", serde_json::to_string(&out)?);
        } else {
            println!("Wrote baseline: {}", out_path);
        }
        return Ok(());
    }

    let (filtered_diagnostics, total_error_count, total_warning_count) =
        super::apply_lint_baseline(baseline, &diagnostics)?;

    if format == "json" {
        let error_count = filtered_diagnostics
            .iter()
            .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
            .count();
        let mut out =
            super::lint_diagnostics_to_json(Path::new(file), &filtered_diagnostics, error_count == 0)?;
        out.baseline = baseline.map(|s| s.to_string());
        out.total_error_count = Some(total_error_count);
        out.total_warning_count = Some(total_warning_count);
        println!(
            "{}",
            serde_json::to_string(&out).map_err(|e| CliError::validation(e.to_string()))?
        );
        if error_count > 0 {
            return Err(CliError::validation(format!(
                "Linting failed with {} errors",
                error_count
            )));
        }
        return Ok(());
    }

    if github {
        let error_count = filtered_diagnostics
            .iter()
            .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
            .count();
        for diag in &filtered_diagnostics {
            println!("{}", format_github_actions_annotation(diag));
        }
        if error_count > 0 {
            return Err(CliError::validation(format!(
                "Linting failed with {} errors",
                error_count
            )));
        }
        return Ok(());
    }

    if filtered_diagnostics.is_empty() {
        println!("✓ No issues found");
        return Ok(());
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for diag in filtered_diagnostics {
        match diag.severity {
            sruja_diagnostics::Severity::Error => errors.push(diag),
            sruja_diagnostics::Severity::Warning => warnings.push(diag),
            _ => {}
        }
    }

    for warning in &warnings {
        eprintln!("{}", format_diagnostic(warning));
    }

    for error in &errors {
        eprintln!("{}", format_diagnostic(error));
    }

    if !errors.is_empty() {
        use crate::utils::colors;
        println!("──────────────────────────────────────────────");
        println!(
            "{} Found {} error(s) and {} warning(s)",
            colors::error("✗"),
            errors.len(),
            warnings.len()
        );
        return Err(CliError::validation(format!(
            "Linting failed with {} errors",
            errors.len()
        )));
    }

    if !warnings.is_empty() {
        use crate::utils::colors;
        println!("──────────────────────────────────────────────");
        println!(
            "{} Found {} warning(s) (no errors)",
            colors::success("✓"),
            warnings.len()
        );
    } else {
        use crate::utils::colors;
        println!("──────────────────────────────────────────────");
        println!("{} No issues found", colors::success("✓"));
    }

    Ok(())
}
