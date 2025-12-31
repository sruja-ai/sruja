package lsp

import (
	"context"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestServerDefinition_ExactAndFallback(t *testing.T) {
	uri := lsp.DocumentURI("file://defs.sruja")
	text := "System=kind \"System\" Container=kind \"Container\" Component=kind \"Component\"\n  S = System \"S\" {\n    C = Container \"C\" {\n      X = Component \"X\"\n    }\n  }\n\n"
	srv := NewServer()
	_ = srv.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uri, Text: text, Version: 1}})

	// Exact match for system
	locs, err := srv.Definition(context.Background(), lsp.TextDocumentPositionParams{TextDocument: lsp.TextDocumentIdentifier{URI: uri}, Position: lsp.Position{Line: 1, Character: 2}})
	if err != nil {
		t.Fatalf("definition error: %v", err)
	}
	if len(locs) == 0 {
		t.Fatalf("expected at least one location")
	}

	// Fallback using qualified name last segment
	locs2, err := srv.Definition(context.Background(), lsp.TextDocumentPositionParams{TextDocument: lsp.TextDocumentIdentifier{URI: uri}, Position: lsp.Position{Line: 3, Character: 6}})
	if err != nil {
		t.Fatalf("definition error: %v", err)
	}
	// cursor at 'X' should resolve to its declaration line
	if len(locs2) == 0 {
		t.Fatalf("expected location for component X")
	}
}
