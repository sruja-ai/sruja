//! Sruja Language Server Protocol Implementation
//!
//! This crate provides LSP server functionality for the Sruja DSL,
//! including diagnostics, completion, hover, definition, references,
//! symbols, formatting, and code actions.

pub mod server;
pub mod workspace;
pub mod diagnostics;
pub mod features;

pub use server::SrujaLanguageServer;
