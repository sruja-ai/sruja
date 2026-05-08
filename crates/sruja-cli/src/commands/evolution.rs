//! Evolutionary architecture fitness evaluation and tracking.

use crate::commands::error::CliError;
use crate::commands::parse_sruja_file;
use sruja_language::ast::TopLevelItem;
use std::path::Path;
use std::process::Command;

/// Run all declared fitness functions and evaluate their state
pub async fn evaluate(architecture: &str) -> Result<(), CliError> {
    let path = Path::new(architecture);
    if !path.exists() {
        return Err(CliError::validation(format!(
            "Architecture file '{}' does not exist.",
            architecture
        )));
    }

    println!("====================================================");
    println!("🧬 SRUJA EVOLUTIONARY ARCHITECTURE FITNESS EVALUATOR");
    println!("====================================================");
    println!("Reading architecture definitions from: {}", architecture);

    let (_, program) = parse_sruja_file(path)?;
    let mut fitness_functions = Vec::new();

    // 1. Collect top-level fitness definitions
    for item in &program.items {
        if let TopLevelItem::Fitness(fit) = item {
            fitness_functions.push((fit.clone(), "Top-level".to_string()));
        } else if let TopLevelItem::ElementDef(e) = item {
            // Also search elements
            if let Some(ref body) = e.assignment.body {
                for ff in &body.fitness_functions {
                    fitness_functions.push((ff.clone(), format!("Element: {}", e.assignment.name)));
                }
            }
        }
    }

    if fitness_functions.is_empty() {
        println!("\nNo fitness functions declared in '{}'.", architecture);
        println!("Add 'fitness' blocks to declare optimization goals. Example:");
        println!("  fitness AccuracyTarget {{");
        println!("    target \"success_rate > 99.0%\"");
        println!("    measure \"scripts/evaluate_accuracy.sh\"");
        println!("  }}");
        return Ok(());
    }

    println!(
        "\nFound {} fitness function(s) to evaluate:",
        fitness_functions.len()
    );

    let mut passed = 0;
    let mut failed = 0;

    for (fit, context) in &fitness_functions {
        println!("\n----------------------------------------------------");
        println!("Fitness ID : {}", fit.id);
        println!("Scope      : {}", context);
        println!("Criterion  : {}", fit.target);
        println!("Measure Cmd: {}", fit.measure);
        println!("----------------------------------------------------");

        println!("Running evaluation command...");
        // Split command for executing
        let parts: Vec<&str> = fit.measure.split_whitespace().collect();
        if parts.is_empty() {
            println!("❌ Error: measure command is empty.");
            failed += 1;
            continue;
        }

        let mut cmd = Command::new(parts[0]);
        let repo_dir = path.parent().unwrap_or(Path::new("."));
        if repo_dir.as_os_str() != "" {
            cmd.current_dir(repo_dir);
        }
        for arg in &parts[1..] {
            cmd.arg(arg);
        }

        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !stdout.is_empty() {
                    println!("\n[Stdout]\n{}", stdout.trim());
                }
                if !stderr.is_empty() {
                    println!("\n[Stderr]\n{}", stderr.trim());
                }

                if output.status.success() {
                    println!("\n🎯 Result: [PASS] (Score matched fitness target)");
                    passed += 1;
                    // Log the success mutation
                    let _ = log_mutation(repo_dir, &fit.id, &fit.target, "pass", &stdout);
                } else {
                    println!("\n⚠️ Result: [FAIL] (Fitness target not met)");
                    failed += 1;
                    let _ = log_mutation(repo_dir, &fit.id, &fit.target, "fail", &stdout);
                }
            }
            Err(e) => {
                println!("\n❌ Execution Failed: {}", e);
                failed += 1;
                let _ = log_mutation(repo_dir, &fit.id, &fit.target, "error", &e.to_string());
            }
        }
    }

    println!("\n====================================================");
    println!("Evaluation Summary: {} Passed, {} Failed", passed, failed);
    println!("====================================================");

    Ok(())
}

/// Show evolution history of mutations
pub async fn evolution_log(repo: &str) -> Result<(), CliError> {
    let log_path = Path::new(repo).join(".sruja").join("evolution.log");
    println!("====================================================");
    println!("📜 SRUJA EVOLUTIONARY MUTATION HISTORY LOG");
    println!("====================================================");

    if !log_path.exists() {
        println!("No evolution history found under .sruja/evolution.log.");
        println!("Run 'sruja evaluate' to execute fitness functions and populate history.");
        return Ok(());
    }

    let contents = std::fs::read_to_string(log_path)?;
    println!("{}", contents);

    Ok(())
}

fn log_mutation(
    repo_dir: &Path,
    id: &str,
    target: &str,
    result: &str,
    detail: &str,
) -> std::io::Result<()> {
    let sruja_dir = repo_dir.join(".sruja");
    if !sruja_dir.exists() {
        std::fs::create_dir_all(&sruja_dir)?;
    }
    let log_path = sruja_dir.join("evolution.log");
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let detail_line = detail.lines().next().unwrap_or("").trim();
    writeln!(
        file,
        "[{}] ID: {} | Target: {} | Result: {} | Output: {}",
        timestamp,
        id,
        target,
        result.to_uppercase(),
        detail_line
    )?;
    Ok(())
}
