package lsp

import "testing"

func TestMax0(t *testing.T) {
	if max0(-5) != 0 {
		t.Fatalf("max0(-5) should be 0")
	}
	if max0(3) != 3 {
		t.Fatalf("max0(3) should be 3")
	}
}
