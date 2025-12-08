package lsp

import (
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestWorkspace_ManageDocuments(t *testing.T) {
	ws := NewWorkspace()
	uri := lsp.DocumentURI("file:///test.sruja")
	content := "architecture Test {}"

	// Add
	ws.AddDocument(uri, content, 1)
	doc := ws.GetDocument(uri)
	if doc == nil {
		t.Fatal("Expected document to be added")
	}
	if doc.Text != content {
		t.Errorf("Expected content '%s', got '%s'", content, doc.Text)
	}
	if doc.Version != 1 {
		t.Errorf("Expected version 1, got %d", doc.Version)
	}

	// Remove
	ws.RemoveDocument(uri)
	doc = ws.GetDocument(uri)
	if doc != nil {
		t.Error("Expected document to be removed")
	}
}

func TestDocument_ApplyChange(t *testing.T) {
	uri := lsp.DocumentURI("file:///test.sruja")
	// Initial:
	// line 0: Hello
	// line 1: World
	content := "Hello\nWorld"
	doc := NewDocument(uri, content, 1)

	// Test full text replacement
	changeFull := lsp.TextDocumentContentChangeEvent{
		Text: "New Content",
	}
	doc.ApplyChange(changeFull)
	if doc.Text != "New Content" {
		t.Errorf("Expected 'New Content', got '%s'", doc.Text)
	}

	// Reset
	doc.SetText("Hello\nWorld")

	// Test incremental update
	// Replace "World" with "Go"
	// Range: line 1, char 0 to line 1, char 5
	changeInc := lsp.TextDocumentContentChangeEvent{
		Range: &lsp.Range{
			Start: lsp.Position{Line: 1, Character: 0},
			End:   lsp.Position{Line: 1, Character: 5},
		},
		Text: "Go",
	}
	doc.ApplyChange(changeInc)
	expected := "Hello\nGo"
	if doc.Text != expected {
		t.Errorf("Expected '%s', got '%s'", expected, doc.Text)
	}

	// Test insertion
	// Insert "!" at end
	changeInsert := lsp.TextDocumentContentChangeEvent{
		Range: &lsp.Range{
			Start: lsp.Position{Line: 1, Character: 2},
			End:   lsp.Position{Line: 1, Character: 2},
		},
		Text: "!",
	}
	doc.ApplyChange(changeInsert)
	expected = "Hello\nGo!"
	if doc.Text != expected {
		t.Errorf("Expected '%s', got '%s'", expected, doc.Text)
	}
}
