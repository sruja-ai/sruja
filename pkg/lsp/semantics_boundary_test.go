package lsp

import "testing"

func TestIsBoundary(t *testing.T) {
    // keyword with spaces around
    if !isBoundary(" architecture ", 1, len("architecture")) {
      t.Fatalf("expected boundary for spaced keyword")
    }
    // embedded within identifier should be false
    if isBoundary("myarchitectureX", 2, len("architecture")) {
      t.Fatalf("expected no boundary within identifier")
    }
    // punctuation boundaries
    if !isBoundary("(architecture)", 1, len("architecture")) {
      t.Fatalf("expected boundary with punctuation")
    }
}

