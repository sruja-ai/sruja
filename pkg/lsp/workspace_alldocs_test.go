package lsp

import (
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestWorkspace_AllDocuments(t *testing.T) {
	w := NewWorkspace()
	uri1 := lsp.DocumentURI("file:///one.sruja")
	uri2 := lsp.DocumentURI("file:///two.sruja")
	w.AddDocument(uri1, "system A \"A\"", 1)
	w.AddDocument(uri2, "system B \"B\"", 1)
	docs := w.AllDocuments()
	if len(docs) != 2 {
		t.Fatalf("expected 2 documents, got %d", len(docs))
	}
}
