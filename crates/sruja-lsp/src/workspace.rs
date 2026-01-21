//! Workspace management for LSP server
//!
//! This module manages open documents and their parsed state.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use sruja_language::{Program, Parser};
use tower_lsp::lsp_types::*;

/// Document in the workspace
pub struct Document {
    pub uri: Url,
    pub version: i32,
    pub text: String,
    pub program: Option<Program>,
}

impl Document {
    pub fn new(uri: Url, text: String, version: i32) -> Self {
        Self {
            uri,
            version,
            text,
            program: None,
        }
    }

    pub fn apply_change(&mut self, change: TextDocumentContentChangeEvent) {
        match change.range {
            Some(range) => {
                // Incremental update
                let start = self.text.lines().take(range.start.line as usize).collect::<Vec<_>>().join("\n").len() + range.start.character as usize;
                let end = self.text.lines().take(range.end.line as usize).collect::<Vec<_>>().join("\n").len() + range.end.character as usize;
                
                self.text.replace_range(start..end, &change.text);
            }
            None => {
                // Full document update
                self.text = change.text;
            }
        }
    }

    pub fn parse(&mut self) {
        let mut parser = Parser::new();
        match parser.parse(&self.text) {
            Ok(program) => {
                self.program = Some(program);
            }
            Err(_) => {
                self.program = None;
            }
        }
    }
}

/// Workspace managing all open documents
pub struct Workspace {
    documents: Arc<RwLock<HashMap<Url, Document>>>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_document(&self, uri: Url, text: String, version: i32) {
        let mut doc = Document::new(uri.clone(), text, version);
        doc.parse();
        self.documents.write().await.insert(uri, doc);
    }

    pub async fn update_document(&self, uri: &Url, version: i32, changes: Vec<TextDocumentContentChangeEvent>) {
        let mut docs = self.documents.write().await;
        if let Some(doc) = docs.get_mut(uri) {
            doc.version = version;
            for change in changes {
                doc.apply_change(change);
            }
            doc.parse();
        }
    }

    pub async fn remove_document(&self, uri: &Url) {
        self.documents.write().await.remove(uri);
    }

    pub async fn get_document(&self, uri: &Url) -> Option<Arc<Document>> {
        self.documents.read().await.get(uri).map(|d| Arc::new(d.clone()))
    }

    pub async fn get_program(&self, uri: &Url) -> Option<Program> {
        self.documents.read().await.get(uri)?.program.clone()
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Document {
    fn clone(&self) -> Self {
        Self {
            uri: self.uri.clone(),
            version: self.version,
            text: self.text.clone(),
            program: self.program.clone(),
        }
    }
}
