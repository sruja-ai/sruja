//! Workspace management for LSP server
//!
//! This module manages open documents and their parsed state.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use sruja_language::{Parser, Program};
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

    /// Get the document URI
    pub fn uri(&self) -> &Url {
        &self.uri
    }

    /// Get the document version
    pub fn version(&self) -> i32 {
        self.version
    }

    /// Get the document text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the parsed program if available
    pub fn program(&self) -> Option<&Program> {
        self.program.as_ref()
    }

    pub fn apply_change(&mut self, change: TextDocumentContentChangeEvent) {
        match change.range {
            Some(range) => {
                // Incremental update - compute byte offsets
                let lines: Vec<&str> = self.text.lines().collect();
                let mut start_offset = 0;
                for i in 0..range.start.line.min(lines.len() as u32) {
                    start_offset += lines[i as usize].len() + 1; // +1 for newline
                }
                start_offset += range.start.character as usize;

                let mut end_offset = 0;
                for i in 0..range.end.line.min(lines.len() as u32) {
                    end_offset += lines[i as usize].len() + 1; // +1 for newline
                }
                end_offset += range.end.character as usize;

                // Apply change
                let start = start_offset.min(self.text.len());
                let end = end_offset.min(self.text.len());
                let mut new_text =
                    String::with_capacity(self.text.len() - (end - start) + change.text.len());
                new_text.push_str(&self.text[..start]);
                new_text.push_str(&change.text);
                new_text.push_str(&self.text[end..]);
                self.text = new_text;
            }
            None => {
                // Full document update
                self.text = change.text;
            }
        }
    }

    pub fn parse(&mut self) {
        let parser = Parser::new(self.uri.to_string());
        match parser.parse(&self.text) {
            Ok(program) => {
                self.program = Some(program);
            }
            Err(_) => {
                self.program = None;
            }
        }
    }

    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.text.lines().nth(line)
    }

    pub fn lines(&self) -> Vec<&str> {
        self.text.lines().collect()
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

    pub async fn update_document(
        &self,
        uri: &Url,
        version: i32,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) {
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

    pub async fn get_document(&self, uri: &Url) -> Option<Document> {
        self.documents.read().await.get(uri).cloned()
    }

    pub async fn get_program(&self, uri: &Url) -> Option<Program> {
        self.documents.read().await.get(uri)?.program.clone()
    }

    pub async fn get_text(&self, uri: &Url) -> Option<String> {
        self.documents.read().await.get(uri).map(|d| d.text.clone())
    }

    pub async fn get_line(&self, uri: &Url, line: u32) -> Option<String> {
        let text = self.documents.read().await.get(uri)?.text.clone();
        text.lines().nth(line as usize).map(|s| s.to_string())
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
