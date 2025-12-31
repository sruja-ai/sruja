package dot_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/dot"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExporter_Export_Comprehensive(t *testing.T) {
	dsl := `
	System = kind "System"
	Container = kind "Container"
	Database = kind "Database"
	Person = kind "Person"

	user = Person "User"
	
	Cloud = System "Cloud" {
		web = Container "Web Application" {
			technology "React"
		}
		db = Database "Database" {
			technology "PostgreSQL"
		}
		web -> db "Writes"
	}
	
	user -> Cloud.web "Uses"
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	prog, _, err := parser.Parse("comprehensive.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	// Test L1 view (Context)
	config1 := dot.DefaultConfig()
	config1.ViewLevel = 1
	exporter1 := dot.NewExporter(config1)
	result1 := exporter1.Export(prog)

	if !strings.Contains(result1.DOT, "User") {
		t.Error("L1: Missing User")
	}
	if !strings.Contains(result1.DOT, "Cloud") {
		t.Error("L1: Missing Cloud")
	}
	if strings.Contains(result1.DOT, "web") {
		t.Error("L1: Should not contain nested web")
	}

	// Test L2 view (Container)
	config2 := dot.DefaultConfig()
	config2.ViewLevel = 2
	exporter2 := dot.NewExporter(config2)
	result2 := exporter2.Export(prog)

	if !strings.Contains(result2.DOT, "subgraph \"cluster_Cloud\"") {
		t.Error("L2: Missing cluster for Cloud")
	}
	if !strings.Contains(result2.DOT, "web") {
		t.Error("L2: Missing nested web")
	}
	if !strings.Contains(result2.DOT, "db") {
		t.Error("L2: Missing nested db")
	}

	// Test focus view
	config3 := dot.DefaultConfig()
	config3.ViewLevel = 2
	config3.FocusNodeID = "Cloud.web"
	exporter3 := dot.NewExporter(config3)
	result3 := exporter3.Export(prog)

	if result3 == nil || result3.DOT == "" {
		t.Fatal("Focus: Missing output")
	}
}

func TestExporter_EdgeAttributes(t *testing.T) {
	elements := []*dot.Element{
		{ID: "A", Title: "Node A"},
		{ID: "B", Title: "Node B"},
	}

	relations := []*dot.Relation{
		{From: "A", To: "B", Label: "Uses"},
	}

	constraints := dot.LayoutConstraints{
		Global: dot.GlobalConstraints{
			RankDir: "LR",
			NodeSep: 100,
			RankSep: 200,
			Splines: "ortho",
			Overlap: "false",
		},
		Edges: []dot.EdgeConstraint{
			{
				From: "A",
				To:   "B",
				Label: dot.EdgeLabel{
					Text:     "Labeled Edge",
					Distance: 2.0,
					Angle:    45.0,
					Position: 0.7,
				},
				Weight:        10,
				MinLen:        2,
				AffectsLayout: true,
			},
		},
		Ranks: []dot.RankConstraint{
			{Type: "same", NodeIDs: []string{"A", "B"}},
		},
	}

	output := dot.GenerateDOTFromConstraints(elements, relations, constraints)

	if !strings.Contains(output, "rankdir=\"LR\"") {
		t.Error("Missing rankdir LR")
	}
	if !strings.Contains(output, "splines=ortho") {
		t.Error("Missing splines ortho")
	}
	if !strings.Contains(output, "label=\"Labeled Edge\"") {
		t.Error("Missing edge label")
	}
	if !strings.Contains(output, "labeldistance=2.00") {
		t.Error("Missing labeldistance")
	}
	if !strings.Contains(output, "labelangle=45.00") {
		t.Error("Missing labelangle")
	}
	if !strings.Contains(output, "labelpos=0.70") {
		t.Error("Missing labelpos")
	}
	if !strings.Contains(output, "weight=10") {
		t.Error("Missing weight")
	}
	if !strings.Contains(output, "minlen=2") {
		t.Error("Missing minlen")
	}
}

func TestExporter_RankOrderingConstraints(t *testing.T) {
	constraints := dot.LayoutConstraints{
		Ranks: []dot.RankConstraint{
			{Type: "min", NodeIDs: []string{"Top1", "Top2"}},
			{Type: "max", NodeIDs: []string{"Bottom1", "Bottom2"}},
		},
	}

	output := dot.GenerateDOTFromConstraints([]*dot.Element{}, []*dot.Relation{}, constraints)

	if !strings.Contains(output, "style=invis") {
		t.Error("Missing invisible edges for rank ordering")
	}
	if !strings.Contains(output, "weight=1000") {
		t.Error("Missing high weight for rank alignment")
	}
}

func TestExporter_EmptyConstraints(t *testing.T) {
	output := dot.GenerateDOTFromConstraints([]*dot.Element{}, []*dot.Relation{}, dot.LayoutConstraints{})
	if !strings.Contains(output, "digraph G") {
		t.Error("Missing digraph header")
	}
}

func TestExporter_groupByParent_EdgeCases(t *testing.T) {
	elements := []*dot.Element{
		{ID: "Root", ParentID: ""},
		{ID: "Child", ParentID: "Root"},
	}
	output := dot.GenerateDOTFromConstraints(elements, []*dot.Relation{}, dot.LayoutConstraints{})
	if !strings.Contains(output, "subgraph \"cluster_Root\"") {
		t.Error("Missing cluster for Root")
	}
}

func TestExporter_Export_L3Focus(t *testing.T) {
	dsl := `
	system = kind "System"
	container = kind "Container"
	component = kind "Component"
	person = kind "Person"

	S = system "S" {
		C = container "C" {
			C1 = component "C1"
		}
	}
	P = person "P"
	P -> S.C.C1
`
	parser, _ := language.NewParser()
	prog, _, _ := parser.Parse("l3.sruja", dsl)

	config := dot.DefaultConfig()
	config.ViewLevel = 3
	config.FocusNodeID = "S.C"
	exporter := dot.NewExporter(config)
	result := exporter.Export(prog)

	if !strings.Contains(result.DOT, "C1") {
		t.Error("L3: Missing component C1")
	}
	// Verify external person P is present (projected)
	if !strings.Contains(result.DOT, "P") {
		t.Error("L3: Missing person P")
	}
}
