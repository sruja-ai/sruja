package lsp

import (
	"github.com/sourcegraph/go-lsp"
	"testing"
)

func TestFindTextRange_NotFoundReturnsFull(t *testing.T) {
	doc := NewDocument(lsp.DocumentURI("file://t"), "line1\nline2", 1)
	r := findTextRange(doc, "missing")
	full := fullDocRange(doc)
	if r != full {
		t.Fatalf("expected full doc range when text not found; got %+v want %+v", r, full)
	}
}
