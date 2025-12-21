package lsp

import (
	"context"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestDocumentLinks_FileURL_CreatesLink(t *testing.T) {
	s := NewServer()
	uri := lsp.DocumentURI("tmp/docs/test.sruja")
	text := `// see file://tmp/docs/examples/config.sruja for details`
	_ = s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text, Version: 1},
	})

	links, err := s.DocumentLinks(context.Background(), DocumentLinkParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: uri},
	})
	if err != nil {
		t.Fatalf("DocumentLinks error: %v", err)
	}
	if len(links) == 0 {
		t.Fatalf("expected at least one link for file:// URL")
	}
	if links[0].Target == nil {
		t.Fatalf("expected target in document link")
	}
	if got := string(*links[0].Target); got != "file://tmp/docs/examples/config.sruja" {
		t.Fatalf("unexpected target: %q", got)
	}
}

func TestDocumentLinks_QuotedRelativePath_CreatesLink(t *testing.T) {
	s := NewServer()
	uri := lsp.DocumentURI("tmp/docs/test.sruja")
	text := `// refer to "examples/config.sruja" in repo`
	_ = s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text, Version: 1},
	})

	links, err := s.DocumentLinks(context.Background(), DocumentLinkParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: uri},
	})
	if err != nil {
		t.Fatalf("DocumentLinks error: %v", err)
	}
	if len(links) == 0 {
		t.Fatalf("expected at least one link for quoted .sruja path")
	}
	if links[0].Target == nil {
		t.Fatalf("expected target in document link")
	}
	// Should resolve relative to currentDir "tmp/docs"
	wantPrefix := "file://tmp/docs/examples/config.sruja"
	if got := string(*links[0].Target); got != wantPrefix {
		t.Fatalf("unexpected target: got %q want %q", got, wantPrefix)
	}
}
