/**
 * Pure formatters for CLI JSON output (status, review). Testable without vscode or exec.
 */

export interface StatusJson {
  baseline?: string | null;
  truth_status?: string;
  violations_count?: number;
  health_score?: number;
  context_updated_at?: string;
}

export interface ReviewJson {
  truth_status: string;
  baseline?: string | null;
  has_drift: boolean;
  violations_count: number;
  health_score?: number;
  new_components: string[];
  missing_components: string[];
  drifted_dependencies: string[];
  open_questions: string[];
  suggestions: string[];
}

/** Format status JSON into display lines for the output channel. */
export function formatStatusLines(status: StatusJson): string[] {
  const base = status.baseline ?? "(none)";
  const truth = status.truth_status ?? "unknown";
  const violations = status.violations_count ?? 0;
  const score = status.health_score != null ? ` ${status.health_score}/100` : "";
  const ctxAt = status.context_updated_at ? ` | Context: ${status.context_updated_at}` : "";
  return [
    `Baseline: ${base}`,
    `Truth: ${truth} (${violations} violation(s))${score}${ctxAt}`,
    "--- Done ---",
  ];
}

/** Format review JSON into display lines for the output channel. */
export function formatReviewLines(review: ReviewJson): string[] {
  const base = review.baseline ?? "(none)";
  const truth = review.truth_status ?? "unknown";
  const violations = review.violations_count ?? 0;
  const score = review.health_score != null ? ` ${review.health_score}/100` : "";
  const lines: string[] = [
    `Baseline: ${base}`,
    `Truth: ${truth} (${violations} violation(s))${score}`,
    `Has drift: ${review.has_drift}`,
    "",
  ];
  const newComponents = Array.isArray(review.new_components) ? review.new_components : [];
  const missingComponents = Array.isArray(review.missing_components) ? review.missing_components : [];
  const driftedDeps = Array.isArray(review.drifted_dependencies) ? review.drifted_dependencies : [];
  const openQuestions = Array.isArray(review.open_questions) ? review.open_questions : [];
  const suggestions = Array.isArray(review.suggestions) ? review.suggestions : [];

  if (newComponents.length > 0) {
    lines.push("New components:");
    newComponents.forEach((c) => lines.push(`  + ${c}`));
    lines.push("");
  }
  if (missingComponents.length > 0) {
    lines.push("Missing components:");
    missingComponents.forEach((c) => lines.push(`  - ${c}`));
    lines.push("");
  }
  if (driftedDeps.length > 0) {
    lines.push("Drifted dependencies:");
    driftedDeps.forEach((d) => lines.push(`  ~ ${d}`));
    lines.push("");
  }
  if (openQuestions.length > 0) {
    lines.push("Open questions:");
    openQuestions.forEach((q) => lines.push(`  ? ${q}`));
    lines.push("");
  }
  if (suggestions.length > 0) {
    lines.push("Suggestions:");
    suggestions.forEach((s) => lines.push(`  > ${s}`));
    lines.push("");
  }
  lines.push("--- Done ---");
  return lines;
}
