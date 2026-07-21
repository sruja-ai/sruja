pub mod types;
pub mod briefing;
pub mod format;

pub use types::{FocusBriefing, MemoryHit};
pub use briefing::{
    build_focus_briefing, build_focus_for_ai_output, compute_ask_plan, focus,
    load_ask_thresholds, resolve_target,
};

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_agent::{Reversibility, Thresholds, Verdict};
    use std::fs;

    fn hit(kind: Option<&str>) -> MemoryHit {
        MemoryHit {
            id: "x".into(),
            kind: Some("learning".into()),
            hitl_kind: kind.map(str::to_string),
            outcome: "success".into(),
            match_reason: "test".into(),
            timestamp: "now".into(),
            hypothesis: "h".into(),
            guardrail_advice: "g".into(),
        }
    }

    #[test]
    fn compute_ask_plan_asks_on_one_way_door() {
        let plan = compute_ask_plan(
            "Database",
            "Orders DB",
            0,
            Some(100),
            &[],
            &Thresholds::default(),
        );
        assert_eq!(plan.verdict, Verdict::Ask);
        assert_eq!(plan.reversibility, Reversibility::OneWay);
    }

    #[test]
    fn compute_ask_plan_proceeds_silent_on_simple_two_way_target() {
        let plan = compute_ask_plan(
            "container",
            "Web Server",
            1,
            Some(90),
            &[],
            &Thresholds::default(),
        );
        assert_eq!(plan.verdict, Verdict::ProceedSilent);
    }

    #[test]
    fn compute_ask_plan_unmeasured_confidence_proceeds_silent_on_two_way_low_blast() {
        let plan = compute_ask_plan(
            "component",
            "API handler",
            1,
            None,
            &[],
            &Thresholds::default(),
        );
        assert_eq!(plan.verdict, Verdict::ProceedSilent);
        assert_eq!(plan.confidence, None);
    }

    #[test]
    fn compute_ask_plan_unmeasured_confidence_still_asks_on_one_way_door() {
        let plan = compute_ask_plan(
            "Database",
            "Orders DB",
            0,
            None,
            &[],
            &Thresholds::default(),
        );
        assert_eq!(plan.verdict, Verdict::Ask);
    }

    #[test]
    fn compute_ask_plan_flags_at_mid_confidence() {
        let plan = compute_ask_plan(
            "component",
            "API handler",
            1,
            Some(60),
            &[],
            &Thresholds::default(),
        );
        assert_eq!(plan.verdict, Verdict::ProceedAndFlag);
    }

    #[test]
    fn compute_ask_plan_asks_on_high_blast_radius() {
        let plan = compute_ask_plan(
            "component",
            "API handler",
            50,
            Some(95),
            &[],
            &Thresholds::default(),
        );
        assert_eq!(plan.verdict, Verdict::Ask);
    }

    #[test]
    fn compute_ask_plan_cites_precedent_from_memory_hit() {
        let plan = compute_ask_plan(
            "Database",
            "Orders DB",
            50,
            Some(10),
            &[hit(Some("precedent"))],
            &Thresholds::default(),
        );
        assert_eq!(plan.verdict, Verdict::ProceedCitingPrecedent);
    }

    #[test]
    fn compute_ask_plan_ignores_non_precedent_memory_hits() {
        let plan = compute_ask_plan(
            "Database",
            "Orders DB",
            0,
            Some(100),
            &[hit(Some("correction")), hit(Some("guardrail"))],
            &Thresholds::default(),
        );
        assert_eq!(plan.verdict, Verdict::Ask);
    }

    #[test]
    fn load_ask_thresholds_falls_back_to_defaults_without_config() {
        let dir = tempfile::tempdir().unwrap();
        let t = load_ask_thresholds(dir.path());
        assert_eq!(t, Thresholds::default());
    }

    #[test]
    fn load_ask_thresholds_reads_config_overrides() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_dir = dir.path().join(".sruja");
        fs::create_dir_all(&cfg_dir).unwrap();
        fs::write(
            cfg_dir.join("config.toml"),
            "[ask]\nblast_ask = 2\nconfidence_floor = 80\n",
        )
        .unwrap();

        let t = load_ask_thresholds(dir.path());
        assert_eq!(t.blast_ask, 2);
        assert_eq!(t.confidence_floor, 80);
        assert_eq!(t.confidence_flag, Thresholds::default().confidence_flag);
        assert_eq!(t.trust_default, Thresholds::default().trust_default);

        let plan = compute_ask_plan("component", "API", 3, Some(85), &[], &t);
        assert_eq!(plan.verdict, Verdict::Ask);
    }
}
