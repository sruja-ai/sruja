package lsp

import (
	"context"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestRename_CollectsEditsAcrossWorkspace(t *testing.T) {
	s := NewServer()
	uriDecl := lsp.DocumentURI("file:///decl.sruja")
	uriUse := lsp.DocumentURI("file:///use.sruja")
	textDecl := "system S \"System\""
	textUse := "S -> X"
	_ = s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uriDecl, Text: textDecl, Version: 1},
	})
	_ = s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uriUse, Text: textUse, Version: 1},
	})

	edit, err := s.Rename(context.Background(), lsp.RenameParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: uriDecl},
		Position:     lsp.Position{Line: 0, Character: 8},
		NewName:      "Sys",
	})
	if err != nil {
		t.Fatalf("rename error: %v", err)
	}
	if edit == nil {
		t.Fatalf("expected workspace edit")
	}
	if len(edit.Changes) < 2 {
		t.Fatalf("expected edits in two files, got %d", len(edit.Changes))
	}
	if _, ok := edit.Changes[string(uriDecl)]; !ok {
		t.Fatalf("expected change for decl.sruja")
	}
	if _, ok := edit.Changes[string(uriUse)]; !ok {
		t.Fatalf("expected change for use.sruja")
	}
}
