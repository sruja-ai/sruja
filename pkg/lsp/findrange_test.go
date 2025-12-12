package lsp

import (
    "testing"
    "github.com/sourcegraph/go-lsp"
)

func TestFindNameRange_EmptyAndTrimmed(t *testing.T) {
    text := "architecture \"A\" {\n  system S {\n    container C {\n      component X\n    }\n  }\n}\n"
    doc := NewDocument(lsp.DocumentURI("file://test.sruja"), text, 1)

    rEmpty := findNameRange(doc, "  ")
    full := fullDocRange(doc)
    if rEmpty != full {
        t.Fatalf("expected full range for empty name, got %+v want %+v", rEmpty, full)
    }

    r := findNameRange(doc, "  S  ")
    if r.Start.Line != 1 || r.End.Line != 1 {
        t.Fatalf("expected range on line 1 for 'S', got %+v", r)
    }
    if r.End.Character-r.Start.Character != len("S") {
        t.Fatalf("unexpected char span for 'S': %+v", r)
    }
}

