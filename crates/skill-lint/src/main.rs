use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "skill-lint")]
#[command(about = "Validate and check skill rule files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Validate skill files against metadata schema")]
    Validate {
        #[arg(short, long)]
        schema: PathBuf,
        #[arg(default_value = "skills/")]
        path: PathBuf,
    },
    #[command(about = "Check skill files for common issues")]
    Check {
        #[arg(default_value = "skills/")]
        path: PathBuf,
    },
    #[command(about = "Test code examples in skill files")]
    Test {
        #[arg(short, long)]
        generate_code: bool,
        #[arg(default_value = "skills/")]
        path: PathBuf,
    },
    #[command(about = "Check for broken links in skill files")]
    CheckLinks {
        #[arg(default_value = "skills/")]
        path: PathBuf,
    },
    #[command(about = "Check for broken cross-references")]
    CheckXrefs {
        #[arg(default_value = "skills/")]
        path: PathBuf,
    },
    #[command(about = "Format skill files")]
    Format {
        #[arg(short, long)]
        check: bool,
        #[arg(default_value = "skills/")]
        path: PathBuf,
    },
    #[command(about = "Suggest rules based on project context")]
    Suggest {
        #[arg(short, long)]
        project: Option<PathBuf>,
        #[arg(short, long)]
        file: Option<PathBuf>,
        #[arg(short, long)]
        top: bool,
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        #[arg(default_value = "skills/")]
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { schema, path } => {
            commands::validate::run(schema, path).await?;
        }
        Commands::Check { path } => {
            commands::check::run(path).await?;
        }
        Commands::Test {
            generate_code,
            path,
        } => {
            commands::test::run(path, generate_code).await?;
        }
        Commands::CheckLinks { path } => {
            commands::check_links::run(path).await?;
        }
        Commands::CheckXrefs { path } => {
            commands::check_xrefs::run(path).await?;
        }
        Commands::Format { check, path } => {
            commands::format::run(path, check).await?;
        }
        Commands::Suggest {
            path,
            project,
            file,
            top,
            limit,
        } => {
            commands::suggest::run(path, project, file, top, limit, None).await?;
        }
    }

    Ok(())
}

pub mod checker;
pub mod commands;
pub mod context;
pub mod error;
