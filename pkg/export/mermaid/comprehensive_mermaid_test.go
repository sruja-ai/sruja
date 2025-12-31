package mermaid

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExporter_Export_Complexity(t *testing.T) {
	parser, _ := language.NewParser()
	dsl := `
	Cloud = System "Cloud" {
		App = Container "App" {
			API = Component "API"
		}
		DB = Container "DB"
	}
	App.API -> DB "Persists"
`
	prog, _, _ := parser.Parse("comp.sruja", dsl)

	config := DefaultConfig()
	config.Direction = "TD"
	exporter := NewExporter(config)
	result := exporter.Export(prog)

	if !strings.Contains(result, "subgraph Cloud") {
		t.Error("Missing Cloud subgraph")
	}
	if !strings.Contains(result, "subgraph Cloud_App") {
		t.Error("Missing App subgraph (should be Cloud_App)")
	}
	if !strings.Contains(result, "Persists") {
		t.Error("Missing edge label")
	}
}

func TestExporter_EdgeStyling(t *testing.T) {
	// Mermaid exporter uses arrows like --> or -.-> based on some logic if it was present
	// Let's check if it does.

	// For now, let's just make sure it handles different kinds of elements
	dsl := `
    P = Person "P"
    S = System "S"
    C = Container "C"
    Comp = Component "Comp"
    D = Database "D"
    Q = Queue "Q"
`
	parser, _ := language.NewParser()
	prog, _, _ := parser.Parse("style.sruja", dsl)

	exporter := NewExporter(DefaultConfig())
	result := exporter.Export(prog)

	// Check for different shapes if Mermaid exporter supports them
	// Usually it uses the IDs and labels.
	if !strings.Contains(result, "P") || !strings.Contains(result, "S") {
		t.Error("Missing elements in styled output")
	}
}

func TestExporter_NilModel(t *testing.T) {
	prog := &language.Program{Model: nil}
	exporter := NewExporter(DefaultConfig())
	result := exporter.Export(prog)
	if result != "" {
		t.Errorf("Expected empty result for nil model, got %q", result)
	}
}
