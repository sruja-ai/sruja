package lsp

import (
	"context"
	"strings"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestServer_ADR_Features(t *testing.T) {
	dsl := `
Component = kind "Component"
adr = kind "Adr"
requirement = kind "Requirement"

ADR001 = adr "Test Decision" {
  status "accepted"
}
c1 = Component "c1"
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
		// 	ADR001 = adr ...
		// Line 5.
		// Tab = 1 char. Position 1 (A) to 6 (1).

		val, err := s.Hover(context.Background(), lsp.TextDocumentPositionParams{
			TextDocument: lsp.TextDocumentIdentifier{URI: uri},
			Position:     lsp.Position{Line: 5, Character: 2},
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
			if item.Label == "Adr" {
				foundAdr = true
			}
			if item.Label == "Requirement" {
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
