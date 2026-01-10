package dot_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/dot"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExporter_Export_BasicModel(t *testing.T) {
	dsl := `
	System = kind "System"
	Container = kind "Container"
	Database = kind "Database"

	service = System "Service" {
		api = Container "API"
		db = Database "Database"
	}

	service.api -> service.db "reads data"
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
		Model: &language.Model{},
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
	if config.NodeSep != dot.DefaultNodeSep {
		t.Errorf("Expected NodeSep=%d, got %d", dot.DefaultNodeSep, config.NodeSep)
	}
	if config.RankSep != dot.DefaultRankSep {
		t.Errorf("Expected RankSep=%d, got %d", dot.DefaultRankSep, config.RankSep)
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
	system = kind "System"
	sys = system "My System"
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
	if !strings.Contains(dot, "height=2.00") {
		t.Errorf("Expected height=2.00 for node 'sys', got DOT:\n%s", dot)
	}
}

func TestExporter_GlobalL2_HidesComponents(t *testing.T) {
	dsl := `
	System = kind "System"
	Container = kind "Container"
	Component = kind "Component"

	sys = System "My System" {
		cont1 = Container "Container 1" {
			comp1 = Component "Component 1"
		}
		cont2 = Container "Container 2" {
			comp2 = Component "Component 2"
		}
	}

	sys.cont1.comp1 -> sys.cont2.comp2 "uses"
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
	config.ViewLevel = 2 // L2 Global
	exporter := dot.NewExporter(config)
	result := exporter.Export(prog)

	if result == nil || result.DOT == "" {
		t.Fatal("Expected non-empty DOT output")
	}

	dot := result.DOT

	// 1. Verify Containers are present
	if !strings.Contains(dot, "\"sys.cont1\"") {
		t.Error("Missing Container 1")
	}
	if !strings.Contains(dot, "\"sys.cont2\"") {
		t.Error("Missing Container 2")
	}

	// 2. Verify Components are HIDDEN (Strict L2)
	if strings.Contains(dot, "\"sys.cont1.comp1\"") {
		t.Error("Component 1 should NOT be visible in L2")
	}
	if strings.Contains(dot, "\"sys.cont2.comp2\"") {
		t.Error("Component 2 should NOT be visible in L2")
	}

	// 3. Verify Edge Projection (Component -> Component becomes Container -> Container)
	// Expected: "sys.cont1" -> "sys.cont2"
	if !strings.Contains(dot, "\"sys.cont1\" -> \"sys.cont2\"") {
		t.Error("Missing projected edge between containers")
	}
}

func TestExporter_GlobalL2_AggregatesEdges(t *testing.T) {
	dsl := `
	System = kind "System"
	Container = kind "Container"
	Component = kind "Component"

	sys = System "My System" {
		cont1 = Container "Container 1" {
			comp1a = Component "Component 1A"
			comp1b = Component "Component 1B"
		}
		cont2 = Container "Container 2" {
			comp2a = Component "Component 2A"
			comp2b = Component "Component 2B"
		}
	}

	sys.cont1.comp1a -> sys.cont2.comp2a "login"
	sys.cont1.comp1b -> sys.cont2.comp2b "logout"
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
	config.ViewLevel = 2 // L2 Global
	exporter := dot.NewExporter(config)
	result := exporter.Export(prog)

	if result == nil || result.DOT == "" {
		t.Fatal("Expected non-empty DOT output")
	}

	dot := result.DOT

	// 1. Verify ONLY ONE edge exists between cont1 and cont2
	edgeStr := "\"sys.cont1\" -> \"sys.cont2\""
	count := strings.Count(dot, edgeStr)
	if count != 1 {
		t.Errorf("Expected exactly 1 aggregated edge between containers, found %d", count)
	}

	// 2. Verify aggregated label
	// Should contain both "login" and "logout" joined by comma
	if !strings.Contains(dot, "login, logout") && !strings.Contains(dot, "logout, login") {
		t.Errorf("Expected aggregated label to contain joined interactions, got DOT:\n%s", dot)
	}
}

func TestExporter_ParentNotRenderedAsNode(t *testing.T) {
	dsl := `
	System = kind "System"
	Container = kind "Container"

	sys = System "My System" {
		cont1 = Container "Container 1"
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
	config.ViewLevel = 2
	exporter := dot.NewExporter(config)
	result := exporter.Export(prog)

	dot := result.DOT

	// The system "sys" should be a subgraph/cluster
	if !strings.Contains(dot, "subgraph \"cluster_sys\"") {
		t.Error("System should be rendered as a cluster")
	}

	// The system "sys" should NOT be rendered as a standalone node definition
	// i.e., "sys" [ ... ];
	// We search for exact node definition pattern
	if strings.Contains(dot, "\"sys\" [\n") {
		t.Error("System 'sys' should NOT be rendered as a standalone node when it is a cluster")
	}
}
