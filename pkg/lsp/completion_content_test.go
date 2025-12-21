package lsp

import (
	"context"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestCompletion_KeywordsAndPrefix(t *testing.T) {
	s := NewServer()
	uri := lsp.DocumentURI("file:///comp.sruja")
	text := ""
	_ = s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text, Version: 1},
	})

	// At start, expect keywords including "system"
	res, err := s.Completion(context.Background(), lsp.CompletionParams{
		TextDocumentPositionParams: lsp.TextDocumentPositionParams{
			TextDocument: lsp.TextDocumentIdentifier{URI: uri},
			Position:     lsp.Position{Line: 0, Character: 0},
		},
	})
	if err != nil {
		t.Fatalf("completion error: %v", err)
	}
	foundSystem := false
	for _, it := range res.Items {
		if it.Label == "system" {
			foundSystem = true
			break
		}
	}
	if !foundSystem {
		t.Fatalf("expected 'system' keyword in completion items")
	}

	// With prefix "sys", expect "system" to be suggested
	text2 := "sys"
	_ = s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text2, Version: 2},
	})
	res2, err := s.Completion(context.Background(), lsp.CompletionParams{
		TextDocumentPositionParams: lsp.TextDocumentPositionParams{
			TextDocument: lsp.TextDocumentIdentifier{URI: uri},
			Position:     lsp.Position{Line: 0, Character: 3},
		},
	})
	if err != nil {
		t.Fatalf("completion error: %v", err)
	}
	foundSystem2 := false
	for _, it := range res2.Items {
		if it.Label == "system" {
			foundSystem2 = true
			break
		}
	}
	if !foundSystem2 {
		t.Fatalf("expected 'system' for prefix 'sys'")
	}
}

func TestCompletion_RelationVerbSuggestions(t *testing.T) {
	s := NewServer()
	uri := lsp.DocumentURI("file:///verbs.sruja")
	text := "model {\n  system S {\n    container C {}\n  }\n  S -> C \n}\n"
	_ = s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text, Version: 1},
	})

	res, err := s.Completion(context.Background(), lsp.CompletionParams{
		TextDocumentPositionParams: lsp.TextDocumentPositionParams{
			TextDocument: lsp.TextDocumentIdentifier{URI: uri},
			Position:     lsp.Position{Line: 4, Character: 9},
		},
	})
	if err != nil {
		t.Fatalf("completion error: %v", err)
	}
	foundUses := false
	for _, it := range res.Items {
		if it.Label == "uses" {
			foundUses = true
			break
		}
	}
	if !foundUses {
		t.Fatalf("expected verb 'uses' after relation arrow")
	}
}
