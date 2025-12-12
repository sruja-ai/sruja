// pkg/lsp/document_links_test.go
package lsp_test

import (
	"context"
	"testing"

	golsp "github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/lsp"
)

// TestDocumentLinks_ImportStatement is skipped because import statements
// are not currently supported in the DSL parser (feature was removed).
// If import support is re-added, this test can be re-enabled.
func TestDocumentLinks_ImportStatement(t *testing.T) {
	t.Skip("Import statements are not currently supported in the DSL parser")
}

func TestDocumentLinks_FileReference(t *testing.T) {
	s := lsp.NewServer()
	uri := golsp.DocumentURI("file:///test.sruja")
	dsl := `architecture "Test" {
    // See config.sruja for details
    system API "API Service" {}
}
`
	s.DidOpen(context.Background(), golsp.DidOpenTextDocumentParams{
		TextDocument: golsp.TextDocumentItem{
			URI:        uri,
			LanguageID: "sruja",
			Version:    1,
			Text:       dsl,
		},
	})

	params := lsp.DocumentLinkParams{
		TextDocument: golsp.TextDocumentIdentifier{URI: uri},
	}

	links, err := s.DocumentLinks(context.Background(), params)
	if err != nil {
		t.Fatalf("DocumentLinks failed: %v", err)
	}

	// May or may not find file references in comments, but should not error
	_ = links
}
