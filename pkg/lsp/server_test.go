package lsp

import (
	"context"
	"io"
	"testing"
	"time"

	"github.com/sourcegraph/go-lsp"
	"github.com/sourcegraph/jsonrpc2"
)

func TestNewServer(t *testing.T) {
	server := NewServer()
	if server == nil {
		t.Fatal("NewServer returned nil")
	}
	// Check if validator is initialized (indirectly)
	// We can't access private fields, but we can verify public behavior
}

func TestInitialize(t *testing.T) {
	server := NewServer()
	params := lsp.InitializeParams{}
	result, err := server.Initialize(context.Background(), params)
	if err != nil {
		t.Fatalf("Initialize failed: %v", err)
	}

	if result.Capabilities.TextDocumentSync == nil {
		t.Error("Expected TextDocumentSync capability")
	}
	if !result.Capabilities.HoverProvider {
		t.Error("Expected HoverProvider capability")
	}
	if result.Capabilities.CompletionProvider == nil {
		t.Error("Expected CompletionProvider capability")
	}
}

func TestDidOpen(t *testing.T) {
	server := NewServer()
	uri := lsp.DocumentURI("file:///test.sruja")
	text := `system S "System"`

	params := lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{
			URI:        uri,
			LanguageID: "sruja",
			Version:    1,
			Text:       text,
		},
	}

	// This might fail or log error because connection is nil, but it shouldn't panic
	err := server.DidOpen(context.Background(), params)
	if err != nil {
		t.Fatalf("DidOpen failed: %v", err)
	}

	// Verify document was added to workspace (we need to access workspace via some way or trust it works)
	// Since workspace is private, we can't check it directly.
	// But we can check if Hover works on it.
}

func TestHover(t *testing.T) {
	server := NewServer()
	uri := lsp.DocumentURI("file:///test.sruja")
	text := `system S "System"`

	// Open document first
	server.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{
			URI: uri, Text: text,
		},
	})

	// Hover over "system" (keyword) or "S" (identifier)
	// "system" is at 0:0
	params := lsp.TextDocumentPositionParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: uri},
		Position:     lsp.Position{Line: 0, Character: 0},
	}

	_, err := server.Hover(context.Background(), params)
	if err != nil {
		t.Fatalf("Hover failed: %v", err)
	}

	// Hover might be nil if nothing found, but it shouldn't error

}

func TestDidChange(t *testing.T) {
	server := NewServer()
	uri := lsp.DocumentURI("file:///test.sruja")
	text := `system S "System"`

	server.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text},
	})

	// Change "System" to "System Updated"
	params := lsp.DidChangeTextDocumentParams{
		TextDocument: lsp.VersionedTextDocumentIdentifier{
			TextDocumentIdentifier: lsp.TextDocumentIdentifier{URI: uri},
			Version:                2,
		},
		ContentChanges: []lsp.TextDocumentContentChangeEvent{
			{
				Range: &lsp.Range{
					Start: lsp.Position{Line: 0, Character: 10},
					End:   lsp.Position{Line: 0, Character: 16},
				},
				Text: "Updated",
			},
		},
	}

	err := server.DidChange(context.Background(), params)
	if err != nil {
		t.Fatalf("DidChange failed: %v", err)
	}
}

func TestDidClose(t *testing.T) {
	server := NewServer()
	uri := lsp.DocumentURI("file:///test.sruja")

	server.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: `system S "System"`},
	})

	err := server.DidClose(context.Background(), lsp.DidCloseTextDocumentParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: uri},
	})
	if err != nil {
		t.Fatalf("DidClose failed: %v", err)
	}
}

// pipeReadWriteCloser adapts a pipe to ReadWriteCloser
type pipeReadWriteCloser struct {
	*io.PipeReader
	*io.PipeWriter
}

func (p pipeReadWriteCloser) Close() error {
	err1 := p.PipeReader.Close()
	err2 := p.PipeWriter.Close()
	if err1 != nil {
		return err1
	}
	return err2
}

func TestStartServer(t *testing.T) {
	// Create pipes for communication
	// clientWriter -> serverReader (server stdin)
	// serverWriter (server stdout) -> clientReader
	serverReader, clientWriter := io.Pipe()
	clientReader, serverWriter := io.Pipe()

	// Run server in goroutine
	serverErrCh := make(chan error, 1)
	go func() {
		serverErrCh <- StartServer(serverReader, serverWriter)
	}()

	// Create client connection
	clientStream := pipeReadWriteCloser{clientReader, clientWriter}
	clientConn := jsonrpc2.NewConn(context.Background(), jsonrpc2.NewBufferedStream(clientStream, jsonrpc2.VSCodeObjectCodec{}), jsonrpc2.HandlerWithError(func(_ context.Context, _ *jsonrpc2.Conn, _ *jsonrpc2.Request) (interface{}, error) {
		return nil, nil
	}))

	// Send initialize request
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	initParams := lsp.InitializeParams{
		RootURI: "file:///workspace",
	}
	var initResult lsp.InitializeResult
	if err := clientConn.Call(ctx, "initialize", initParams, &initResult); err != nil {
		t.Fatalf("Initialize call failed: %v", err)
	}

	if !initResult.Capabilities.HoverProvider {
		t.Error("Expected HoverProvider capability")
	}

	// Send didOpen notification
	didOpenParams := lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{
			URI:        "file:///workspace/test.sruja",
			LanguageID: "sruja",
			Version:    1,
			Text:       `system S "System"`,
		},
	}
	if err := clientConn.Notify(ctx, "textDocument/didOpen", didOpenParams); err != nil {
		t.Fatalf("DidOpen notify failed: %v", err)
	}

	// Send hover request
	hoverParams := lsp.TextDocumentPositionParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: "file:///workspace/test.sruja"},
		Position:     lsp.Position{Line: 0, Character: 0},
	}
	var hoverResult *lsp.Hover
	if err := clientConn.Call(ctx, "textDocument/hover", hoverParams, &hoverResult); err != nil {
		t.Fatalf("Hover call failed: %v", err)
	}

	// Send didChange notification
	didChangeParams := lsp.DidChangeTextDocumentParams{
		TextDocument: lsp.VersionedTextDocumentIdentifier{
			TextDocumentIdentifier: lsp.TextDocumentIdentifier{URI: "file:///workspace/test.sruja"},
			Version:                2,
		},
		ContentChanges: []lsp.TextDocumentContentChangeEvent{
			{
				Range: &lsp.Range{
					Start: lsp.Position{Line: 0, Character: 10},
					End:   lsp.Position{Line: 0, Character: 16},
				},
				Text: "Updated",
			},
		},
	}
	if err := clientConn.Notify(ctx, "textDocument/didChange", didChangeParams); err != nil {
		t.Fatalf("DidChange notify failed: %v", err)
	}

	// Send completion request
	completionParams := lsp.CompletionParams{
		TextDocumentPositionParams: lsp.TextDocumentPositionParams{
			TextDocument: lsp.TextDocumentIdentifier{URI: "file:///workspace/test.sruja"},
			Position:     lsp.Position{Line: 0, Character: 0},
		},
	}
	var completionResult *lsp.CompletionList
	if err := clientConn.Call(ctx, "textDocument/completion", completionParams, &completionResult); err != nil {
		t.Fatalf("Completion call failed: %v", err)
	}

	// Send formatting request
	formattingParams := lsp.DocumentFormattingParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: "file:///workspace/test.sruja"},
	}
	var formattingResult []lsp.TextEdit
	if err := clientConn.Call(ctx, "textDocument/formatting", formattingParams, &formattingResult); err != nil {
		t.Fatalf("Formatting call failed: %v", err)
	}

	// Send didClose notification
	didCloseParams := lsp.DidCloseTextDocumentParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: "file:///workspace/test.sruja"},
	}
	if err := clientConn.Notify(ctx, "textDocument/didClose", didCloseParams); err != nil {
		t.Fatalf("DidClose notify failed: %v", err)
	}

	// Close connection to stop server
	clientConn.Close()

	// Wait for server to stop
	select {
	case err := <-serverErrCh:
		if err != nil {
			t.Errorf("Server exited with error: %v", err)
		}
	case <-time.After(5 * time.Second):
		t.Error("Server did not stop after connection close")
	}
}
