package dot_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/dot"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExporter_Export_BasicModel(t *testing.T) {
	dsl := `
specification {
	element system
	element container
	element database
}

model {
	service = system "Service" {
		api = container "API"
		db = database "Database"
	}

	service.api -> service.db "reads data"
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	config := dot.DefaultConfig()
	config.ViewLevel = 2 // L2 to show containers
	exporter := dot.NewExporter(config)
	result := exporter.Export(prog)

	if result == nil || result.DOT == "" {
		t.Fatal("Expected non-empty DOT output")
	}

	dot := result.DOT

	// Verify digraph structure
	if !strings.Contains(dot, "digraph G {") {
		t.Error("Missing digraph header")
	}

	// Verify nodes are present
	if !strings.Contains(dot, "\"service\"") {
		t.Error("Missing service node")
	}
	if !strings.Contains(dot, "\"service.api\"") {
		t.Error("Missing api node")
	}
	if !strings.Contains(dot, "\"service.db\"") {
		t.Error("Missing db node")
	}

	// Verify edge with label
	if !strings.Contains(dot, "\"service.api\" -> \"service.db\"") {
		t.Error("Missing api->db edge")
	}
	if !strings.Contains(dot, "reads data") {
		t.Error("Missing edge label")
	}

	// Verify rank constraints are present
	if !strings.Contains(dot, "rank=same") {
		t.Error("Missing rank constraints")
	}
}

func TestExporter_Export_EmptyModel(t *testing.T) {
	prog := &language.Program{
		Model: &language.ModelBlock{},
	}

	exporter := dot.NewExporter(dot.DefaultConfig())
	result := exporter.Export(prog)

	if result != nil && result.DOT != "" {
		t.Errorf("Expected empty output for empty model, got: %s", result.DOT)
	}
}

func TestExporter_DefaultConfig(t *testing.T) {
	config := dot.DefaultConfig()

	if config.RankDir != "TB" {
		t.Errorf("Expected RankDir=TB, got %s", config.RankDir)
	}
	if config.NodeSep != 150 {
		t.Errorf("Expected NodeSep=150, got %d", config.NodeSep)
	}
	if config.RankSep != 180 {
		t.Errorf("Expected RankSep=180, got %d", config.RankSep)
	}
	if !config.UseRankConstraints {
		t.Error("Expected UseRankConstraints=true")
	}
	if !config.UseEdgeWeights {
		t.Error("Expected UseEdgeWeights=true")
	}
}

func TestExporter_NodeSizes(t *testing.T) {
	dsl := `
specification {
	element system
}
model {
	sys = system "My System"
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	config := dot.DefaultConfig()
	config.NodeSizes = map[string]struct{ Width, Height float64 }{
		"sys": {Width: 288.0, Height: 144.0}, // 4.0 x 2.0 inches
	}
	exporter := dot.NewExporter(config)
	result := exporter.Export(prog)

	if result == nil || result.DOT == "" {
		t.Fatal("Expected non-empty DOT output")
	}

	dot := result.DOT

	// Check for explicit width/height in inches (288/72 = 4.00, 144/72 = 2.00)
	if !strings.Contains(dot, "width=4.00") {
		t.Errorf("Expected width=4.00 for node 'sys', got DOT:\n%s", dot)
	}
	if !strings.Contains(dot, "height=2.00") {
		t.Errorf("Expected height=2.00 for node 'sys', got DOT:\n%s", dot)
	}
}
