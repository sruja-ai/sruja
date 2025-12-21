package lsp

import (
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestDocument_EnsureParsed_CacheAndInvalidate(t *testing.T) {
	text := `model { system S "System" }`
	d := NewDocument(lsp.DocumentURI("file:///cache.sruja"), text, 1)
	p1 := d.EnsureParsed()
	if p1 == nil {
		t.Fatalf("EnsureParsed returned nil program")
	}
	// Second call should return cached pointer
	p2 := d.EnsureParsed()
	if p2 != p1 {
		t.Fatalf("expected cached program pointer equality")
	}
	// Mutate text and ensure cache invalidates
	d.SetText(`model { system X "X" }`)
	p3 := d.EnsureParsed()
	if p3 == nil || p3 == p1 {
		t.Fatalf("expected new program after SetText")
	}
}
