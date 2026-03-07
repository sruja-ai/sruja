//! Intent Source Parsers
//!
//! Parsers for various intent sources: ADR files, .sruja files, design docs.

mod adr;

pub use adr::{
    AdrParser, AdrStatus, BoundaryChange, BoundaryChangeType, ParsedAdr, StructuralImplication,
};
