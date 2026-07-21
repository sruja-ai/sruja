use crate::commands;
use crate::commands::CliError;

use super::super::commands::*;
use super::super::subcommands::{DslCommand, InspectCommand};

pub(super) async fn handle(command: Commands) -> Result<(), CliError> {
    match command {
        Commands::Dsl(DslArgs { cmd }) => match cmd {
            DslCommand::List { file } => commands::list_elements(&file).await,
            DslCommand::Tree { file } => commands::tree(&file).await,
            DslCommand::Diff {
                file1,
                file2,
                format,
            } => commands::diff(&file1, &file2, &format).await,
            DslCommand::Explain {
                element_id,
                file,
                json,
            } => commands::explain(&element_id, file.as_deref(), json).await,
            DslCommand::Import { format, file } => commands::import(&format, &file).await,
            DslCommand::Compile { file } => commands::compile(&file).await,
            DslCommand::Validate {
                file,
                constraints,
                fail_on_violations,
                format_json,
            } => commands::validate(&file, constraints, fail_on_violations, format_json).await,
            DslCommand::Generate {
                repo,
                skill_path,
                prompt_only,
                output,
            } => {
                if !prompt_only {
                    eprintln!("Only --prompt-only is supported. Use: sruja dsl generate -r . --prompt-only -o prompt.txt");
                    eprintln!("Then use the prompt with any LLM; save output as architecture.sruja and run sruja lint.");
                    std::process::exit(1);
                }
                commands::generate_prompt(&repo, skill_path.as_deref(), output.as_deref())
            }
        },
        Commands::Inspect(InspectArgs { cmd }) => match cmd {
            InspectCommand::Health {
                repo,
                architecture,
                format,
            } => commands::health(&repo, architecture.as_deref(), &format).await,
            InspectCommand::Impact {
                target,
                repo,
                depth,
                format,
            } => commands::impact(&repo, &target, depth, &format).await,
            InspectCommand::Why {
                question,
                repo,
                format,
                reasoned,
                llmguided,
            } => commands::why(&repo, &question, &format, reasoned, llmguided).await,
            InspectCommand::Query {
                query,
                repo,
                architecture,
                format,
            } => commands::query_registry(&repo, architecture.as_deref(), &query, &format).await,
            InspectCommand::ContextScore {
                repo,
                format,
                fail_under,
            } => commands::context_score(&repo, &format, fail_under).await,
            InspectCommand::ContextGraph { repo, output, open } => {
                commands::context_graph(&repo, &output, open).await
            }
            InspectCommand::Onboard {
                repo,
                format,
                max_items,
                ref enrich,
                output,
            } => {
                commands::onboard(
                    &repo,
                    &format,
                    max_items,
                    &enrich.as_ref(),
                    output.as_deref(),
                )
                .await
            }
            InspectCommand::Quickstart {
                path,
                format,
                generate_baseline,
                fail_on,
                advisory,
            } => {
                commands::quickstart(
                    &path,
                    &format,
                    generate_baseline,
                    fail_on.as_deref(),
                    advisory,
                )
                .await
            }
            InspectCommand::Watch { path, clear, focus } => {
                commands::watch(&path, clear, focus).await
            }
            InspectCommand::Learn {
                path,
                file,
                since,
                skip_proposals,
                apply_proposals,
                format,
            } => {
                let skip = skip_proposals || !apply_proposals;
                commands::learn(&path, file.as_deref(), since.as_deref(), skip, &format).await
            }
            InspectCommand::Ingest {
                sources,
                repo,
                category,
                elements,
            } => {
                commands::ingest(&repo, &sources, category.as_deref(), elements.as_deref())
                    .await
            }
        },
        _ => unreachable!(),
    }
}
