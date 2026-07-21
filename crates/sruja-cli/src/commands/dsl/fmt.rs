use std::fs;
use std::path::Path;

use sruja_export::dsl::printer::DslPrinter;

use crate::modules::validation::enrich_diagnostics_with_source;
use crate::commands::CliError;

pub async fn fmt(file: &str, check: bool) -> Result<(), CliError> {
    let path = Path::new(file);
    if !path.exists() {
        return Err(CliError::validation(format!(
            "Architecture file '{}' does not exist.",
            file
        )));
    }

    let content = fs::read_to_string(path)?;
    let parser = sruja_language::Parser::new(file.to_string());
    let program = match parser.parse(&content) {
        Ok(p) => p,
        Err(mut diagnostics) => {
            enrich_diagnostics_with_source(&content, &mut diagnostics);
            for diag in &diagnostics {
                eprintln!("{}", sruja_diagnostics::format_diagnostic(diag));
            }
            return Err(CliError::Parse {
                file: file.to_string(),
                message: format!("Parsing failed with {} errors", diagnostics.len()),
                diagnostics,
                help: Some("Fix the syntax errors, then re-run 'sruja fmt'.".into()),
                fix: Some("Correct invalid DSL syntax and run the command again.".into()),
            });
        }
    };

    let printer = DslPrinter::new();
    let mut formatted = printer.print(&program);
    if !formatted.ends_with('\n') {
        formatted.push('\n');
    }

    if check {
        if formatted != content {
            return Err(CliError::validation(
                "File is not formatted. Run `sruja fmt <file>`.".to_string(),
            ));
        }
        return Ok(());
    }

    if formatted != content {
        fs::write(path, formatted)?;
    }

    Ok(())
}
