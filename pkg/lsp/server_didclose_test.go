package lsp

import (
    "context"
    "testing"
    "github.com/sourcegraph/go-lsp"
)

func TestServerDidCloseRemovesDoc(t *testing.T) {
    s := NewServer()
    uri := lsp.DocumentURI("file:///x.sruja")
    s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uri, Text: "architecture \"A\" {}"}})
    if s.workspace.GetDocument(uri) == nil { t.Fatalf("doc should exist after open") }
    s.DidClose(context.Background(), lsp.DidCloseTextDocumentParams{TextDocument: lsp.TextDocumentIdentifier{URI: uri}})
    if s.workspace.GetDocument(uri) != nil { t.Fatalf("doc should be removed after close") }
}

