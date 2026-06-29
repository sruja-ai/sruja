//! @element Sruja.Agent.Calibration
//! @layer Core Engine
//! @boundary The ask/proceed decision is a pure deterministic function owned
//!           by the grader/governance layer. The actor may not override it; the
//!           harness (focus, agent loop) invokes it and records its verdict.
//!
//! Ask-vs-proceed calibration. Decides whether an agent should interrupt a
//! human before acting, from reversibility, blast radius, confidence, trust
//! level, and precedent. Pure and side-effect free.

use serde::{Deserialize, Serialize};

use crate::cognition::{DecisionRecord, DecisionStatus};

/// Whether a change can be undone cheaply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    OneWay,
    TwoWay,
}

/// The calibration verdict. `Ask` interrupts the human; the three `Proceed*`
/// variants let the agent act, differing only in how loudly they record it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Ask,
    ProceedSilent,
    ProceedAndFlag,
    ProceedCitingPrecedent,
}

impl Verdict {
    pub fn should_ask(self) -> bool {
        matches!(self, Verdict::Ask)
    }

    pub fn proceeds(self) -> bool {
        !matches!(self, Verdict::Ask)
    }
}

/// A human-readable, machine-parseable decision rendered into briefings.
#[derive(Debug, Clone, Serialize)]
pub struct AskPlan {
    pub verdict: Verdict,
    pub reason: String,
    pub reversibility: Reversibility,
    pub blast_radius: u16,
    pub confidence: Option<u8>,
    pub trust_level: Option<u8>,
    pub has_precedent: bool,
    pub policy_says_ask: bool,
}

/// Surface hints used to infer [`Reversibility`] without a per-element tag.
#[derive(Debug, Clone, Copy)]
pub struct TargetHints<'a> {
    pub kind: &'a str,
    pub label: &'a str,
}

/// Inputs to [`decide`]. All fields are plain values; no I/O.
#[derive(Debug, Clone, Copy)]
pub struct AskInput {
    pub reversibility: Reversibility,
    pub blast_radius: u16,
    /// `Some(0..=100)` (saturated) is a measured confidence; `None` means
    /// unmeasured — no confidence-based escalation or flagging is applied.
    pub confidence: Option<u8>,
    /// `Some(0..=100)` (saturated) trust level; `None` means unmeasured.
    pub trust_level: Option<u8>,
    pub has_precedent: bool,
    pub policy_says_ask: bool,
}

/// Tunable thresholds. Defaults are conservative; override via config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Thresholds {
    pub blast_ask: u16,
    pub confidence_floor: u8,
    pub confidence_flag: u8,
    pub trust_default: u8,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            blast_ask: 8,
            confidence_floor: 45,
            confidence_flag: 70,
            trust_default: 50,
        }
    }
}

/// Single-word infrastructure keywords that indicate a one-way door.
const ONE_WAY_KEYWORDS: &[&str] = &[
    "database",
    "datastore",
    "migration",
    "schema",
    "deploy",
    "queue",
    "event store",
];

/// Multi-word patterns that are more specific than single keywords.
/// These catch "deploy to prod" but not "production code".
const ONE_WAY_PATTERNS: &[&str] = &[
    "prod database",
    "prod data",
    "production database",
    "production data",
    "deploy to prod",
    "deploy prod",
    "production deploy",
    "delete from",
    "drop table",
    "drop database",
    "publish to",
    "publish package",
    "publish release",
    "migrate data",
    "data migration",
];

/// Infer reversibility from target kind/label text. Conservative: any
/// storage/migration/deploy marker is treated as a one-way door.
///
/// Uses both single-word keywords and multi-word patterns to avoid
/// false positives (e.g. "production code" is two-way, but "deploy to
/// prod" is one-way).
pub fn infer_reversibility(hints: TargetHints<'_>) -> Reversibility {
    let haystack = format!("{} {}", hints.kind, hints.label).to_lowercase();
    if ONE_WAY_KEYWORDS.iter().any(|kw| haystack.contains(kw))
        || ONE_WAY_PATTERNS.iter().any(|pat| haystack.contains(pat))
    {
        Reversibility::OneWay
    } else {
        Reversibility::TwoWay
    }
}

/// The calibration function. Precedent is earned autonomy: it overrides
/// every ask condition (including one-way doors). One-way doors and explicit
/// policy asks are the only conditions that force `Ask`; blast radius and low
/// confidence escalate to `Ask` only in the absence of precedent.
pub fn decide(input: &AskInput, thresholds: &Thresholds) -> AskPlan {
    let confidence = input.confidence.map(|c| c.min(100));
    let trust_level = input.trust_level.map(|t| t.min(100));

    let asks_without_precedent = input.reversibility == Reversibility::OneWay
        || input.policy_says_ask
        || input.blast_radius >= thresholds.blast_ask
        || confidence.is_some_and(|c| c < thresholds.confidence_floor);

    let verdict = if !input.has_precedent && asks_without_precedent {
        Verdict::Ask
    } else if input.has_precedent {
        Verdict::ProceedCitingPrecedent
    } else if confidence.is_some_and(|c| c < thresholds.confidence_flag) {
        Verdict::ProceedAndFlag
    } else {
        Verdict::ProceedSilent
    };

    let conf_text = |c: Option<u8>| match c {
        Some(v) => v.to_string(),
        None => "unmeasured".to_string(),
    };

    let reason = match verdict {
        Verdict::Ask if input.reversibility == Reversibility::OneWay => {
            "One-way door without precedent; human approval required.".to_string()
        }
        Verdict::Ask if input.policy_says_ask => {
            "Policy requires human approval for this change.".to_string()
        }
        Verdict::Ask if input.blast_radius >= thresholds.blast_ask => format!(
            "Blast radius {} >= threshold {}; human approval required.",
            input.blast_radius, thresholds.blast_ask
        ),
        Verdict::Ask => format!(
            "Confidence {} < floor {}; human input needed to resolve uncertainty.",
            conf_text(confidence),
            thresholds.confidence_floor
        ),
        Verdict::ProceedCitingPrecedent => {
            "Precedent exists for this decision; proceeding with recorded autonomy.".to_string()
        }
        Verdict::ProceedAndFlag => format!(
            "Confidence {} below flag threshold {}; proceeding but flagging for review.",
            conf_text(confidence),
            thresholds.confidence_flag
        ),
        Verdict::ProceedSilent => "Two-way door, bounded blast radius; proceeding.".to_string(),
    };

    AskPlan {
        verdict,
        reason,
        reversibility: input.reversibility,
        blast_radius: input.blast_radius,
        confidence,
        trust_level,
        has_precedent: input.has_precedent,
        policy_says_ask: input.policy_says_ask,
    }
}

/// Construct a [`DecisionRecord`] from a `Proceed*` verdict.
///
/// Returns `None` for `Ask` (no DR for the halt case — the halt message IS
/// the record) and for `ProceedSilent` (matches "silent" semantics).
///
/// The DR captures *why the agent decided to act without asking* — a
/// deterministic, pre-execution artifact distinct from the LLM-generated DR
/// that explains *why the change was made*.
pub fn proceed_decision_record(plan: &AskPlan, goal: &str) -> Option<DecisionRecord> {
    match plan.verdict {
        Verdict::Ask | Verdict::ProceedSilent => None,
        Verdict::ProceedAndFlag | Verdict::ProceedCitingPrecedent => {
            let title = format!("Calibration: proceeded — {}", truncate_str(goal, 80));
            let context = format!(
                "Reversibility: {:?}, blast_radius: {}, confidence: {}, precedent: {}, policy: {}",
                plan.reversibility,
                plan.blast_radius,
                plan.confidence
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "unmeasured".into()),
                plan.has_precedent,
                plan.policy_says_ask,
            );
            let decision = format!("Verdict: {:?} — {}", plan.verdict, plan.reason,);
            let consequences: Vec<String> = vec![
                format!("Risk profile: {:?}", plan.verdict),
                "Agent proceeded without human interruption.".into(),
            ];
            let alternatives: Vec<String> =
                vec!["Ask human first — not required by calibration thresholds.".into()];

            Some(
                DecisionRecord::new(title, context, decision)
                    .with_status(DecisionStatus::Accepted)
                    .with_consequence(consequences[0].clone())
                    .with_consequence(consequences[1].clone())
                    .with_alternative(alternatives[0].clone()),
            )
        }
    }
}

/// Truncate a string to `max_len` characters, adding `…` if truncated.
/// Uses `chars()` to avoid panicking on non-ASCII boundaries.
fn truncate_str(s: &str, max_len: usize) -> String {
    let mut chars = s.chars();
    match chars.nth(max_len) {
        None => s.to_string(),
        Some(_) => {
            let prefix: String = s.chars().take(max_len).collect();
            format!("{prefix}…")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rank(v: Verdict) -> i32 {
        match v {
            Verdict::Ask => 0,
            Verdict::ProceedAndFlag => 1,
            Verdict::ProceedCitingPrecedent | Verdict::ProceedSilent => 2,
        }
    }

    fn two_way() -> AskInput {
        AskInput {
            reversibility: Reversibility::TwoWay,
            blast_radius: 1,
            confidence: Some(90),
            trust_level: Some(90),
            has_precedent: false,
            policy_says_ask: false,
        }
    }

    fn t() -> Thresholds {
        Thresholds::default()
    }

    #[test]
    fn two_way_low_blast_high_confidence_proceeds_silent() {
        let p = decide(&two_way(), &t());
        assert_eq!(p.verdict, Verdict::ProceedSilent);
        assert!(p.verdict.proceeds());
        assert!(!p.verdict.should_ask());
    }

    #[test]
    fn mid_confidence_proceeds_with_flag() {
        let mut i = two_way();
        i.confidence = Some(60);
        assert_eq!(decide(&i, &t()).verdict, Verdict::ProceedAndFlag);
    }

    #[test]
    fn low_confidence_asks() {
        let mut i = two_way();
        i.confidence = Some(10);
        assert_eq!(decide(&i, &t()).verdict, Verdict::Ask);
    }

    #[test]
    fn unmeasured_confidence_proceeds_silent_on_two_way_low_blast() {
        let mut i = two_way();
        i.confidence = None;
        i.trust_level = None;
        let p = decide(&i, &t());
        assert_eq!(p.verdict, Verdict::ProceedSilent);
        assert_eq!(p.confidence, None);
    }

    #[test]
    fn unmeasured_confidence_still_asks_on_one_way_door() {
        let mut i = two_way();
        i.reversibility = Reversibility::OneWay;
        i.confidence = None;
        assert_eq!(decide(&i, &t()).verdict, Verdict::Ask);
    }

    #[test]
    fn unmeasured_confidence_still_asks_on_high_blast() {
        let mut i = two_way();
        i.blast_radius = 50;
        i.confidence = None;
        assert_eq!(decide(&i, &t()).verdict, Verdict::Ask);
    }

    #[test]
    fn high_blast_asks_without_precedent() {
        let mut i = two_way();
        i.blast_radius = 50;
        assert_eq!(decide(&i, &t()).verdict, Verdict::Ask);
    }

    #[test]
    fn policy_says_ask_asks_without_precedent() {
        let mut i = two_way();
        i.policy_says_ask = true;
        assert_eq!(decide(&i, &t()).verdict, Verdict::Ask);
    }

    #[test]
    fn one_way_always_asks_without_precedent_even_at_full_confidence() {
        let mut i = two_way();
        i.reversibility = Reversibility::OneWay;
        i.confidence = Some(100);
        i.trust_level = Some(100);
        i.blast_radius = 0;
        let p = decide(&i, &t());
        assert_eq!(p.verdict, Verdict::Ask);
        assert!(p.reason.contains("One-way door"));
    }

    #[test]
    fn precedent_overrides_one_way_door() {
        let mut i = two_way();
        i.reversibility = Reversibility::OneWay;
        i.has_precedent = true;
        assert_eq!(decide(&i, &t()).verdict, Verdict::ProceedCitingPrecedent);
    }

    #[test]
    fn precedent_overrides_high_blast() {
        let mut i = two_way();
        i.blast_radius = 50;
        i.has_precedent = true;
        assert_eq!(decide(&i, &t()).verdict, Verdict::ProceedCitingPrecedent);
    }

    #[test]
    fn precedent_overrides_policy_ask() {
        let mut i = two_way();
        i.policy_says_ask = true;
        i.has_precedent = true;
        assert_eq!(decide(&i, &t()).verdict, Verdict::ProceedCitingPrecedent);
    }

    #[test]
    fn precedent_overrides_low_confidence() {
        let mut i = two_way();
        i.confidence = Some(0);
        i.has_precedent = true;
        assert_eq!(decide(&i, &t()).verdict, Verdict::ProceedCitingPrecedent);
    }

    #[test]
    fn confidence_and_trust_saturate_at_100() {
        let mut i = two_way();
        i.confidence = Some(200);
        i.trust_level = Some(250);
        let p = decide(&i, &t());
        assert_eq!(p.confidence, Some(100));
        assert_eq!(p.trust_level, Some(100));
        assert_eq!(p.verdict, Verdict::ProceedSilent);
    }

    #[test]
    fn blast_radius_at_exact_threshold_asks() {
        let mut i = two_way();
        i.blast_radius = t().blast_ask;
        assert_eq!(decide(&i, &t()).verdict, Verdict::Ask);
    }

    #[test]
    fn reason_is_populated_for_every_verdict() {
        let cases = [
            (Verdict::Ask, {
                let mut i = two_way();
                i.reversibility = Reversibility::OneWay;
                i
            }),
            (Verdict::ProceedSilent, two_way()),
            (Verdict::ProceedAndFlag, {
                let mut i = two_way();
                i.confidence = Some(60);
                i
            }),
            (Verdict::ProceedCitingPrecedent, {
                let mut i = two_way();
                i.has_precedent = true;
                i
            }),
        ];
        for (expected, input) in cases {
            let p = decide(&input, &t());
            assert_eq!(p.verdict, expected, "verdict mismatch");
            assert!(!p.reason.is_empty(), "empty reason for {expected:?}");
        }
    }

    #[test]
    fn confidence_monotonic_non_decreasing_in_proceediness() {
        let base = two_way();
        let mut prev_rank = i32::MIN;
        for c in 0u8..=100 {
            let mut i = base;
            i.confidence = Some(c);
            let rank = rank(decide(&i, &t()).verdict);
            assert!(
                rank >= prev_rank,
                "confidence {c}: rank {rank} < prev {prev_rank} (proceed-ness dropped as confidence rose)"
            );
            prev_rank = rank;
        }
    }

    #[test]
    fn blast_radius_monotonic_non_increasing_in_proceediness() {
        let base = two_way();
        let mut prev_rank = i32::MAX;
        for br in 0u16..=60 {
            let mut i = base;
            i.blast_radius = br;
            let rank = rank(decide(&i, &t()).verdict);
            assert!(
                rank <= prev_rank,
                "blast {br}: rank {rank} > prev {prev_rank} (proceed-ness rose as blast radius rose)"
            );
            prev_rank = rank;
        }
    }

    #[test]
    fn one_way_door_forces_ask_across_all_confidence_and_trust() {
        for c in 0u8..=100 {
            for trust in (0..=100u8).step_by(25) {
                let i = AskInput {
                    reversibility: Reversibility::OneWay,
                    blast_radius: 0,
                    confidence: Some(c),
                    trust_level: Some(trust),
                    has_precedent: false,
                    policy_says_ask: false,
                };
                assert_eq!(
                    decide(&i, &t()).verdict,
                    Verdict::Ask,
                    "one-way must ask at confidence {c}, trust {trust}"
                );
            }
        }
    }

    #[test]
    fn infer_reversibility_flags_storage_and_migration() {
        for (kind, label, expected) in [
            ("Database", "Orders DB", Reversibility::OneWay),
            ("Container", "Migration Runner", Reversibility::OneWay),
            ("Queue", "Event Bus", Reversibility::OneWay),
            ("component", "drop table users", Reversibility::OneWay),
            ("container", "deploy to prod", Reversibility::OneWay),
            (
                "Goal",
                "fix unwrap calls in production code",
                Reversibility::TwoWay,
            ),
            ("Goal", "refactor the delete handler", Reversibility::TwoWay),
            (
                "Goal",
                "publish a blog post about the API",
                Reversibility::TwoWay,
            ),
            ("component", "API", Reversibility::TwoWay),
            ("container", "Web Server", Reversibility::TwoWay),
        ] {
            let got = infer_reversibility(TargetHints { kind, label });
            assert_eq!(got, expected, "{kind}/{label}");
        }
    }

    // --- proceed_decision_record tests ---

    fn flag_plan() -> AskPlan {
        let mut i = two_way();
        i.confidence = Some(60);
        decide(&i, &t())
    }

    fn precedent_plan() -> AskPlan {
        let mut i = two_way();
        i.has_precedent = true;
        decide(&i, &t())
    }

    fn ask_plan() -> AskPlan {
        let mut i = two_way();
        i.reversibility = Reversibility::OneWay;
        decide(&i, &t())
    }

    fn silent_plan() -> AskPlan {
        decide(&two_way(), &t())
    }

    #[test]
    fn proceed_decision_record_returns_some_for_flag() {
        let plan = flag_plan();
        assert_eq!(plan.verdict, Verdict::ProceedAndFlag);
        let dr = proceed_decision_record(&plan, "refactor handler").unwrap();
        assert_eq!(dr.status, DecisionStatus::Accepted);
    }

    #[test]
    fn proceed_decision_record_returns_some_for_precedent() {
        let plan = precedent_plan();
        assert_eq!(plan.verdict, Verdict::ProceedCitingPrecedent);
        let dr = proceed_decision_record(&plan, "add cache layer").unwrap();
        assert_eq!(dr.status, DecisionStatus::Accepted);
    }

    #[test]
    fn proceed_decision_record_returns_none_for_ask() {
        let plan = ask_plan();
        assert!(proceed_decision_record(&plan, "migrate db").is_none());
    }

    #[test]
    fn proceed_decision_record_returns_none_for_silent() {
        let plan = silent_plan();
        assert!(proceed_decision_record(&plan, "rename var").is_none());
    }

    #[test]
    fn proceed_decision_record_title_contains_goal() {
        let plan = flag_plan();
        let dr = proceed_decision_record(&plan, "add health check endpoint").unwrap();
        assert!(dr.title.contains("add health check endpoint"));
    }

    #[test]
    fn proceed_decision_record_context_contains_blast_and_reversibility() {
        let plan = flag_plan();
        let dr = proceed_decision_record(&plan, "test goal").unwrap();
        assert!(dr.context.contains("blast_radius"));
        assert!(dr.context.contains("Reversibility"));
    }

    #[test]
    fn proceed_decision_record_decision_contains_verdict_reason() {
        let plan = flag_plan();
        let dr = proceed_decision_record(&plan, "test goal").unwrap();
        assert!(dr.decision.contains("ProceedAndFlag"));
        assert!(!dr.decision.is_empty());
    }

    #[test]
    fn proceed_decision_record_markdown_renders() {
        let plan = precedent_plan();
        let dr = proceed_decision_record(&plan, "deploy to prod").unwrap();
        let md = dr.to_markdown();
        assert!(md.contains("# Decision: Calibration: proceeded"));
        assert!(md.contains("deploy to prod"));
        assert!(md.contains("Ask human first"));
    }

    #[test]
    fn truncate_str_handles_multibyte_chars() {
        // 3-byte CJK chars: 100 chars = 300 bytes. max_len=80 chars should
        // truncate at char boundary (byte 240) without panicking.
        let cjk: String = "字".repeat(100);
        let truncated = truncate_str(&cjk, 80);
        assert!(truncated.ends_with('…'));
        assert_eq!(truncate_str(&cjk, 100), cjk);
        // Also verify the prefix is correct length (80 chars + ellipsis)
        assert_eq!(truncated.chars().count(), 81);
    }

    #[test]
    fn proceed_decision_record_handles_non_ascii_goal() {
        let plan = precedent_plan();
        let long_goal = "データベースのスキーマを移行する".repeat(5);
        let dr = proceed_decision_record(&plan, &long_goal);
        assert!(dr.is_some());
    }
}
