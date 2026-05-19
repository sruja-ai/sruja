//! AST structures for Sruja DSL
//!
//! Core element and program types live in [`core`]. SLO, loops, fitness, and
//! incident types live in [`extended`].

mod core;
mod extended;

#[cfg(test)]
mod tests;

pub use core::*;
pub use extended::*;
