package lsp

import (
	"github.com/sourcegraph/go-lsp"
	"testing"
)

func TestDocument_ApplyChange_SetText(t *testing.T) {
	doc := NewDocument(lsp.DocumentURI("file://x"), "line1\nline2", 1)
	doc.ApplyChange(lsp.TextDocumentContentChangeEvent{Text: "new\ntext"})
	if doc.Text != "new\ntext" {
		t.Fatalf("expected SetText to replace content, got %q", doc.Text)
	}
}

func TestDocument_ApplyChange_RangeReplace(t *testing.T) {
	doc := NewDocument(lsp.DocumentURI("file://x"), "abc\ndef", 1)
	rng := lsp.Range{Start: lsp.Position{Line: 0, Character: 1}, End: lsp.Position{Line: 1, Character: 1}}
	doc.ApplyChange(lsp.TextDocumentContentChangeEvent{Range: &rng, Text: "X"})
	// Replace from position (0,1) to (1,1): "bc\n d" -> "X"
	if got := doc.Text; got != "aXef" {
		t.Fatalf("unexpected text after range replace: %q", got)
	}
}
