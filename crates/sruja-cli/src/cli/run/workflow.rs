use crate::commands;

use super::super::subcommands::{AidlcCommand, WorkflowCommand};

pub(super) async fn handle_workflow(cmd: WorkflowCommand) -> Result<(), crate::commands::CliError> {
    match cmd {
        WorkflowCommand::Init {
            repo,
            title,
            id,
            target_elements,
            strict_gates,
            with_aidlc,
            aidlc_profile,
            install_aidlc_rules,
            profile,
            template,
        } => commands::workflow_init(
            &repo,
            &title,
            id.as_deref(),
            target_elements,
            strict_gates,
            commands::WorkflowInitOptions {
                with_aidlc,
                aidlc_profile,
                install_rules: install_aidlc_rules,
                profile,
                template,
            },
        ),
        WorkflowCommand::List { repo } => commands::workflow_list(&repo),
        WorkflowCommand::Status { repo, id, check } => {
            commands::workflow_status(&repo, id.as_deref(), check)
        }
        WorkflowCommand::RecordImpact { repo, id, depth } => {
            commands::workflow_record_impact(&repo, &id, depth)
        }
        WorkflowCommand::Approve {
            repo,
            id,
            phase,
            by,
        } => commands::workflow_approve(&repo, &id, &phase, by.as_deref()),
        WorkflowCommand::Advance { repo, id } => commands::workflow_advance(&repo, &id),
        WorkflowCommand::InstallRules { repo } => commands::workflow_install_rules(&repo),
        WorkflowCommand::Validate { repo, id } => {
            commands::workflow_validate(&repo, id.as_deref())
        }
        WorkflowCommand::Audit {
            repo,
            id,
            event,
            by,
        } => commands::workflow_audit(&repo, &id, &event, by.as_deref()),
        WorkflowCommand::Trace {
            repo,
            id,
            format,
            check,
        } => commands::workflow_trace(&repo, &id, &format, check),
        WorkflowCommand::Run {
            repo,
            id,
            vision,
            dry_run,
        } => commands::workflow_run(&repo, &id, std::path::Path::new(&vision), dry_run),
        WorkflowCommand::DesignReview {
            repo,
            id,
            output,
            enrich_cmd,
        } => {
            commands::review_design(
                &repo,
                &id,
                output.as_deref().map(std::path::Path::new),
                enrich_cmd.as_deref(),
            )
            .await
        }
        WorkflowCommand::CaptureRequirements {
            repo,
            id,
            from_issue,
            enrich_cmd,
        } => commands::workflow_capture_requirements(
            &repo,
            id.as_deref(),
            from_issue.as_deref(),
            enrich_cmd.as_deref(),
        ),
        WorkflowCommand::RecordTestResults {
            repo,
            id,
            profile,
            from_file,
        } => {
            commands::workflow_record_test_results(
                &repo,
                id.as_deref(),
                profile.as_deref(),
                from_file.as_deref(),
            )
            .await
        }
        WorkflowCommand::RecordReadiness { repo, id } => {
            commands::workflow_record_readiness(&repo, id.as_deref())
        }
        WorkflowCommand::Summary { repo, id, format } => {
            commands::workflow_summary(&repo, id.as_deref(), &format)
        }
        WorkflowCommand::NextSteps { repo, id } => {
            commands::workflow_next_steps(&repo, id.as_deref())
        }
    }
}

pub(super) async fn handle_aidlc(cmd: AidlcCommand) -> Result<(), crate::commands::CliError> {
    match cmd {
        AidlcCommand::Init {
            repo,
            title,
            id,
            profile,
            template,
            target_elements,
        } => commands::workflow_init(
            &repo,
            &title,
            id.as_deref(),
            target_elements,
            true,
            commands::WorkflowInitOptions {
                with_aidlc: true,
                aidlc_profile: profile.clone(),
                install_rules: true,
                profile,
                template,
            },
        ),
        AidlcCommand::Status { repo, id, check } => {
            commands::workflow_status(&repo, id.as_deref(), check)
        }
        AidlcCommand::Validate { repo, id } => commands::workflow_validate(&repo, id.as_deref()),
        AidlcCommand::NextSteps { repo, id } => commands::workflow_next_steps(&repo, id.as_deref()),
        AidlcCommand::InstallRules { repo } => commands::workflow_install_rules(&repo),
        AidlcCommand::Summary { repo, id, format } => {
            commands::workflow_summary(&repo, id.as_deref(), &format)
        }
    }
}
