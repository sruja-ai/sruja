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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_new() {
        let uri = Url::parse("file:///test.sruja").unwrap();
        let text = "test content".to_string();
        let version = 1;

        let doc = Document::new(uri.clone(), text.clone(), version);

        assert_eq!(*doc.uri(), uri);
        assert_eq!(doc.version(), version);
        assert_eq!(doc.text(), text);
        assert!(doc.program().is_none());
    }

    #[test]
    fn test_document_get_line() {
        let uri = Url::parse("file:///test.sruja").unwrap();
        let text = "line1\nline2\nline3".to_string();
        let doc = Document::new(uri, text, 1);

        assert_eq!(doc.get_line(0), Some("line1"));
        assert_eq!(doc.get_line(1), Some("line2"));
        assert_eq!(doc.get_line(2), Some("line3"));
        assert_eq!(doc.get_line(3), None);
    }

    #[test]
    fn test_document_lines() {
        let uri = Url::parse("file:///test.sruja").unwrap();
        let text = "line1\nline2\nline3".to_string();
        let doc = Document::new(uri, text, 1);

        let lines = doc.lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn test_document_apply_change_incremental() {
        let uri = Url::parse("file:///test.sruja").unwrap();
        let text = "hello world\nsecond line".to_string();
        let mut doc = Document::new(uri, text, 1);

        let change = TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 0,
                    character: 6,
                },
                end: Position {
                    line: 0,
                    character: 11,
                },
            }),
            range_length: Some(5),
            text: "rust".to_string(),
        };

        doc.apply_change(change);

        assert_eq!(doc.text(), "hello rust\nsecond line");
    }

    #[test]
    fn test_document_apply_change_full() {
        let uri = Url::parse("file:///test.sruja").unwrap();
        let text = "old content".to_string();
        let mut doc = Document::new(uri, text, 1);

        let change = TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "new content".to_string(),
        };

        doc.apply_change(change);

        assert_eq!(doc.text(), "new content");
    }

    #[test]
    fn test_document_parse_valid() {
        let uri = Url::parse("file:///test.sruja").unwrap();
        let text = r#"
app = system "My App" {
  description "Test"
}
"#
        .to_string();
        let mut doc = Document::new(uri, text, 1);

        doc.parse();

        assert!(doc.program().is_some());
    }

    #[test]
    fn test_document_parse_invalid() {
        let uri = Url::parse("file:///test.sruja").unwrap();
        let text = "invalid syntax here".to_string();
        let mut doc = Document::new(uri, text, 1);

        doc.parse();

        assert!(doc.program().is_none());
    }

    #[test]
    fn test_document_clone() {
        let uri = Url::parse("file:///test.sruja").unwrap();
        let text = "test content".to_string();
        let doc1 = Document::new(uri.clone(), text.clone(), 1);

        let doc2 = doc1.clone();

        assert_eq!(doc2.uri(), &uri);
        assert_eq!(doc2.version(), 1);
        assert_eq!(doc2.text(), text);
        assert_eq!(doc2.program(), doc1.program());
    }

    #[tokio::test]
    async fn test_workspace_new() {
        let workspace = Workspace::new();

        let uri = Url::parse("file:///test.sruja").unwrap();
        let text = "test".to_string();

        // Should not panic
        workspace.add_document(uri, text, 1).await;
    }

    #[tokio::test]
    async fn test_workspace_add_document() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///add.sruja").unwrap();
        let text = r#"
app = system "My App" {}
"#
        .to_string();

        workspace.add_document(uri.clone(), text, 1).await;

        let doc = workspace.get_document(&uri).await;
        assert!(doc.is_some());
        let unwrapped = doc.unwrap();
        assert_eq!(unwrapped.version(), 1);
        assert!(unwrapped.program().is_some());
    }

    #[tokio::test]
    async fn test_workspace_update_document() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///update.sruja").unwrap();
        let text = "original".to_string();

        workspace.add_document(uri.clone(), text, 1).await;

        let changes = vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "updated".to_string(),
        }];

        workspace.update_document(&uri, 2, changes).await;

        let doc = workspace.get_document(&uri).await;
        assert!(doc.is_some());
        let unwrapped = doc.unwrap();
        assert_eq!(unwrapped.version(), 2);
        assert_eq!(unwrapped.text(), "updated");
    }

    #[tokio::test]
    async fn test_workspace_update_nonexistent_document() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///nonexistent.sruja").unwrap();

        let changes = vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "text".to_string(),
        }];

        // Should not panic
        workspace.update_document(&uri, 1, changes).await;

        let doc = workspace.get_document(&uri).await;
        assert!(doc.is_none());
    }

    #[tokio::test]
    async fn test_workspace_remove_document() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///remove.sruja").unwrap();
        let text = "test".to_string();

        workspace.add_document(uri.clone(), text, 1).await;

        let doc = workspace.get_document(&uri).await;
        assert!(doc.is_some());

        workspace.remove_document(&uri).await;

        let doc = workspace.get_document(&uri).await;
        assert!(doc.is_none());
    }

    #[tokio::test]
    async fn test_workspace_get_document() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///get.sruja").unwrap();
        let text = "test content".to_string();

        workspace.add_document(uri.clone(), text.clone(), 1).await;

        let doc = workspace.get_document(&uri).await;
        assert!(doc.is_some());
        let unwrapped = doc.unwrap();
        assert_eq!(unwrapped.text(), text);
        assert_eq!(unwrapped.version(), 1);
    }

    #[tokio::test]
    async fn test_workspace_get_program() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///program.sruja").unwrap();
        let text = r#"
app = system "My App" {}
"#
        .to_string();

        workspace.add_document(uri.clone(), text, 1).await;

        let program = workspace.get_program(&uri).await;
        assert!(program.is_some());
    }

    #[tokio::test]
    async fn test_workspace_get_text() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///text.sruja").unwrap();
        let text = "test content\nline 2".to_string();

        workspace.add_document(uri.clone(), text.clone(), 1).await;

        let retrieved_text = workspace.get_text(&uri).await;
        assert!(retrieved_text.is_some());
        assert_eq!(retrieved_text.unwrap(), text);
    }

    #[tokio::test]
    async fn test_workspace_get_line() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///line.sruja").unwrap();
        let text = "line 1\nline 2\nline 3".to_string();

        workspace.add_document(uri.clone(), text, 1).await;

        assert_eq!(
            workspace.get_line(&uri, 0).await,
            Some("line 1".to_string())
        );
        assert_eq!(
            workspace.get_line(&uri, 1).await,
            Some("line 2".to_string())
        );
        assert_eq!(
            workspace.get_line(&uri, 2).await,
            Some("line 3".to_string())
        );
        assert_eq!(workspace.get_line(&uri, 3).await, None);
    }

    #[tokio::test]
    async fn test_workspace_default() {
        let workspace = Workspace::default();
        assert_eq!(workspace.documents.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_workspace_multiple_documents() {
        let workspace = Workspace::new();

        let uri1 = Url::parse("file:///doc1.sruja").unwrap();
        let uri2 = Url::parse("file:///doc2.sruja").unwrap();

        workspace
            .add_document(uri1.clone(), "content 1".to_string(), 1)
            .await;
        workspace
            .add_document(uri2.clone(), "content 2".to_string(), 1)
            .await;

        let doc1 = workspace.get_document(&uri1).await;
        let doc2 = workspace.get_document(&uri2).await;

        assert!(doc1.is_some());
        assert!(doc2.is_some());

        assert_eq!(doc1.unwrap().text(), "content 1");
        assert_eq!(doc2.unwrap().text(), "content 2");
    }
}
