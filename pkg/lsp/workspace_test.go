package lsp

import (
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestWorkspace_ManageDocuments(t *testing.T) {
	ws := NewWorkspace()
	uri := lsp.DocumentURI("file:///test.sruja")
	content := "model {}"

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
func TestRebuildDefs_Complex(t *testing.T) {
	content := `model {
  system S1 {
    container C1 {
      component Comp1
    }
  }
  system S2 {
    datastore D1
    queue Q1
  }
  person P1
  adr ADR1
  requirement REQ1
  policy POL1
}`
	doc := NewDocument("file:///test.sruja", content, 1)

	tests := []struct {
		id   string
		want lsp.SymbolKind
	}{
		{"S1", lsp.SKClass},
		{"S1.C1", lsp.SKModule},
		{"S1.C1.Comp1", lsp.SKFunction},
		{"S2.D1", lsp.SKStruct},
		{"S2.Q1", lsp.SKEnum},
		{"P1", lsp.SKVariable},
		{"ADR1", lsp.SKString},
		{"REQ1", lsp.SKProperty},
		{"POL1", lsp.SKConstant},
	}

	for _, tt := range tests {
		if got, ok := doc.defKinds[tt.id]; !ok || got != tt.want {
			t.Errorf("definition %s: got kind %v, want %v (ok=%v)", tt.id, got, tt.want, ok)
		}
	}

	// Test nested container closure
	if doc.defContainers["S1.C1"] != "S1" {
		t.Errorf("expected container of S1.C1 to be S1, got %q", doc.defContainers["S1.C1"])
	}
}

func TestFindDefinition(t *testing.T) {
	ws := NewWorkspace()
	uri := lsp.DocumentURI("file:///test.sruja")
	content := "model {\n  system S1\n}"
	ws.AddDocument(uri, content, 1)

	// Exact match
	u, r, ok := ws.FindDefinition("S1")
	if !ok || u != uri {
		t.Errorf("failed to find S1: ok=%v, u=%v", ok, u)
	}
	if r.Start.Line != 1 || r.Start.Character != 9 {
		t.Errorf("wrong range for S1: %+v", r)
	}

	// Fallback match (by last part)
	u2, r2, ok2 := ws.FindDefinition("other.S1")
	if !ok2 || u2 != uri {
		t.Errorf("failed to find other.S1 fallback: ok=%v, u=%v", ok2, u2)
	}
	if r2.Start.Line != 1 {
		t.Errorf("wrong range for fallback S1: %+v", r2)
	}

	// Not found
	_, _, ok3 := ws.FindDefinition("NonExistent")
	if ok3 {
		t.Error("expected not to find NonExistent")
	}
}
