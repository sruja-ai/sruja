package lsp

import (
	"context"
	"strings"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestServer_ADR_Features(t *testing.T) {
	dsl := `
specification {
	element component
}
model {
	adr ADR001 "Test Decision" {
		status "Accepted"
	}
	component c1
}
`
	s := NewServer()
	uri := lsp.DocumentURI("file:///test.sruja")
	s.workspace.AddDocument(uri, dsl, 1)

	// 1. Verify Definition/Symbols
	t.Run("Symbols", func(t *testing.T) {
		symbols, err := s.DocumentSymbols(context.Background(), lsp.DocumentSymbolParams{
			TextDocument: lsp.TextDocumentIdentifier{URI: uri},
		})
		if err != nil {
			t.Fatalf("DocumentSymbols failed: %v", err)
		}
		found := false
		for _, sym := range symbols {
			if sym.Name == "ADR001" && sym.Kind == lsp.SKString {
				found = true
				break
			}
		}
		if !found {
			t.Error("ADR001 symbol not found or incorrect kind")
		}
	})

	// 2. Verify Hover
	t.Run("Hover", func(t *testing.T) {
		// Mock position for "ADR001"
		// adr ADR001 ...
		// 0123456789
		// Line 6 (0-indexed base on dsl string above? Let's check lines)
		// Line 0: empty
		// Line 1: specification {
		// Line 2: element component
		// Line 3: }
		// Line 4: model {
		// Line 5: adr ADR001 "Test Decision" {

		val, err := s.Hover(context.Background(), lsp.TextDocumentPositionParams{
			TextDocument: lsp.TextDocumentIdentifier{URI: uri},
			Position:     lsp.Position{Line: 5, Character: 6},
		})
		if err != nil {
			t.Fatalf("Hover failed: %v", err)
		}
		if val == nil {
			t.Fatal("Hover returned nil")
		}
		content := val.Contents[0].Value
		if !strings.Contains(content, "**ADR**") || !strings.Contains(content, "Test Decision") {
			t.Errorf("Unexpected hover content: %s", content)
		}
	})

	// 3. Verify Completion
	t.Run("Completion", func(t *testing.T) {
		list, err := s.Completion(context.Background(), lsp.CompletionParams{
			TextDocumentPositionParams: lsp.TextDocumentPositionParams{
				TextDocument: lsp.TextDocumentIdentifier{URI: uri},
				Position:     lsp.Position{Line: 5, Character: 0},
			},
		})
		if err != nil {
			t.Fatalf("Completion failed: %v", err)
		}
		foundAdr := false
		foundReq := false
		for _, item := range list.Items {
			if item.Label == "adr" {
				foundAdr = true
			}
			if item.Label == "requirement" {
				foundReq = true
			}
		}
		if !foundAdr {
			t.Error("Completion 'adr' not found")
		}
		if !foundReq {
			t.Error("Completion 'requirement' not found")
		}
	})
}
