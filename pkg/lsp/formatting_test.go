package lsp

import (
	"context"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestDocumentFormatting(t *testing.T) {
	s := NewServer()
	uri := lsp.DocumentURI("file:///test.sruja")
	// Unformatted content
	content := `System=kind "System"
S=System "System"`
	s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{
			URI:  uri,
			Text: content,
		},
	})

	params := lsp.DocumentFormattingParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: uri},
	}
	edits, err := s.DocumentFormatting(context.Background(), params)
	if err != nil {
		t.Fatalf("Formatting failed: %v", err)
	}
	if len(edits) != 1 {
		t.Fatalf("Expected 1 text edit, got %d", len(edits))
	}

	formatted := edits[0].NewText
	if formatted == content {
		t.Error("Expected formatted text to be different from input")
	}
	// Basic check for printer output structure

}
