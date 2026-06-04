//! Relations, qualified identifiers, and imports.

use sruja_diagnostics::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relation {
    pub location: SourceLocation,
    pub from: QualifiedIdent,
    pub to: QualifiedIdent,
    pub label: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedIdent {
    pub parts: Vec<String>,
}

impl QualifiedIdent {
    pub fn simple(ident: String) -> Self {
        Self { parts: vec![ident] }
    }

    pub fn qualified(parts: Vec<String>) -> Self {
        Self { parts }
    }

    pub fn as_string(&self) -> String {
        self.parts.join(".")
    }
}

impl std::fmt::Display for QualifiedIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStatement {
    pub location: SourceLocation,
    pub elements: Vec<ImportElement>,
    pub from: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportElement {
    /// Import a specific named element (e.g., `import { MyBoundary } from "path"`)
    Ident(String),
    /// Import all elements (e.g., `import "path"` or `import { * } from "path"`)
    Wildcard,
    /// Import all boundaries from a file (e.g., `import { boundary } from "path"`)
    Boundary,
    /// Import all policies from a file (e.g., `import { policy } from "path"`)
    Policy,
}
