use std::path::Path;
use crate::commands::CliError;
use crate::utils::{colors, progress};
use crate::scoring::health::calculate_health;

pub async fn health(repo_root: &str, architecture: Option<&str>) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    
    // 1. Parse architecture file
    let arch_path = crate::utils::architecture_path::resolve_architecture_path_or_default(repo_path, architecture);
    let (_content, program) = super::parse_sruja_file(arch_path.to_str().unwrap())?;

    // 2. Run drift detection to get violations
    let pb = progress::spinner("Calculating health score...");
    let graph = match sruja_scan::scan_repo(repo_path) {
        Ok(g) => g,
        Err(e) => {
            pb.abandon();
            return Err(CliError::scan_with_help(
                e.to_string(),
                "Ensure your repo has readable source files and your ignore rules are correct.",
            ));
        }
    };

    let proposed_graph = sruja_diff::program_to_graph(&program);
    let diff = sruja_diff::compare_graphs(&graph, &proposed_graph);
    pb.finish_and_clear();

    // 3. Calculate score
    let health = calculate_health(&diff.violations, &program);

    // 4. Report
    colors::print_header("🩺 Architecture Health Report");
    
    let score_color = if health.score >= 90 {
        colors::success(health.score)
    } else if health.score >= 70 {
        colors::info(health.score)
    } else if health.score >= 50 {
        colors::warning(health.score)
    } else {
        colors::error(health.score)
    };

    println!("Score: {}/100", score_color);
    println!();

    if health.deductions.is_empty() {
        println!("{} Your architecture is in perfect health!", colors::success("✨"));
    } else {
        println!("{}", colors::style("Deductions:").bold());
        for d in &health.deductions {
            println!("  {} {} (-{} pts)", colors::error("-"), colors::dim(&d.message), d.points);
        }
        
        println!();
        println!("{}", colors::style("Recommendations:").bold());
        if health.score < 90 {
            println!("  • Resolve architectural drift using 'sruja drift --fix'");
            println!("  • Add missing descriptions to components in your .sruja files");
            println!("  • Ensure all components are linked to a system or container (no orphans)");
        } else {
            println!("  • Your architecture is looking great! Keep maintaining it.");
        }
    }

    Ok(())
}
