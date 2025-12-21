package lsp

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"

	"github.com/sourcegraph/go-lsp"
	"github.com/sourcegraph/jsonrpc2"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

type Server struct {
	conn      *jsonrpc2.Conn
	workspace *Workspace
	validator *engine.Validator
}

func NewServer() *Server {
	v := engine.NewValidator()
	v.RegisterRule(&engine.UniqueIDRule{})
	v.RegisterRule(&engine.ValidReferenceRule{})
	v.RegisterRule(&engine.CycleDetectionRule{})
	v.RegisterRule(&engine.OrphanDetectionRule{})
	v.RegisterRule(&engine.SimplicityRule{})
	v.RegisterRule(&engine.LayerViolationRule{})
	v.RegisterRule(&engine.ScenarioFQNRule{})
	return &Server{
		workspace: NewWorkspace(),
		validator: v,
	}
}

//nolint:gocritic,revive // params is large but standard, unused param required by signature
func (s *Server) Initialize(_ context.Context, _ lsp.InitializeParams) (*lsp.InitializeResult, error) {
	return &lsp.InitializeResult{
		Capabilities: lsp.ServerCapabilities{
			TextDocumentSync:           &lsp.TextDocumentSyncOptionsOrKind{Options: &lsp.TextDocumentSyncOptions{OpenClose: true, Change: lsp.TDSKIncremental}},
			HoverProvider:              true,
			CompletionProvider:         &lsp.CompletionOptions{TriggerCharacters: []string{".", ":"}},
			DefinitionProvider:         true,
			ReferencesProvider:         true,
			CodeActionProvider:         true, // Enable code actions (using Command-based approach)
			DocumentSymbolProvider:     true,
			WorkspaceSymbolProvider:    true,
			DocumentFormattingProvider: true,
			RenameProvider:             true, // Enable rename provider
			// Note: DocumentLinkProvider and FoldingRangeProvider are not in go-lsp ServerCapabilities
			// but we handle the requests manually in the switch statement below
		},
	}, nil
}

func (s *Server) DidOpen(_ context.Context, params lsp.DidOpenTextDocumentParams) error {
	doc := params.TextDocument
	s.workspace.AddDocument(doc.URI, doc.Text, doc.Version)
	return s.publishDiagnostics(doc.URI)
}

func (s *Server) DidChange(_ context.Context, params lsp.DidChangeTextDocumentParams) error {
	doc := s.workspace.GetDocument(params.TextDocument.URI)
	if doc == nil {
		return nil
	}
	for _, change := range params.ContentChanges {
		doc.ApplyChange(change)
	}
	return s.publishDiagnostics(params.TextDocument.URI)
}

func (s *Server) DidClose(_ context.Context, params lsp.DidCloseTextDocumentParams) error {
	s.workspace.RemoveDocument(params.TextDocument.URI)
	return s.notifyDiagnostics(params.TextDocument.URI, []lsp.Diagnostic{})
}

func (s *Server) publishDiagnostics(uri lsp.DocumentURI) error {
	doc := s.workspace.GetDocument(uri)
	if doc == nil {
		return nil
	}
	p, err := language.NewParser()
	if err != nil {
		// Log error but don't fail diagnostics - return empty diagnostics
		// This allows the LSP to continue functioning even if parser creation fails
		return s.notifyDiagnostics(uri, []lsp.Diagnostic{})
	}
	program, diags, parseErr := p.Parse(string(uri), doc.Text)
	_ = parseErr // Ignore parse errors, we still want to show diagnostics

	var lspDiagnostics []lsp.Diagnostic
	if len(diags) > 0 {
		lspDiagnostics = append(lspDiagnostics, s.convertDiagnosticsToLSP(diags)...)
	}

	// Run validation even if parse had errors, as partial programs may still be validatable
	if program != nil {
		validatorDiags := s.validator.Validate(program)
		lspDiagnostics = append(lspDiagnostics, s.convertDiagnosticsToLSP(validatorDiags)...)
	}

	return s.notifyDiagnostics(uri, lspDiagnostics)
}

func (s *Server) notifyDiagnostics(uri lsp.DocumentURI, diagnostics []lsp.Diagnostic) error {
	if s.conn == nil {
		return nil
	}
	params := lsp.PublishDiagnosticsParams{URI: uri, Diagnostics: diagnostics}
	return s.conn.Notify(context.Background(), "textDocument/publishDiagnostics", params)
}

type stdioStream struct {
	in  io.Reader
	out io.Writer
}

func (s stdioStream) Read(p []byte) (int, error)  { return s.in.Read(p) }
func (s stdioStream) Write(p []byte) (int, error) { return s.out.Write(p) }
func (s stdioStream) Close() error                { return nil }

func StartStdioServer() error {
	return StartServer(os.Stdin, os.Stdout)
}

// unmarshalParams safely unmarshals JSON-RPC request parameters into the target type.
// Returns an error if unmarshaling fails, allowing proper error propagation.
func unmarshalParams[T any](params *json.RawMessage) (T, error) {
	var result T
	if params == nil {
		return result, nil
	}
	if err := json.Unmarshal(*params, &result); err != nil {
		return result, fmt.Errorf("failed to unmarshal params for %T: %w", result, err)
	}
	return result, nil
}

//nolint:funlen,gocyclo // Server start logic is complex and requires length
func StartServer(in io.Reader, out io.Writer) error {
	srv := NewServer()
	handler := jsonrpc2.HandlerWithError(func(ctx context.Context, conn *jsonrpc2.Conn, req *jsonrpc2.Request) (interface{}, error) {
		srv.conn = conn
		switch req.Method {
		case "initialize":
			params, err := unmarshalParams[lsp.InitializeParams](req.Params)
			if err != nil {
				return nil, err
			}
			return srv.Initialize(ctx, params)
		case "textDocument/didOpen":
			params, err := unmarshalParams[lsp.DidOpenTextDocumentParams](req.Params)
			if err != nil {
				return nil, err
			}
			return nil, srv.DidOpen(ctx, params)
		case "textDocument/didChange":
			params, err := unmarshalParams[lsp.DidChangeTextDocumentParams](req.Params)
			if err != nil {
				return nil, err
			}
			return nil, srv.DidChange(ctx, params)
		case "textDocument/didClose":
			params, err := unmarshalParams[lsp.DidCloseTextDocumentParams](req.Params)
			if err != nil {
				return nil, err
			}
			return nil, srv.DidClose(ctx, params)
		case "textDocument/formatting":
			params, err := unmarshalParams[lsp.DocumentFormattingParams](req.Params)
			if err != nil {
				return nil, err
			}
			edits, err := srv.DocumentFormatting(ctx, params)
			return edits, err
		case "textDocument/hover":
			params, err := unmarshalParams[lsp.TextDocumentPositionParams](req.Params)
			if err != nil {
				return nil, err
			}
			h, err := srv.Hover(ctx, params)
			return h, err
		case "textDocument/completion":
			params, err := unmarshalParams[lsp.CompletionParams](req.Params)
			if err != nil {
				return nil, err
			}
			c, err := srv.Completion(ctx, params)
			return c, err
		case "textDocument/definition":
			params, err := unmarshalParams[lsp.TextDocumentPositionParams](req.Params)
			if err != nil {
				return nil, err
			}
			def, err := srv.Definition(ctx, params)
			return def, err
		case "textDocument/references":
			params, err := unmarshalParams[lsp.ReferenceParams](req.Params)
			if err != nil {
				return nil, err
			}
			refs, err := srv.References(ctx, params)
			return refs, err
		case "textDocument/documentSymbol":
			params, err := unmarshalParams[lsp.DocumentSymbolParams](req.Params)
			if err != nil {
				return nil, err
			}
			syms, err := srv.DocumentSymbols(ctx, params)
			return syms, err
		case "textDocument/semanticTokens/full":
			var params struct {
				TextDocument lsp.TextDocumentIdentifier `json:"textDocument"`
			}
			if req.Params != nil {
				if err := json.Unmarshal(*req.Params, &params); err != nil {
					return nil, fmt.Errorf("failed to unmarshal semanticTokens/full params: %w", err)
				}
			}
			toks, err := srv.SemanticTokensFull(ctx, params.TextDocument)
			return toks, err
		case "textDocument/semanticTokens":
			// Some clients call this path with a nested "full" param
			var params struct {
				TextDocument lsp.TextDocumentIdentifier `json:"textDocument"`
			}
			if req.Params != nil {
				if err := json.Unmarshal(*req.Params, &params); err != nil {
					return nil, fmt.Errorf("failed to unmarshal semanticTokens params: %w", err)
				}
			}
			toks, err := srv.SemanticTokensFull(ctx, params.TextDocument)
			return toks, err
		case "textDocument/semanticTokens/legend":
			legend := srv.SemanticTokensLegend()
			return legend, nil
		case "textDocument/codeAction":
			params, err := unmarshalParams[lsp.CodeActionParams](req.Params)
			if err != nil {
				return nil, err
			}
			actions, err := srv.CodeAction(ctx, params)
			return actions, err
		case "textDocument/rename":
			params, err := unmarshalParams[lsp.RenameParams](req.Params)
			if err != nil {
				return nil, err
			}
			edit, err := srv.Rename(ctx, params)
			return edit, err
		case "textDocument/documentLink":
			params, err := unmarshalParams[DocumentLinkParams](req.Params)
			if err != nil {
				return nil, err
			}
			links, err := srv.DocumentLinks(ctx, params)
			return links, err
		case "textDocument/foldingRange":
			params, err := unmarshalParams[FoldingRangeParams](req.Params)
			if err != nil {
				return nil, err
			}
			ranges, err := srv.FoldingRanges(ctx, params)
			return ranges, err
		default:
			// Unhandled methods: respond with nil
			return nil, nil
		}
	})
	stream := jsonrpc2.NewBufferedStream(stdioStream{in: in, out: out}, jsonrpc2.VSCodeObjectCodec{})
	conn := jsonrpc2.NewConn(context.Background(), stream, handler)
	<-conn.DisconnectNotify()
	return nil
}
