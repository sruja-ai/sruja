package lsp

import "testing"

func TestIsTokenBoundary(t *testing.T) {
    if !isTokenBoundary(" a ", 1, "a") || !isTokenBoundary("\na\n", 1, "a") {
        t.Fatalf("expected whitespace to be token boundary")
    }
    if isTokenBoundary("zaz", 1, "a") || isTokenBoundary("_a_", 1, "a") || isTokenBoundary("1a1", 1, "a") {
        t.Fatalf("expected letters/digits/underscore to not be token boundary")
    }
}
