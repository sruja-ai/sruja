package lsp

import (
	"testing"

	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestEnsureParsed_WithStdlib(t *testing.T) {
	// Test that EnsureParsed loads stdlib
	dsl := `
person = kind "Person"
User = person "User"
`

	doc := NewDocument("file:///test.sruja", dsl, 1)
	prog := doc.EnsureParsed()

	if prog == nil {
		t.Fatal("Expected program to be parsed")
	}

	// Verify program has model
	if prog.Model == nil {
		t.Error("Expected model to be present")
	}
}

func TestEnsureParsed_CachesProgram(t *testing.T) {
	dsl := `User = person "User"`

	doc := NewDocument("file:///test.sruja", dsl, 1)

	// First parse
	prog1 := doc.EnsureParsed()
	if prog1 == nil {
		t.Fatal("Expected program to be parsed")
	}

	// Second call should return cached program
	prog2 := doc.EnsureParsed()
	if prog1 != prog2 {
		t.Error("Expected cached program to be returned")
	}
}

func TestLoadStdLibIntoWorkspace(t *testing.T) {
	p, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	ws := language.NewWorkspace()
	// Just verify calling this function doesn't panic
	loadStdLibIntoWorkspace(p, ws)

	// The workspace map should exist (may or may not have stdlib depending on test env)
	if ws.Programs == nil {
		t.Error("Expected Programs map to exist")
	}
}

func TestDocument_rebuildDefs_View(t *testing.T) {
	// Test view definitions are tracked
	dsl := `
view index {
  title "Main View"
  include *
}
`
	doc := NewDocument("file:///test.sruja", dsl, 1)

	// rebuildDefs is called in NewDocument, just verify it doesn't crash
	if doc.defs == nil {
		t.Error("Expected defs to be initialized")
	}
}

func TestDocument_ApplyChange_FullReplace(t *testing.T) {
	dsl := `User = person "User"`
	doc := NewDocument("file:///test.sruja", dsl, 1)

	// Apply full content change (no range)
	newContent := `Admin = person "Admin"`
	doc.ApplyChange(lsp.TextDocumentContentChangeEvent{Text: newContent})

	if doc.Text != newContent {
		t.Errorf("Expected text to be updated, got: %s", doc.Text)
	}
}
