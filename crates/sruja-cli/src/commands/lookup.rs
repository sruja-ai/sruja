//! `sruja lookup` — fetch a compact concept card for an architecture element.
//!
//! Native CLI front-end for `Graph::concept_card` (and the DSL equivalent), so the
//! sruja agent can call it directly via [`SrujaLookupTool`]. Output is a small,
//! schema-versioned JSON envelope — the deterministic, token-cheap alternative to
//! pulling a whole `focus` briefing (or a whole file) when only one element is needed.

use std::path::Path;

use super::CliError;

/// Fetch a compact concept card for the element matching `name`.
pub async fn lookup(name: &str, repo: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);

    // Prefer the DSL program (grounded, author-reviewed truth); fall back to scan.
    let (arch, warning) = crate::commands::scan_domain::mcp::helpers::load_architecture_program_best_effort(repo_path);
    let out = if let Some((source_file, program)) = arch {
        crate::commands::scan_domain::mcp::ladder::build_concept_card_from_program(
            &source_file,
            &program,
            name,
            warning.as_deref(),
        )?
    } else {
        let graph = crate::commands::scan_repo_cached(repo_path)?;
        crate::commands::scan_domain::mcp::ladder::build_concept_card_from_scan(
            &graph,
            name,
            warning.as_deref(),
        )?
    };

    match format {
        "json" => println!("{out}"),
        _ => println!("{out}"),
    }
    Ok(())
}
