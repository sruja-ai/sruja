use super::CliError;
use super::types::OutputFormat;
use sruja_diff::Proposal;
use std::path::Path;

pub async fn propose_list(repo_root: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    let format = OutputFormat::parse(format)?;
    let proposals = Proposal::load_all(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    if proposals.is_empty() {
        match format {
            OutputFormat::Text => println!("No proposals found."),
            OutputFormat::Json => println!("[]"),
        }
        return Ok(());
    }

    let mut proposals = proposals;
    proposals.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    match format {
        OutputFormat::Text => {
            println!(
                "{:<12} {:<10} {:<24} Description",
                "ID", "Status", "Created"
            );
            println!("{}", "-".repeat(80));
            for p in proposals {
                println!(
                    "{:<12} {:<10} {:<24} {}",
                    p.id,
                    format!("{:?}", p.status),
                    p.created_at,
                    p.description
                );
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&proposals)?);
        }
    }

    Ok(())
}
