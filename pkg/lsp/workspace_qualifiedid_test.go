package lsp

import "testing"

func TestBuildQualifiedIDForWorkspace(t *testing.T) {
	if got := buildQualifiedIDForWorkspace(); got != "" {
		t.Fatalf("expected empty for no parts, got %q", got)
	}
	if got := buildQualifiedIDForWorkspace(""); got != "" {
		t.Fatalf("expected empty for empty part, got %q", got)
	}
	if got := buildQualifiedIDForWorkspace("A"); got != "A" {
		t.Fatalf("expected 'A', got %q", got)
	}
	if got := buildQualifiedIDForWorkspace("A", "", "B"); got != "A.B" {
		t.Fatalf("expected 'A.B', got %q", got)
	}
	if got := buildQualifiedIDForWorkspace("A", "B", "C"); got != "A.B.C" {
		t.Fatalf("expected 'A.B.C', got %q", got)
	}
}
