package lsp

import (
	"github.com/sourcegraph/go-lsp"
	"testing"
)

func TestWorkspace_FindDefinition_ExactAndQualifiedFallback(t *testing.T) {
	w := NewWorkspace()
	uri := lsp.DocumentURI("file://defs.sruja")
	text := "architecture \"A\" {\n  system S {\n    container C {\n      component X\n    }\n  }\n}\n"
	w.AddDocument(uri, text, 1)

	// Exact match
	u, r, ok := w.FindDefinition("S")
	if !ok || u != uri || r.Start.Line != 1 {
		t.Fatalf("expected exact match for S at line 1, got ok=%v uri=%v range=%+v", ok, u, r)
	}

	// Qualified fallback by last segment
	u2, r2, ok2 := w.FindDefinition("S.C.X")
	if !ok2 || u2 != uri || r2.Start.Line != 3 {
		t.Fatalf("expected fallback match for X at line 3, got ok=%v uri=%v range=%+v", ok2, u2, r2)
	}
}
