package lsp

import (
	"context"
	"github.com/sourcegraph/go-lsp"
	"testing"
)

func TestReferences(t *testing.T) {
	srv := NewServer()
	uri := lsp.DocumentURI("file:///refs.sruja")
	text := "system S \"System\"\ncontainer C \"Container\"\nS -> C \"uses\""
	srv.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text},
	})

	locs, err := srv.References(context.Background(), lsp.ReferenceParams{
		TextDocumentPositionParams: lsp.TextDocumentPositionParams{
			TextDocument: lsp.TextDocumentIdentifier{URI: uri},
			Position:     lsp.Position{Line: 2, Character: 0},
		},
	})
	if err != nil {
		t.Fatalf("References failed: %v", err)
	}
	if len(locs) < 2 {
		t.Fatalf("Expected at least two references for S (declaration and usage), got %d", len(locs))
	}
}

func TestReferences_CrossFile(t *testing.T) {
	srv := NewServer()
	uriDecl := lsp.DocumentURI("file:///decl.sruja")
	uriUse := lsp.DocumentURI("file:///use.sruja")
	textDecl := "system S \"System\""
	textUse := "S -> X"
	srv.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uriDecl, Text: textDecl}})
	srv.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uriUse, Text: textUse}})

	locs, err := srv.References(context.Background(), lsp.ReferenceParams{TextDocumentPositionParams: lsp.TextDocumentPositionParams{TextDocument: lsp.TextDocumentIdentifier{URI: uriDecl}, Position: lsp.Position{Line: 0, Character: 8}}})
	if err != nil {
		t.Fatalf("References failed: %v", err)
	}
	if len(locs) < 2 {
		t.Fatalf("Expected at least two references across files, got %d", len(locs))
	}
}
