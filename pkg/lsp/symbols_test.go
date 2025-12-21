package lsp

import (
	"context"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestDocumentSymbols_CollectsHierarchy(t *testing.T) {
	uri := lsp.DocumentURI("file://test.sruja")
	text := "model {\n" +
		"  system S {\n" +
		"    container C {\n" +
		"      component X\n" +
		"    }\n" +
		"    database D\n" +
		"    queue Q\n" +
		"  }\n" +
		"  person P \"Person\"\n" +
		"}\n"
	srv := NewServer()
	_ = srv.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uri, Text: text, Version: 1}})
	syms, err := srv.DocumentSymbols(context.Background(), lsp.DocumentSymbolParams{TextDocument: lsp.TextDocumentIdentifier{URI: uri}})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	kinds := make(map[string]lsp.SymbolKind)
	for _, s := range syms {
		kinds[s.Name] = s.Kind
	}
	want := map[string]lsp.SymbolKind{
		"S": lsp.SKClass,
		"C": lsp.SKModule,
		"X": lsp.SKFunction,
		"D": lsp.SKStruct,
		"Q": lsp.SKEnum,
		"P": lsp.SKInterface,
	}
	for name, kind := range want {
		if kinds[name] != kind {
			t.Fatalf("symbol %s kind=%v, want %v", name, kinds[name], kind)
		}
	}
}
func TestFindIDRange(t *testing.T) {
	doc := NewDocument("file://test.sruja", "  system MySystem \"Label\"", 1)
	id := "MySystem"
	r := findIDRange(doc, id)
	if r.Start.Character != 9 {
		t.Errorf("expected start character 9, got %d", r.Start.Character)
	}
	if r.End.Character != 17 {
		t.Errorf("expected end character 17, got %d", r.End.Character)
	}

	// Test case where ID is not in line
	r2 := findIDRange(doc, "Other")
	if r2.Start.Character != 0 || r2.End.Character != 0 {
		t.Error("expected full doc range (0,0) for non-existent ID")
	}
}

func TestFindNameRange(t *testing.T) {
	doc := NewDocument("file://test.sruja", "  system MySystem \"Label\"", 1)
	r := findNameRange(doc, "Label")
	if r.Start.Line != 0 || r.Start.Character != 19 {
		t.Errorf("expected range for Label, got %+v", r)
	}

	r2 := findNameRange(doc, "  ")
	if r2.Start.Line != 0 || r2.End.Line != 1 {
		t.Errorf("expected full doc range for empty name, got %+v", r2)
	}
}

func TestFindTextRange_NotFound(t *testing.T) {
	doc := NewDocument("file://test.sruja", "text", 1)
	r := findTextRange(doc, "missing")
	if r.Start.Line != 0 || r.End.Line != 1 {
		t.Errorf("expected full doc range for missing text, got %+v", r)
	}
}
