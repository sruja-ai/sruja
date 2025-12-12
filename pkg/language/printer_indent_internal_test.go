package language

import "testing"

func TestPrinter_indent(t *testing.T) {
    p := NewPrinter()
    p.IndentLevel = 1
    if p.indent() != "  " {
        t.Fatalf("indent level 1 expected two spaces, got %q", p.indent())
    }
    p.IndentLevel = 25
    got := p.indent()
    if len(got) != 50 { // 25 * 2 spaces
        t.Fatalf("indent level 25 expected 50 spaces, got %d", len(got))
    }
}

