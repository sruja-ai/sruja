package lsp

import (
	"context"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestFoldingRanges_ASTBlocks(t *testing.T) {
	s := NewServer()
	uri := lsp.DocumentURI("file:///fold.sruja")
	text := "architecture \"A\" {\n  system S {\n    container C {\n    }\n  }\n}\n"
	_ = s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text, Version: 1},
	})

	ranges, err := s.FoldingRanges(context.Background(), FoldingRangeParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: uri},
	})
	if err != nil {
		t.Fatalf("FoldingRanges error: %v", err)
	}
	if len(ranges) == 0 {
		t.Fatalf("expected folding ranges for architecture/system/container blocks")
	}
}

func TestFoldingRanges_TextFallback(t *testing.T) {
	s := NewServer()
	uri := lsp.DocumentURI("file:///fold_text.sruja")
	// Malformed content to force parser error, but has braces for fallback
	text := "notdsl {\n  broken\n}\n"
	_ = s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text, Version: 1},
	})

	ranges, err := s.FoldingRanges(context.Background(), FoldingRangeParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: uri},
	})
	if err != nil {
		t.Fatalf("FoldingRanges error: %v", err)
	}
	if len(ranges) == 0 {
		t.Fatalf("expected folding ranges from text fallback")
	}
}
