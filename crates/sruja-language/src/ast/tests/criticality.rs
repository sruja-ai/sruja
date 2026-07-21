use super::*;

#[test]
fn test_criticality_parsing_and_display() {
    assert_eq!(Criticality::from_str("low").unwrap(), Criticality::Low);
    assert_eq!(
        Criticality::from_str("medium").unwrap(),
        Criticality::Medium
    );
    assert_eq!(Criticality::from_str("med").unwrap(), Criticality::Medium);
    assert_eq!(Criticality::from_str("high").unwrap(), Criticality::High);
    assert_eq!(
        Criticality::from_str("critical").unwrap(),
        Criticality::Critical
    );
    assert!(Criticality::from_str("invalid_criticality").is_err());

    assert_eq!(Criticality::Low.as_str(), "low");
    assert_eq!(format!("{}", Criticality::High), "high");
}

#[test]
fn test_criticality_display() {
    assert_eq!(format!("{}", Criticality::Low), "low");
    assert_eq!(format!("{}", Criticality::Medium), "medium");
    assert_eq!(format!("{}", Criticality::High), "high");
    assert_eq!(format!("{}", Criticality::Critical), "critical");
}

#[test]
fn test_criticality_as_str() {
    assert_eq!(Criticality::Low.as_str(), "low");
    assert_eq!(Criticality::Medium.as_str(), "medium");
    assert_eq!(Criticality::High.as_str(), "high");
    assert_eq!(Criticality::Critical.as_str(), "critical");
}

#[test]
fn test_criticality_from_str_case_insensitive() {
    assert_eq!(Criticality::from_str("LOW").unwrap(), Criticality::Low);
    assert_eq!(Criticality::from_str("High").unwrap(), Criticality::High);
    assert_eq!(
        Criticality::from_str("CRITICAL").unwrap(),
        Criticality::Critical
    );
}

#[test]
fn test_criticality_from_str_invalid() {
    assert!(Criticality::from_str("invalid").is_err());
    assert!(Criticality::from_str("").is_err());
    assert!(Criticality::from_str("medium_high").is_err());
}
