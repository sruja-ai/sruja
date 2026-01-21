//! LSP server implementation
//!
//! This module provides the main Language Server Protocol server implementation
//! for the Sruja DSL.

use async_trait::async_trait;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use sruja_engine::Validator;
use sruja_language::Parser;

use crate::diagnostics::convert_diagnostics_to_lsp;
use crate::workspace::Workspace;

/// Sruja Language Server
pub struct SrujaLanguageServer {
    client: Client,
    workspace: Workspace,
    validator: Validator,
}

impl SrujaLanguageServer {
    pub fn new(client: Client) -> Self {
        let mut validator = Validator::new();
        validator.register_default_rules();
        
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

        // Parse the document
        let parser = Parser::new(uri.to_string());
        let mut diagnostics = Vec::new();
        
        match parser.parse(&text) {
            Ok(program) => {
                // Get parser diagnostics (none if successful)
                // Run validation
                let validation_diagnostics = self.validator.validate_sync(&program);
                diagnostics = convert_diagnostics_to_lsp(&validation_diagnostics);
            }
            Err(parse_diagnostics) => {
                diagnostics = convert_diagnostics_to_lsp(&parse_diagnostics);
                
                // Try to parse partially for validation
                // For now, skip validation if parse failed
            }
        }

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
                        change: Some(TextDocumentSyncKind::INCR),
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
                workspace_symbol_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "Sruja LSP server initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.workspace
            .add_document(uri.clone(), params.text_document.text, params.text_document.version)
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
        self.client
            .publish_diagnostics(uri, vec![], None)
            .await;
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        // TODO: Implement hover
        Ok(None)
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // TODO: Implement completion
        Ok(None)
    }

    async fn goto_definition(&self, _params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        // TODO: Implement definition
        Ok(None)
    }

    async fn references(&self, _params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        // TODO: Implement references
        Ok(None)
    }

    async fn document_symbol(&self, _params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        // TODO: Implement document symbols
        Ok(None)
    }

    async fn workspace_symbol(&self, _params: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        // TODO: Implement workspace symbols
        Ok(None)
    }

    async fn formatting(&self, _params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        // TODO: Implement formatting
        Ok(None)
    }

    async fn rename(&self, _params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        // TODO: Implement rename
        Ok(None)
    }

    async fn code_action(&self, _params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        // TODO: Implement code actions
        Ok(None)
    }
}

/// Create and start the LSP server
pub fn create_lsp_service() -> LspService<SrujaLanguageServer> {
    LspService::new(|client| SrujaLanguageServer::new(client))
}

/// Run the LSP server on stdio
pub async fn run_stdio() -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = create_lsp_service();
    Server::new(stdin, stdout, socket).serve(service).await;
}
