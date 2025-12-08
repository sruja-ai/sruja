package lsp

import (
	"context"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestCompletion(t *testing.T) {
	s := NewServer()
	uri := lsp.DocumentURI("file:///test.sruja")
	content := `architecture "Test" {
	system S "System"
}`
	s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{
			URI:  uri,
			Text: content,
		},
	})

	// Test keyword completion (empty prefix)
	params := lsp.CompletionParams{
		TextDocumentPositionParams: lsp.TextDocumentPositionParams{
			TextDocument: lsp.TextDocumentIdentifier{URI: uri},
			Position:     lsp.Position{Line: 0, Character: 0},
		},
	}
	list, err := s.Completion(context.Background(), params)
	if err != nil {
		t.Fatalf("Completion failed: %v", err)
	}
	if list == nil {
		t.Fatal("Expected completion list, got nil")
	}

	foundArch := false
	for _, item := range list.Items {
		if item.Label == "architecture" {
			foundArch = true
			break
		}
	}
	if !foundArch {
		t.Error("Expected 'architecture' keyword in completion list")
	}

	// Test ID completion
	// Position inside the block, after "system S"
	params.Position = lsp.Position{Line: 1, Character: 10}
	list, err = s.Completion(context.Background(), params)
	if err != nil {
		t.Fatalf("Completion failed: %v", err)
	}

	foundS := false
	for _, item := range list.Items {
		if item.Label == "S" {
			foundS = true
			break
		}
	}
	if !foundS {
		t.Error("Expected 'S' identifier in completion list")
	}
}

func TestCompletionVerbSuggestions(t *testing.T) {
	s := NewServer()
	uri := lsp.DocumentURI("file:///verbs.sruja")
	content := `system S "System"
container C "Container"
S -> C `
	s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uri, Text: content}})

	params := lsp.CompletionParams{TextDocumentPositionParams: lsp.TextDocumentPositionParams{TextDocument: lsp.TextDocumentIdentifier{URI: uri}, Position: lsp.Position{Line: 2, Character: 7}}}
	list, err := s.Completion(context.Background(), params)
	if err != nil {
		t.Fatalf("Completion failed: %v", err)
	}
	found := false
	for _, it := range list.Items {
		if it.Label == "reads" || it.Label == "uses" || it.Label == "calls" {
			found = true
			break
		}
	}
	if !found {
		t.Fatalf("Expected verb suggestions after relation arrow")
	}
}
