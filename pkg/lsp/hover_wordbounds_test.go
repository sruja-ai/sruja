package lsp

import "testing"

func TestWordBounds(t *testing.T) {
	line := `system Shop "Shop System"`
	// Position inside identifier
	start, end := wordBounds(line, 8) // 'p' in Shop
	if line[start:end] != "Shop" {
		t.Fatalf("expected 'Shop', got %q", line[start:end])
	}
	// At space between tokens
	s2, e2 := wordBounds(line, 6) // space after 'system'
	if s2 != 0 || e2 != 6 {
		t.Fatalf("expected previous token bounds at space (system), got %d:%d", s2, e2)
	}
	// At quotes should yield empty
	s3, e3 := wordBounds(line, 12) // at first quote
	if s3 != e3 {
		t.Fatalf("expected empty bounds at quote, got %d:%d", s3, e3)
	}
}
