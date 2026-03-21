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

        let locations = find_definition(&doc, &program, word);
        match locations.len() {
            0 => Ok(None),
            1 => Ok(Some(GotoDefinitionResponse::Scalar(locations[0].clone()))),
            _ => Ok(Some(GotoDefinitionResponse::Array(locations))),
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

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri.clone();
        let doc = match self.workspace.get_document(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        let text = doc.text.clone();
        let parser = Parser::new(uri.to_string());
        let program = match parser.parse(&text) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let mut actions: Vec<CodeActionOrCommand> = Vec::new();

        for diag in &params.context.diagnostics {
            if let Some(NumberOrString::String(code)) = &diag.code {
                let line = diag.range.start.line as usize;
                let character = diag.range.start.character as usize;

                match code.as_str() {
                    "E302" => {
                        if let Some(action) =
                            create_add_description_action(&uri, &doc, line, character)
                        {
                            actions.push(CodeActionOrCommand::CodeAction(action));
                        }
                    }
                    "E202" => {
                        if let Some(action) =
                            create_missing_element_action(&uri, &doc, &program, line, character)
                        {
                            actions.push(CodeActionOrCommand::CodeAction(action));
                        }
                    }
                    "E205" => {
                        if let Some(action) =
                            create_add_relation_action(&uri, &doc, &program, line, character)
                        {
                            actions.push(CodeActionOrCommand::CodeAction(action));
                        }
                    }
                    _ => {}
                }
            }
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
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

fn create_add_description_action(
    uri: &Url,
    doc: &crate::workspace::Document,
    line: usize,
    _character: usize,
) -> Option<CodeAction> {
    let line_text = doc.get_line(line)?;
    let trimmed = line_text.trim();

    let kind_prefix = [
        "system",
        "container",
        "component",
        "database",
        "datastore",
        "queue",
        "person",
    ]
    .iter()
    .find(|prefix| trimmed.starts_with(*prefix))?;

    let rest = trimmed.strip_prefix(kind_prefix)?.trim();
    let name_end = rest
        .find(|c: char| c.is_whitespace() || c == '{')
        .unwrap_or(rest.len());
    let element_name = &rest[..name_end];

    let insert_pos = line_text.find('{')?;
    let indent = "  ".repeat(line_text.len() - line_text.trim_start().len() + 1);
    let new_text = format!("\n{}description \"TODO: Add description\"", indent);

    Some(CodeAction {
        title: format!("Add description to '{}'", element_name),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some({
                let mut map = std::collections::HashMap::new();
                map.insert(
                    uri.clone(),
                    vec![TextEdit {
                        range: Range {
                            start: Position {
                                line: line as u32,
                                character: insert_pos as u32 + 1,
                            },
                            end: Position {
                                line: line as u32,
                                character: insert_pos as u32 + 1,
                            },
                        },
                        new_text,
                    }],
                );
                map
            }),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

fn create_missing_element_action(
    uri: &Url,
    doc: &crate::workspace::Document,
    program: &sruja_language::ast::Program,
    line: usize,
    _character: usize,
) -> Option<CodeAction> {
    let line_text = doc.get_line(line)?;

    let missing_name = extract_identifier_from_error(line_text)?;

    let (elements, _) = crate::features::collect_elements(program);
    if elements.contains_key(&missing_name) {
        return None;
    }

    let last_line = doc.lines().len();
    let last_non_empty = (0..last_line).rev().find(|&i| {
        doc.get_line(i)
            .map(|l| !l.trim().is_empty())
            .unwrap_or(false)
    })?;

    let insert_line = last_non_empty + 1;
    let new_text = format!(
        "\n\n{} = system \"{}\" {{\n  description \"TODO: Add description\"\n}}",
        missing_name, missing_name
    );

    Some(CodeAction {
        title: format!("Create missing element '{}'", missing_name),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some({
                let mut map = std::collections::HashMap::new();
                map.insert(
                    uri.clone(),
                    vec![TextEdit {
                        range: Range {
                            start: Position {
                                line: insert_line as u32,
                                character: 0,
                            },
                            end: Position {
                                line: insert_line as u32,
                                character: 0,
                            },
                        },
                        new_text,
                    }],
                );
                map
            }),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

fn create_add_relation_action(
    uri: &Url,
    doc: &crate::workspace::Document,
    program: &sruja_language::ast::Program,
    line: usize,
    _character: usize,
) -> Option<CodeAction> {
    let line_text = doc.get_line(line)?;

    let trimmed = line_text.trim();
    let kind_prefix = [
        "system",
        "container",
        "component",
        "database",
        "datastore",
        "queue",
        "person",
    ]
    .iter()
    .find(|prefix| trimmed.starts_with(*prefix))?;

    let rest = trimmed.strip_prefix(kind_prefix)?.trim();
    let name_end = rest
        .find(|c: char| c.is_whitespace() || c == '{' || c == '"')
        .unwrap_or(rest.len());
    let element_name = rest[..name_end].trim_matches('"').to_string();

    let (elements, _) = crate::features::collect_elements(program);
    let target = elements.keys().next()?;

    let last_line = doc.lines().len();
    let last_non_empty = (0..last_line).rev().find(|&i| {
        doc.get_line(i)
            .map(|l| !l.trim().is_empty())
            .unwrap_or(false)
    })?;

    let insert_line = last_non_empty + 1;
    let new_text = format!("\n{} -> {} \"uses\"", element_name, target);

    Some(CodeAction {
        title: format!("Add relation from '{}' to '{}'", element_name, target),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some({
                let mut map = std::collections::HashMap::new();
                map.insert(
                    uri.clone(),
                    vec![TextEdit {
                        range: Range {
                            start: Position {
                                line: insert_line as u32,
                                character: 0,
                            },
                            end: Position {
                                line: insert_line as u32,
                                character: 0,
                            },
                        },
                        new_text,
                    }],
                );
                map
            }),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(false),
        disabled: None,
        data: None,
    })
}

fn extract_identifier_from_error(line: &str) -> Option<String> {
    if let Some(arrow_idx) = line.find("->") {
        let right_side = &line[arrow_idx + 2..];
        let trimmed = right_side.trim();
        let end = trimmed
            .find(|c: char| c.is_whitespace() || c == '"')
            .unwrap_or(trimmed.len());
        let ident = &trimmed[..end];
        if !ident.is_empty() {
            return Some(ident.to_string());
        }
    }
    None
}
