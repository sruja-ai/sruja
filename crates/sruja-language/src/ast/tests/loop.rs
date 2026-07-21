use super::*;

#[test]
fn test_feedback_loop_type_to_symbol_reinforcing() {
    let loop_type = FeedbackLoopType::Reinforcing;
    assert_eq!(loop_type.to_symbol(), "+");
}

#[test]
fn test_feedback_loop_type_to_symbol_balancing() {
    let loop_type = FeedbackLoopType::Balancing;
    assert_eq!(loop_type.to_symbol(), "-");
}

#[test]
fn test_feedback_loop_type_display_reinforcing() {
    let loop_type = FeedbackLoopType::Reinforcing;
    assert_eq!(format!("{}", loop_type), "reinforcing");
}

#[test]
fn test_feedback_loop_type_display_balancing() {
    let loop_type = FeedbackLoopType::Balancing;
    assert_eq!(format!("{}", loop_type), "balancing");
}

#[test]
fn test_causal_polarity_to_symbol_positive() {
    let polarity = CausalPolarity::Positive;
    assert_eq!(polarity.to_symbol(), "+");
}

#[test]
fn test_causal_polarity_to_symbol_negative() {
    let polarity = CausalPolarity::Negative;
    assert_eq!(polarity.to_symbol(), "-");
}

#[test]
fn test_causal_polarity_display_positive() {
    let polarity = CausalPolarity::Positive;
    assert_eq!(format!("{}", polarity), "+");
}

#[test]
fn test_causal_polarity_display_negative() {
    let polarity = CausalPolarity::Negative;
    assert_eq!(format!("{}", polarity), "-");
}

#[test]
fn test_feedback_loop_creation() {
    let feedback_loop = FeedbackLoop {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "loop1".to_string(),
        loop_type: FeedbackLoopType::Reinforcing,
        loop_id: Some("growth".to_string()),
        title: "Growth Loop".to_string(),
        description: Some("Positive feedback loop".to_string()),
        relationships: Vec::new(),
    };

    assert_eq!(feedback_loop.id, "loop1");
    assert_eq!(feedback_loop.loop_type, FeedbackLoopType::Reinforcing);
    assert_eq!(feedback_loop.title, "Growth Loop");
}

#[test]
fn test_causal_loop_creation() {
    let causal_loop = CausalLoop {
        location: SourceLocation::new("test.sruja".to_string(), 1, 1),
        id: "cl1".to_string(),
        loop_type: FeedbackLoopType::Balancing,
        loop_id: Some("stability".to_string()),
        title: "Stability Loop".to_string(),
        description: Some("Balancing loop".to_string()),
        variables: vec![CausalLoopVariable {
            id: "var1".to_string(),
            label: Some("Variable 1".to_string()),
        }],
        relationships: vec![CausalRelationship {
            from: "var1".to_string(),
            to: "var2".to_string(),
            effect: Some("increases".to_string()),
            polarity: CausalPolarity::Positive,
            delay: Some("1h".to_string()),
        }],
    };

    assert_eq!(causal_loop.id, "cl1");
    assert_eq!(causal_loop.variables.len(), 1);
    assert_eq!(causal_loop.relationships.len(), 1);
}
