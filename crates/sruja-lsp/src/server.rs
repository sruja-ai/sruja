//! LSP server implementation
//!
//! This module provides the main Language Server Protocol server implementation
//! for the Sruja DSL.

use async_trait::async_trait;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, ClientSocket, LanguageServer, LspService, Server};

use sruja_engine::Validator;
use sruja_language::Parser;

use crate::diagnostics::convert_diagnostics_to_lsp;
use crate::features::*;
use crate::workspace::Workspace;

/// Sruja Language Server
pub struct SrujaLanguageServer {
    client: Client,
    workspace: Workspace,
    validator: Validator,
}

impl SrujaLanguageServer {
    pub fn new(client: Client) -> Self {
        let validator = Validator::with_default_rules();

        Self {
            client,
            workspace: Workspace::new(),
            validator,
        }
    }

    async fn publish_diagnostics(&self, uri: Url) {
        let text = match self.workspace.get_text(&uri).await {
            Some(text) => text,
            None => return,
        };

        // Parse document
        let parser = Parser::new(uri.to_string());

        let diagnostics = match parser.parse(&text) {
            Ok(program) => {
                // Get parser diagnostics (none if successful)
                // Run validation
                let validation_diagnostics = self.validator.validate_sync(&program);
                convert_diagnostics_to_lsp(&validation_diagnostics)
            }
            Err(parse_diagnostics) => {
                // Try to parse partially for validation
                // For now, skip validation if parse failed
                convert_diagnostics_to_lsp(&parse_diagnostics)
            }
        };

        // Publish diagnostics
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[async_trait]
impl LanguageServer for SrujaLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "sruja-lsp".to_string(),
                version: Some("1.0.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        will_save: None,
                        will_save_wait_until: None,
                        save: None,
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Sruja LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.workspace
            .add_document(
                uri.clone(),
                params.text_document.text,
                params.text_document.version,
            )
            .await;
        self.publish_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;
        self.workspace
            .update_document(&uri, version, params.content_changes)
            .await;
        self.publish_diagnostics(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.workspace.remove_document(&uri).await;

        // Clear diagnostics
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let doc = match self.workspace.get_document(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        let line = params.text_document_position_params.position.line as usize;
        let character = params.text_document_position_params.position.character as usize;

        // Parse program
        let text = doc.text.clone();
        let parser = Parser::new(uri.to_string());
        let program = match parser.parse(&text) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        Ok(get_hover(&doc, &program, line, character))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let doc = match self.workspace.get_document(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        let line = params.text_document_position.position.line as usize;
        let character = params.text_document_position.position.character as usize;

        // Parse program
        let text = doc.text.clone();
        let parser = Parser::new(uri.to_string());
        let program = parser.parse(&text).unwrap_or_default();

        let items = get_completion(&doc, &program, line, character);
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let doc = match self.workspace.get_document(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        let line = params.text_document_position_params.position.line as usize;
        let character = params.text_document_position_params.position.character as usize;

        let line_text = match doc.get_line(line) {
            Some(l) => l,
            None => return Ok(None),
        };

        let (start, end) = word_bounds(line_text, character);
        let word = line_text[start..end].trim();

        if word.is_empty() {
            return Ok(None);
        }

        // Parse program
        let text = doc.text.clone();
        let parser = Parser::new(uri.to_string());
        let program = match parser.parse(&text) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        if let Some(location) = find_definition(&doc, &program, word) {
            Ok(Some(GotoDefinitionResponse::Scalar(location)))
        } else {
            Ok(None)
        }
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let doc = match self.workspace.get_document(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        let line = params.text_document_position.position.line as usize;
        let character = params.text_document_position.position.character as usize;

        let line_text = match doc.get_line(line) {
            Some(l) => l,
            None => return Ok(None),
        };

        let (start, end) = word_bounds(line_text, character);
        let word = line_text[start..end].trim();

        if word.is_empty() {
            return Ok(None);
        }

        // Parse program
        let text = doc.text.clone();
        let parser = Parser::new(uri.to_string());
        let program = match parser.parse(&text) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let locations = find_references(&doc, &program, word);
        Ok(Some(locations))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.clone();
        let doc = match self.workspace.get_document(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        // Parse program
        let text = doc.text.clone();
        let parser = Parser::new(uri.to_string());
        let program = match parser.parse(&text) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let symbols = get_document_symbols(&doc, &program);
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.clone();
        let doc = match self.workspace.get_document(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        // Parse program
        let text = doc.text.clone();
        let parser = Parser::new(uri.to_string());
        let program = match parser.parse(&text) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        Ok(format_document(&doc, &program))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let doc = match self.workspace.get_document(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        let line = params.text_document_position.position.line as usize;
        let character = params.text_document_position.position.character as usize;

        let line_text = match doc.get_line(line) {
            Some(l) => l,
            None => return Ok(None),
        };

        let (start, end) = word_bounds(line_text, character);
        let old_name = line_text[start..end].trim();

        if old_name.is_empty() {
            return Ok(None);
        }

        // Find all references
        let text = doc.text.clone();
        let parser = Parser::new(uri.to_string());
        let program = match parser.parse(&text) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let locations = find_references(&doc, &program, old_name);

        // Create text edits
        let mut changes = std::collections::HashMap::new();
        let mut edits = Vec::new();

        for location in locations {
            edits.push(TextEdit {
                range: location.range,
                new_text: params.new_name.clone(),
            });
        }

        changes.insert(uri, edits);

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
    }

    async fn code_action(&self, _params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        // TODO: Implement code actions (quick fixes)
        // This will provide quick fixes for common errors like:
        // - Create missing elements
        // - Add missing descriptions
        // - Fix duplicate IDs
        Ok(None)
    }
}

/// Create and start LSP server
pub fn create_lsp_service() -> (LspService<SrujaLanguageServer>, ClientSocket) {
    LspService::new(SrujaLanguageServer::new)
}

/// Run the LSP server on stdio
pub async fn run_stdio() -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = create_lsp_service();
    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}
