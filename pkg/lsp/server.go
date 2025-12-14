package lsp

import (
	"context"
	"encoding/json"
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
		return nil
	}
	program, diags, err := p.Parse(string(uri), doc.Text)

	var lspDiagnostics []lsp.Diagnostic
	if len(diags) > 0 {
		lspDiagnostics = append(lspDiagnostics, s.convertDiagnosticsToLSP(diags)...)
	}

	if err == nil && program != nil {
		diags := s.validator.Validate(program)
		lspDiagnostics = append(lspDiagnostics, s.convertDiagnosticsToLSP(diags)...)
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

//nolint:funlen,gocyclo // Server start logic is complex and requires length
func StartServer(in io.Reader, out io.Writer) error {
	srv := NewServer()
	handler := jsonrpc2.HandlerWithError(func(ctx context.Context, conn *jsonrpc2.Conn, req *jsonrpc2.Request) (interface{}, error) {
		srv.conn = conn
		switch req.Method {
		case "initialize":
			var params lsp.InitializeParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			return srv.Initialize(ctx, params)
		case "textDocument/didOpen":
			var params lsp.DidOpenTextDocumentParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			return nil, srv.DidOpen(ctx, params)
		case "textDocument/didChange":
			var params lsp.DidChangeTextDocumentParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			return nil, srv.DidChange(ctx, params)
		case "textDocument/didClose":
			var params lsp.DidCloseTextDocumentParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			return nil, srv.DidClose(ctx, params)
		case "textDocument/formatting":
			var params lsp.DocumentFormattingParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			edits, err := srv.DocumentFormatting(ctx, params)
			return edits, err
		case "textDocument/hover":
			var params lsp.TextDocumentPositionParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			h, err := srv.Hover(ctx, params)
			return h, err
		case "textDocument/completion":
			var params lsp.CompletionParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			c, err := srv.Completion(ctx, params)
			return c, err
		case "textDocument/definition":
			var params lsp.TextDocumentPositionParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			def, err := srv.Definition(ctx, params)
			return def, err
		case "textDocument/references":
			var params lsp.ReferenceParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			refs, err := srv.References(ctx, params)
			return refs, err
		case "textDocument/documentSymbol":
			var params lsp.DocumentSymbolParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			syms, err := srv.DocumentSymbols(ctx, params)
			return syms, err
		case "textDocument/semanticTokens/full":
			var params struct {
				TextDocument lsp.TextDocumentIdentifier `json:"textDocument"`
			}
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			toks, err := srv.SemanticTokensFull(ctx, params.TextDocument)
			return toks, err
		case "textDocument/semanticTokens":
			// Some clients call this path with a nested "full" param
			var params struct {
				TextDocument lsp.TextDocumentIdentifier `json:"textDocument"`
			}
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			toks, err := srv.SemanticTokensFull(ctx, params.TextDocument)
			return toks, err
		case "textDocument/semanticTokens/legend":
			legend := srv.SemanticTokensLegend()
			return legend, nil
		case "textDocument/codeAction":
			var params lsp.CodeActionParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			actions, err := srv.CodeAction(ctx, params)
			return actions, err
		case "textDocument/rename":
			var params lsp.RenameParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			edit, err := srv.Rename(ctx, params)
			return edit, err
		case "textDocument/documentLink":
			var params DocumentLinkParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
			}
			links, err := srv.DocumentLinks(ctx, params)
			return links, err
		case "textDocument/foldingRange":
			var params FoldingRangeParams
			if req.Params != nil {
				_ = json.Unmarshal(*req.Params, &params)
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
