//! Language Server Protocol (LSP) implementation for Sruja DSL.
//!
//! This crate provides IDE support for Sruja architecture files, including:
//! - Diagnostics (errors, warnings)
//! - Go-to-definition
//! - Auto-completion
//! - Hover information
//!
//! # Example
//!
//! ```no_run
//! use sruja_lsp::SrujaLanguageServer;
//! // Run the LSP server on stdio
//! // sruja_lsp::server::run_stdio().await
//! ```

pub mod diagnostics;
pub mod features;
pub mod server;
pub mod workspace;

pub use server::SrujaLanguageServer;
