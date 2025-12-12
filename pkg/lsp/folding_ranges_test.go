// pkg/lsp/folding_ranges_test.go
package lsp_test

import (
	"context"
	"testing"

	golsp "github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/lsp"
)

func TestFoldingRanges_BasicArchitecture(t *testing.T) {
	s := lsp.NewServer()
	uri := golsp.DocumentURI("file:///test.sruja")
	dsl := `
architecture "Test" {
    system API "API Service" {
        container WebApp "Web App" {}
    }
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

	params := lsp.FoldingRangeParams{
		TextDocument: golsp.TextDocumentIdentifier{URI: uri},
	}

	ranges, err := s.FoldingRanges(context.Background(), params)
	if err != nil {
		t.Fatalf("FoldingRanges failed: %v", err)
	}

	if len(ranges) == 0 {
		t.Error("Expected at least one folding range")
	}

	// Should have folding ranges for architecture and system blocks
	foundArch := false
	foundSystem := false
	for _, r := range ranges {
		if r.StartLine == 1 { // architecture block starts at line 2 (0-based)
			foundArch = true
		}
		if r.StartLine == 2 { // system block starts at line 3 (0-based)
			foundSystem = true
		}
	}

	if !foundArch {
		t.Error("Expected folding range for architecture block")
	}
	if !foundSystem {
		t.Error("Expected folding range for system block")
	}
}

func TestFoldingRanges_MultipleBlocks(t *testing.T) {
	s := lsp.NewServer()
	uri := golsp.DocumentURI("file:///test.sruja")
	dsl := `
architecture "Test" {
    system API "API Service" {
        container WebApp "Web App" {}
    }
    system DB "Database" {}
    scenario LoginFlow "Login Flow" {
        User -> API "Login"
    }
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

	params := lsp.FoldingRangeParams{
		TextDocument: golsp.TextDocumentIdentifier{URI: uri},
	}

	ranges, err := s.FoldingRanges(context.Background(), params)
	if err != nil {
		t.Fatalf("FoldingRanges failed: %v", err)
	}

	if len(ranges) < 2 {
		t.Errorf("Expected at least 2 folding ranges, got %d", len(ranges))
	}
	// Log the ranges for debugging
	for i, r := range ranges {
		t.Logf("Folding range %d: lines %d-%d", i+1, r.StartLine, r.EndLine)
	}
}
