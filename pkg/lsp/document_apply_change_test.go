package lsp

import (
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestDocument_ApplyChange_RebuildDefs(t *testing.T) {
	text := `system S "System"
`
	d := NewDocument(lsp.DocumentURI("file:///t.sruja"), text, 1)
	if _, ok := d.defs["S"]; !ok {
		t.Fatalf("expected initial def S")
	}
	line := d.GetLine(0)
	idx := Index(line, "S")
	if idx < 0 {
		t.Fatalf("S not found in line")
	}
	// Replace S with Shop
	change := lsp.TextDocumentContentChangeEvent{
		Range: &lsp.Range{
			Start: lsp.Position{Line: 0, Character: idx},
			End:   lsp.Position{Line: 0, Character: idx + 1},
		},
		Text: "Shop",
	}
	d.ApplyChange(change)
	if _, ok := d.defs["S"]; ok {
		t.Fatalf("unexpected old def S present after change")
	}
	if _, ok := d.defs["Shop"]; !ok {
		t.Fatalf("expected new def Shop after change")
	}
}

// Index finds the first index of substr in s, returns -1 if not found.
func Index(s, substr string) int {
	for i := 0; i+len(substr) <= len(s); i++ {
		if s[i:i+len(substr)] == substr {
			return i
		}
	}
	return -1
}
