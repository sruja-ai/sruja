//! Intent Source Parsers
//!
//! Parsers for various intent sources: ADR files, .sruja files, design docs.

mod adr;

pub use adr::{
    AdrParser, AdrStatus, BoundaryChange, BoundaryChangeType, ParsedAdr, StructuralImplication,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_exports() {
        // Just verify the types are exported and can be instantiated/used
        let _parser = AdrParser::new();
        assert!(format!("{:?}", AdrStatus::Proposed).contains("Proposed"));
        let _ = BoundaryChange {
            component: "A".to_string(),
            change_type: BoundaryChangeType::Added,
            description: "Desc".to_string(),
        };
    }
}
