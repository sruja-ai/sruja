use colored::Colorize;
use sruja_agent::{Reversibility, Verdict};

use crate::utils::colors;
use super::types::{BoundaryInfo, FocusBriefing, truncate};

pub fn print_focus_briefing(b: &FocusBriefing) {
    let width = 56;
    let border = "─".repeat(width);

    println!();
    println!(
        "╭─ {} {} ─{}╮",
        "Context Focus:".bold(),
        colors::info(&b.target.label),
        "─".repeat(width.saturating_sub(18 + b.target.label.len()))
    );
    println!("│{:width$}│", "", width = width);

    println!(
        "│  📍 Component: {:width$}│",
        colors::style(&b.target.id).bold(),
        width = width - 17
    );
    if let Some(run_id) = &b.run_id {
        println!("│  🧾 Run ID:    {:width$}│", run_id, width = width - 17);
    }
    if let Some(ref sys) = b.target.system {
        println!("│  🏗  System:    {:width$}│", sys, width = width - 17);
    }
    if let Some(ref tech) = b.target.technology {
        println!("│  🔧 Technology: {:width$}│", tech, width = width - 18);
    }

    if !b.target.gotchas.is_empty() {
        println!(
            "│  💡 Gotchas:    {} recorded{:width$}│",
            b.target.gotchas.len(),
            "",
            width = width - 30
        );
    }
    if !b.target.operational_constraints.is_empty() {
        println!(
            "│  ⚠️  Constraints: {} recorded{:width$}│",
            b.target.operational_constraints.len(),
            "",
            width = width - 30
        );
    }

    println!("│{:width$}│", "", width = width);

    let risk = if b.blast_radius.total_affected > 10 {
        colors::error("HIGH").to_string()
    } else if b.blast_radius.total_affected > 5 {
        colors::warning("MEDIUM").to_string()
    } else {
        colors::success("LOW").to_string()
    };
    println!(
        "│  Blast Radius: {} components affected{:width$}│",
        b.blast_radius.total_affected,
        "",
        width = width - 42
    );
    println!(
        "│  Risk Level:   {}{:width$}│",
        risk,
        "",
        width = width - 20
    );

    if b.hotspot_status.is_hotspot {
        println!(
            "│  🔥 Hotspot:   {}{:width$}│",
            b.hotspot_status.role,
            "",
            width = width.saturating_sub(18 + b.hotspot_status.role.len())
        );
    }

    println!("│{:width$}│", "", width = width);

    if !b.blast_radius.upstream.is_empty() {
        println!(
            "│  ── Upstream (depends on this) ──{:width$}│",
            "",
            width = width - 36
        );
        for node in b.blast_radius.upstream.iter().take(5) {
            println!(
                "│  • {} (depth {}) — {}{:width$}│",
                node.id,
                node.depth,
                truncate(&node.relationship, 20),
                "",
                width =
                    width.saturating_sub(10 + node.id.len() + 10 + node.relationship.len().min(20))
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    if !b.blast_radius.downstream.is_empty() {
        println!(
            "│  ── Downstream (this depends on) ──{:width$}│",
            "",
            width = width - 38
        );
        for node in b.blast_radius.downstream.iter().take(5) {
            println!(
                "│  • {} (depth {}) — {}{:width$}│",
                node.id,
                node.depth,
                truncate(&node.relationship, 20),
                "",
                width =
                    width.saturating_sub(10 + node.id.len() + 10 + node.relationship.len().min(20))
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    if !b.decisions.is_empty() {
        println!(
            "│  ── Active Decisions ──{:width$}│",
            "",
            width = width - 26
        );
        for d in &b.decisions {
            println!(
                "│  {}: {}{:width$}│",
                d.id,
                truncate(&d.title, 40),
                "",
                width = width.saturating_sub(6 + d.id.len() + d.title.len().min(40))
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    if !b.active_drift_violations.is_empty() {
        println!(
            "│  ── Active drift (target-scoped) ──{:width$}│",
            "",
            width = width.saturating_sub(36)
        );
        for v in b.active_drift_violations.iter().take(5) {
            let loc = v
                .location
                .as_deref()
                .map(|l| format!(" @ {l}"))
                .unwrap_or_default();
            let display = truncate(&format!("{}{}", v.message, loc), width - 8);
            println!(
                "│  ⚠  {}{:width$}│",
                display,
                "",
                width = width.saturating_sub(6 + display.len())
            );
        }
        if b.active_drift_violations.len() > 5 {
            println!(
                "│  … +{} more violation(s){:width$}│",
                b.active_drift_violations.len() - 5,
                "",
                width = width
                    .saturating_sub(28 + format!("{}", b.active_drift_violations.len() - 5).len())
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    let not_allowed: Vec<&BoundaryInfo> = b.boundaries.iter().filter(|b| !b.allowed).collect();
    if !not_allowed.is_empty() {
        println!("│  ── Boundaries ──{:width$}│", "", width = width - 20);
        for bi in &not_allowed {
            println!(
                "│  ⛔ {} → {}: NOT allowed{:width$}│",
                bi.from,
                bi.to,
                "",
                width = width.saturating_sub(10 + bi.from.len() + bi.to.len() + 15)
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    if let Some(ref t) = b.temporal {
        println!(
            "│  ── Git temporal ({}..{}) ──{:width$}│",
            t.base_ref,
            t.head_ref,
            "",
            width = width.saturating_sub(21 + t.base_ref.len() + t.head_ref.len())
        );
        println!(
            "│  Diff-mapped components: {}{:width$}│",
            t.diff_mapped_component_ids.len(),
            "",
            width =
                width.saturating_sub(29 + format!("{}", t.diff_mapped_component_ids.len()).len())
        );
        if t.touches_focus_target {
            println!(
                "│  Target overlaps diff map: yes{:width$}│",
                "",
                width = width - 30
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    if !b.external_context.is_empty() {
        println!(
            "│  ── External Context ──{:width$}│",
            "",
            width = width - 26
        );
        for ec in &b.external_context {
            println!(
                "│  📄 {} [{}]{:width$}│",
                ec.file,
                ec.category,
                "",
                width = width.saturating_sub(8 + ec.file.len() + ec.category.len())
            );
        }
        println!("│{:width$}│", "", width = width);
    }

    println!(
        "│  ── AI Agent Instructions ──{:width$}│",
        "",
        width = width - 31
    );
    for (i, instr) in b.ai_instructions.iter().enumerate() {
        let display = truncate(instr, width - 8);
        println!(
            "│  {}. {}{:width$}│",
            i + 1,
            display,
            "",
            width = width.saturating_sub(6 + display.len())
        );
    }

    if !b.anti_patterns.is_empty() {
        println!("│{:width$}│", "", width = width);
        println!(
            "│  ── What NOT to try (Scratchpad) ──{:width$}│",
            "",
            width = width - 37
        );
        for ap in b.anti_patterns.iter().take(5) {
            let display = truncate(ap, width - 8);
            println!(
                "│  ⛔ {}{:width$}│",
                display,
                "",
                width = width.saturating_sub(6 + display.len())
            );
        }
    }

    println!("│{:width$}│", "", width = width);
    println!(
        "│  Context Score: {}{:width$}│",
        colors::health_bar(b.context_score, 15),
        "",
        width = width.saturating_sub(45)
    );

    if let Some(plan) = &b.ask_plan {
        let (tag, tag_color) = match plan.verdict {
            Verdict::Ask => ("ASK", colored::Color::Red),
            Verdict::ProceedAndFlag => ("PROCEED*", colored::Color::Yellow),
            Verdict::ProceedCitingPrecedent => ("PROCEED (precedent)", colored::Color::Green),
            Verdict::ProceedSilent => ("PROCEED", colored::Color::Green),
        };
        let door = match plan.reversibility {
            Reversibility::OneWay => "one-way",
            Reversibility::TwoWay => "two-way",
        };
        let blast_s = plan.blast_radius.to_string();
        let conf_s = match plan.confidence {
            Some(c) => c.to_string(),
            None => "?".to_string(),
        };
        println!("│{:width$}│", "", width = width);
        let fixed = 40;
        let used = fixed + tag.len() + door.len() + blast_s.len() + conf_s.len();
        println!(
            "│  Ask/Proceed: {}  [{} door, blast {}, conf {}]{:width$}│",
            tag.color(tag_color),
            door,
            blast_s,
            conf_s,
            "",
            width = width.saturating_sub(used)
        );
        let reason = truncate(&plan.reason, width.saturating_sub(6));
        println!(
            "│    {}{:width$}│",
            reason,
            "",
            width = width.saturating_sub(6 + reason.len())
        );
    }

    println!("╰{}╯", border);
    println!();
}
