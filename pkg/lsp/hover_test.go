package lsp

import (
	"context"
	"strings"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestHover_Elements(t *testing.T) {
	server := NewServer()
	uri := lsp.DocumentURI("file:///test.sruja")
	text := `

System = kind "System"
Container = kind "Container"
Component = kind "Component"
Database = kind "DataStore"
Queue = kind "Queue"
Person = kind "Person"

S1 = System "System 1" {
	C1 = Container "Container 1" {
		Comp1 = Component "Component 1"
	}
	DB1 = Database "Database 1"
	Q1 = Queue "Queue 1"
}
P1 = Person "Person 1"
`

	server.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text},
	})

	tests := []struct {
		name      string
		line      int
		char      int
		wantType  string
		wantLabel string
	}{
		{"System", 9, 2, "System", "System 1"},           // S1
		{"Container", 10, 3, "Container", "Container 1"}, // C1
		{"Component", 11, 4, "Component", "Component 1"}, // Comp1
		{"DataStore", 13, 3, "DataStore", "Database 1"},  // DB1
		{"Queue", 14, 3, "Queue", "Queue 1"},             // Q1
		{"Person", 16, 2, "Person", "Person 1"},          // P1
		{"Unknown", 0, 0, "", ""},                        // whitespace - line 0 is empty
		{"Whitespace", 0, 0, "", ""},                     // Empty line
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			params := lsp.TextDocumentPositionParams{
				TextDocument: lsp.TextDocumentIdentifier{URI: uri},
				Position:     lsp.Position{Line: tt.line, Character: tt.char},
			}
			hover, err := server.Hover(context.Background(), params)
			if err != nil {
				t.Fatalf("Hover failed: %v", err)
			}

			if tt.wantType == "" {
				if hover != nil {
					t.Errorf("Expected nil hover, got %v", hover)
				}
			} else {
				if hover == nil {
					t.Fatal("Expected hover, got nil")
				}
				if len(hover.Contents) == 0 {
					t.Fatal("Expected hover contents")
				}
				content := hover.Contents[0].Value
				if !strings.Contains(content, "**"+tt.wantType+"**") {
					t.Errorf("Expected type %q in content %q", tt.wantType, content)
				}
				if !strings.Contains(content, tt.wantLabel) {
					t.Errorf("Expected label %q in content %q", tt.wantLabel, content)
				}
			}
		})
	}
}

func TestHover_EdgeCases(t *testing.T) {
	server := NewServer()
	uri := lsp.DocumentURI("file:///empty.sruja")

	// Empty document
	server.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: ""},
	})

	params := lsp.TextDocumentPositionParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: uri},
		Position:     lsp.Position{Line: 0, Character: 0},
	}
	hover, err := server.Hover(context.Background(), params)
	if err != nil {
		t.Fatalf("Hover failed: %v", err)
	}
	if hover != nil {
		t.Error("Expected nil hover for empty document")
	}

	// Invalid position (out of bounds)
	params.Position.Line = 100
	hover, err = server.Hover(context.Background(), params)
	if err != nil {
		t.Fatalf("Hover failed: %v", err)
	}
	if hover != nil {
		t.Error("Expected nil hover for invalid position")
	}
}
func TestHoverArrowRelation(t *testing.T) {
	s := NewServer()
	uri := lsp.DocumentURI("file:///hover.sruja")
	text := "system=kind \"System\" container=kind \"Container\"\n\nS = system \"System\"\nC = container \"Container\"\nS -> C \"uses\"\n"
	s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uri, Text: text}})
	h, err := s.Hover(context.Background(), lsp.TextDocumentPositionParams{TextDocument: lsp.TextDocumentIdentifier{URI: uri}, Position: lsp.Position{Line: 4, Character: 3}})
	if err != nil {
		t.Fatalf("Hover failed: %v", err)
	}
	if h == nil {
		t.Fatalf("Expected hover over arrow")
	}
}
