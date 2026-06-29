// ---------------------------------------------------------------------------
// loop_event tests (U1) - Basic module compilation test
// ---------------------------------------------------------------------------

#[test]
fn loop_event_module_compiles() {
    // Test that LoopEvent, LoopPhase, and PlanBrief are accessible
    use crate::cognition::LoopEvent;
    use crate::cognition::LoopPhase;
    use crate::cognition::PlanBrief;

    // Test LoopPhase
    let phase = LoopPhase::Execute;
    assert_eq!(phase, LoopPhase::Execute);

    // Test LoopEvent::Started
    let event = LoopEvent::Started {
        goal: "test".into(),
        max_iterations: 10,
    };
    if let LoopEvent::Started {
        goal,
        max_iterations,
    } = event
    {
        assert_eq!(goal, "test");
        assert_eq!(max_iterations, 10);
    } else {
        panic!("Wrong variant");
    }

    // Test LoopEvent::Done
    let event = LoopEvent::Done {
        outcome_summary: "Completed".into(),
    };
    if let LoopEvent::Done { outcome_summary } = event {
        assert_eq!(outcome_summary, "Completed");
    } else {
        panic!("Wrong variant");
    }

    // Test PlanBrief
    let brief = PlanBrief {
        goal: "test goal".into(),
        criteria: vec![],
        subtasks: vec![],
    };
    assert_eq!(brief.goal, "test goal");
}
