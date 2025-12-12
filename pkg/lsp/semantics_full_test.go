package lsp

import (
    "context"
    "testing"
    "github.com/sourcegraph/go-lsp"
)

func TestSemanticTokens_LegendAndFull(t *testing.T) {
    s := NewServer()
    // Include keywords, declarations, quoted strings and arrow operator
    text := "architecture \"A\" {\n  system S \"Shop\" {\n    container C \"Web\"\n  }\n}\nS->C"
    uri := lsp.DocumentURI("file:///sem.sruja")
    s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uri, Text: text}})

    legend := s.SemanticTokensLegend()
    if len(legend.TokenTypes) == 0 || len(legend.TokenModifiers) == 0 { t.Fatalf("legend missing entries") }

    toks, err := s.SemanticTokensFull(context.Background(), lsp.TextDocumentIdentifier{URI: uri})
    if err != nil { t.Fatalf("semantic tokens error: %v", err) }
    if len(toks.Data) == 0 { t.Fatalf("expected tokens data") }
}

