package json

import "testing"

func TestStringPtrToIntPtr_InvalidReturnsNil(t *testing.T) {
    x := "abc"
    if stringPtrToIntPtr(&x) != nil { t.Fatalf("expected nil for non-numeric") }
    if stringPtrToIntPtr(nil) != nil { t.Fatalf("expected nil for nil input") }
}

func TestIdOrLabel(t *testing.T) {
    if idOrLabel("id", "label") != "id" { t.Fatalf("expected id preference") }
    if idOrLabel("", "label") != "label" { t.Fatalf("expected label fallback") }
}

func TestSanitize(t *testing.T) {
    if sanitize("  a  ") != "a" { t.Fatalf("expected trim") }
}

