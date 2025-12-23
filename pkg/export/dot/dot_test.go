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

	exporter := dot.NewExporter(dot.DefaultConfig())
	output := exporter.Export(prog)

	if output == "" {
		t.Fatal("Expected non-empty DOT output")
	}

	// Verify digraph structure
	if !strings.Contains(output, "digraph G {") {
		t.Error("Missing digraph header")
	}

	// Verify nodes are present
	if !strings.Contains(output, "\"service\"") {
		t.Error("Missing service node")
	}
	if !strings.Contains(output, "\"service.api\"") {
		t.Error("Missing api node")
	}
	if !strings.Contains(output, "\"service.db\"") {
		t.Error("Missing db node")
	}

	// Verify edge with label
	if !strings.Contains(output, "\"service.api\" -> \"service.db\"") {
		t.Error("Missing api->db edge")
	}
	if !strings.Contains(output, "reads data") {
		t.Error("Missing edge label")
	}

	// Verify rank constraints are present
	if !strings.Contains(output, "rank=same") {
		t.Error("Missing rank constraints")
	}
}

func TestExporter_Export_EmptyModel(t *testing.T) {
	prog := &language.Program{
		Model: &language.ModelBlock{},
	}

	exporter := dot.NewExporter(dot.DefaultConfig())
	output := exporter.Export(prog)

	if output != "" {
		t.Errorf("Expected empty output for empty model, got: %s", output)
	}
}

func TestExporter_DefaultConfig(t *testing.T) {
	config := dot.DefaultConfig()

	if config.RankDir != "TB" {
		t.Errorf("Expected RankDir=TB, got %s", config.RankDir)
	}
	if config.NodeSep != 80 {
		t.Errorf("Expected NodeSep=80, got %d", config.NodeSep)
	}
	if config.RankSep != 90 {
		t.Errorf("Expected RankSep=90, got %d", config.RankSep)
	}
	if !config.UseRankConstraints {
		t.Error("Expected UseRankConstraints=true")
	}
	if !config.UseEdgeWeights {
		t.Error("Expected UseEdgeWeights=true")
	}
}
