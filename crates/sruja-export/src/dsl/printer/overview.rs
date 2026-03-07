//! Overview block printing.

use sruja_language::OverviewBlock;

pub fn print_overview(out: &mut String, overview: &OverviewBlock) {
    out.push_str("overview {\n");
    if let Some(summary) = &overview.summary {
        out.push_str(&format!("    summary \"{}\"\n", summary));
    }
    if let Some(audience) = &overview.audience {
        out.push_str(&format!("    audience \"{}\"\n", audience));
    }
    if let Some(scope) = &overview.scope {
        out.push_str(&format!("    scope \"{}\"\n", scope));
    }
    if !overview.goals.is_empty() {
        let goals_str = overview
            .goals
            .iter()
            .map(|g| format!("\"{}\"", g))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("    goals [{}]\n", goals_str));
    }
    if !overview.non_goals.is_empty() {
        let non_goals_str = overview
            .non_goals
            .iter()
            .map(|g| format!("\"{}\"", g))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("    non_goals [{}]\n", non_goals_str));
    }
    if !overview.risks.is_empty() {
        let risks_str = overview
            .risks
            .iter()
            .map(|r| format!("\"{}\"", r))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("    risks [{}]\n", risks_str));
    }
    out.push_str("}\n");
}
