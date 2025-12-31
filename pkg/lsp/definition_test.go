//nolint:goconst // Test data repetition
package lsp

import (
	"context"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestDefinition(t *testing.T) {
	srv := NewServer()
	uri := lsp.DocumentURI("file:///test.sruja")
	text := "System=kind \"System\"\nContainer=kind \"Container\"\nS = System \"System\"\nC = Container \"Container\"\n"
	srv.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text, Version: 1},
	})

	locs, err := srv.Definition(context.Background(), lsp.TextDocumentPositionParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: uri},
		Position:     lsp.Position{Line: 2, Character: 0},
	})
	if err != nil {
		t.Fatalf("Definition failed: %v", err)
	}
	if len(locs) == 0 {
		t.Fatalf("Expected at least one location")
	}
}

func TestDefinition_CrossFile(t *testing.T) {
	srv := NewServer()
	uriDecl := lsp.DocumentURI("file:///decl.sruja")
	uriUse := lsp.DocumentURI("file:///use.sruja")
	textDecl := "System=kind \"System\"\nS = System \"System\"\n"
	textUse := "S -> X\n"
	srv.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uriDecl, Text: textDecl, Version: 1}})
	srv.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uriUse, Text: textUse, Version: 1}})

	locs, err := srv.Definition(context.Background(), lsp.TextDocumentPositionParams{TextDocument: lsp.TextDocumentIdentifier{URI: uriUse}, Position: lsp.Position{Line: 0, Character: 0}})
	if err != nil {
		t.Fatalf("Definition failed: %v", err)
	}
	if len(locs) == 0 {
		t.Fatalf("Expected a location for cross-file definition")
	}
	if string(locs[0].URI) != string(uriDecl) {
		t.Fatalf("Expected definition in decl.sruja, got %s", locs[0].URI)
	}
}
