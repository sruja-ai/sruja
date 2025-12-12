package lsp

import (
    "context"
    "testing"

    "github.com/sourcegraph/go-lsp"
)

func TestDocumentSymbols_CollectsHierarchy(t *testing.T) {
    uri := lsp.DocumentURI("file://test.sruja")
    text := "architecture \"A\" {\n" +
        "  system S {\n" +
        "    container C {\n" +
        "      component X\n" +
        "    }\n" +
        "    datastore D\n" +
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
        "A": lsp.SKNamespace,
        "S": lsp.SKClass,
        "C": lsp.SKModule,
        "X": lsp.SKFunction,
        "D": lsp.SKStruct,
        "Q": lsp.SKEnum,
        "P": lsp.SKVariable,
    }
    for name, kind := range want {
        if kinds[name] != kind {
            t.Fatalf("symbol %s kind=%v, want %v", name, kinds[name], kind)
        }
    }
}

