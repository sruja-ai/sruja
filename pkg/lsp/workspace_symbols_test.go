package lsp

import (
	"context"
	"github.com/sourcegraph/go-lsp"
	"testing"
)

func TestWorkspaceSymbols(t *testing.T) {
	srv := NewServer()
	uri1 := lsp.DocumentURI("file:///a.sruja")
	uri2 := lsp.DocumentURI("file:///b.sruja")
	srv.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uri1, Text: "system A \"A\""}})
	srv.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uri2, Text: "system B \"B\""}})

	syms, err := srv.WorkspaceSymbols(context.Background(), lsp.WorkspaceSymbolParams{Query: ""})
	if err != nil {
		t.Fatalf("WorkspaceSymbols failed: %v", err)
	}
	if len(syms) < 2 {
		t.Fatalf("Expected at least 2 symbols, got %d", len(syms))
	}

	symsA, err := srv.WorkspaceSymbols(context.Background(), lsp.WorkspaceSymbolParams{Query: "A"})
	if err != nil {
		t.Fatalf("WorkspaceSymbols failed: %v", err)
	}
	found := false
	for _, s := range symsA {
		if s.Name == "A" {
			if s.Kind != lsp.SKClass {
				t.Fatalf("Expected kind Class for A, got %v", s.Kind)
			}
			found = true
			break
		}
	}
	if !found {
		t.Fatalf("Expected symbol A in results")
	}
}
