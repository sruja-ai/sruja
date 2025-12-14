package lsp

import (
	"context"
	"path/filepath"
	"testing"

	golsp "github.com/sourcegraph/go-lsp"
)

func TestDocumentLinks_SrujaPathCreatesLink(t *testing.T) {
	s := NewServer()
	// Use plain path-style URI (matches server's document link resolver expectations)
	uri := golsp.DocumentURI("tmp/docs/test.sruja")
	dsl := `// see examples/config.sruja for configuration`
	s.DidOpen(context.Background(), golsp.DidOpenTextDocumentParams{
		TextDocument: golsp.TextDocumentItem{
			URI:        uri,
			LanguageID: "sruja",
			Version:    1,
			Text:       dsl,
		},
	})

	params := DocumentLinkParams{TextDocument: golsp.TextDocumentIdentifier{URI: uri}}
	links, err := s.DocumentLinks(context.Background(), params)
	if err != nil {
		t.Fatalf("DocumentLinks failed: %v", err)
	}
	if len(links) == 0 || links[0].Target == nil {
		t.Fatalf("expected at least one link with target, got %d", len(links))
	}
	// Validate target path resolution contains examples/config.sruja
	target := string(*links[0].Target)
	want := filepath.Join("tmp", "docs", "examples", "config.sruja")
	if !contains(target, want) {
		t.Fatalf("expected target to include %q, got %q", want, target)
	}
}

func contains(s, sub string) bool {
	return len(s) >= len(sub) && (indexOf(s, sub) >= 0)
}

func indexOf(s, sub string) int {
	for i := 0; i+len(sub) <= len(s); i++ {
		if s[i:i+len(sub)] == sub {
			return i
		}
	}
	return -1
}
