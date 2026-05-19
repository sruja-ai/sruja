//! AST structures for Sruja DSL
//!
//! Types are grouped by domain: [`kinds`], [`element`], [`relation`], [`spec`],
//! [`governance`], [`blocks`], and [`extended`] (SLO, loops, incidents).

mod blocks;
mod element;
mod extended;
mod governance;
mod kinds;
mod program;
mod relation;
mod spec;

#[cfg(test)]
mod tests;

pub use blocks::*;
pub use element::*;
pub use extended::*;
pub use governance::*;
pub use kinds::*;
pub use program::*;
pub use relation::*;
pub use spec::*;
