package svg

import (
	"strings"
	"testing"

	"gonum.org/v1/gonum/graph/simple"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExport_Themes(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test Arch",
		Systems: []*language.System{
			{ID: "S1", Label: "System 1"},
		},
	}

	themes := []struct {
		name  string
		theme *Theme
	}{
		{"Default", DefaultTheme()},
		{"Professional", ProfessionalTheme()},
		{"Minimal", MinimalTheme()},
		{"C4", C4Theme()},
	}

	for _, tt := range themes {
		t.Run(tt.name, func(t *testing.T) {
			exporter := NewExporter()
			exporter.Theme = tt.theme
			svg, err := exporter.Export(arch)
			if err != nil {
				t.Fatalf("Export failed: %v", err)
			}
			if !strings.Contains(svg, "<svg") {
				t.Error("Expected SVG output")
			}
		})
	}
}

func TestExport_Direction(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test Arch",
		Metadata: []*language.MetaEntry{
			{Key: "svg_direction", Value: strPtr("TB")},
		},
		Systems: []*language.System{
			{ID: "S1", Label: "System 1"},
		},
	}
	exporter := NewExporter()
	_, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}
	if exporter.Direction != "TB" {
		t.Errorf("Expected direction TB, got %s", exporter.Direction)
	}
}

func TestExportSystemContainer(t *testing.T) {
	sys := &language.System{
		ID:    "S1",
		Label: "System 1",
		Containers: []*language.Container{
			{ID: "C1", Label: "Container 1"},
			{ID: "C2", Label: "Container 2"},
		},
	}
	arch := &language.Architecture{
		Name:    "Test Arch",
		Systems: []*language.System{sys},
		Relations: []*language.Relation{
			{From: language.QualifiedIdent{Parts: []string{"S1", "C1"}}, To: language.QualifiedIdent{Parts: []string{"S1", "C2"}}, Label: strPtr("uses")},
		},
	}

	exporter := NewExporter()
	svg, err := exporter.ExportSystemContainer(arch, sys)
	if err != nil {
		t.Fatalf("ExportSystemContainer failed: %v", err)
	}
	if !strings.Contains(svg, "Container 1") {
		t.Error("Expected Container 1 in output")
	}
	if !strings.Contains(svg, "Container 2") {
		t.Error("Expected Container 2 in output")
	}
	if !strings.Contains(svg, "uses") {
		t.Error("Expected relation label 'uses' in output")
	}
}

func TestExportContainerComponent(t *testing.T) {
	comp := &language.Component{
		ID:    "Comp1",
		Label: "Component 1",
	}
	cont := &language.Container{
		ID:         "C1",
		Label:      "Container 1",
		Components: []*language.Component{comp},
	}
	sys := &language.System{
		ID:         "S1",
		Label:      "System 1",
		Containers: []*language.Container{cont},
	}
	arch := &language.Architecture{
		Name:    "Test Arch",
		Systems: []*language.System{sys},
	}

	exporter := NewExporter()
	svg, err := exporter.ExportContainerComponent(arch, sys, cont)
	if err != nil {
		t.Fatalf("ExportContainerComponent failed: %v", err)
	}
	if !strings.Contains(svg, "Component 1") {
		t.Error("Expected Component 1 in output")
	}
}

func TestHelperFunctions(t *testing.T) {
	// Test lightenColor
	c := lightenColor("#000000")
	if c == "" {
		t.Error("Expected lightened color")
	}

	// Test min
	if min(10, 5) != 5 {
		t.Error("min(10, 5) should be 5")
	}
	if min(5, 10) != 5 {
		t.Error("min(5, 10) should be 5")
	}

	// Test lastSegment
	if lastSegment("a.b.c") != "c" {
		t.Errorf("lastSegment('a.b.c') = %s, want c", lastSegment("a.b.c"))
	}
	if lastSegment("a") != "a" {
		t.Errorf("lastSegment('a') = %s, want a", lastSegment("a"))
	}

	// Test relationLabel
	rel := &language.Relation{Label: strPtr("uses")}
	if l := relationLabel(rel); l != "uses" {
		t.Errorf("relationLabel(rel) = %s, want uses", l)
	}
	relEmpty := &language.Relation{}
	if l := relationLabel(relEmpty); l != "" {
		t.Errorf("relationLabel(relEmpty) = %s, want empty string", l)
	}
}

func strPtr(s string) *string {
	return &s
}

func TestSVG_Internals(t *testing.T) {
	// Test resolveOverviewNode
	arch := &language.Architecture{
		Systems: []*language.System{
			{ID: "S1"},
			{ID: "S2"},
		},
		Persons: []*language.Person{
			{ID: "P1"},
		},
	}
	contToSys := map[string]string{
		"C1": "S1",
		"C2": "S2",
	}

	tests := []struct {
		endpoint string
		wantID   string
		wantOk   bool
	}{
		{"S1", "S1", true},
		{"P1", "P1", true},
		{"S1.C1", "S1", true}, // Container resolves to system
		{"S2.C2", "S2", true},
		{"Unknown", "Unknown", true},    // Unknown resolves to itself (external)
		{"S1.Unknown", "Unknown", true}, // Unknown part resolves to itself
	}

	for _, tt := range tests {
		gotID, gotOk := resolveOverviewNode(arch, contToSys, tt.endpoint)
		if gotID != tt.wantID || gotOk != tt.wantOk {
			t.Errorf("resolveOverviewNode(%s) = (%s, %v), want (%s, %v)", tt.endpoint, gotID, gotOk, tt.wantID, tt.wantOk)
		}
	}

	// Test longestPathLayers with cycle
	g := simple.NewDirectedGraph()
	n1 := g.NewNode()
	g.AddNode(n1)
	n2 := g.NewNode()
	g.AddNode(n2)
	g.SetEdge(g.NewEdge(n1, n2))
	g.SetEdge(g.NewEdge(n2, n1)) // Cycle

	layers := longestPathLayers(g)
	if len(layers) != 2 {
		t.Errorf("Expected 2 nodes in layers, got %d", len(layers))
	}

	// Test nameByID
	m := map[string]int64{"A": 1, "B": 2}
	if n := nameByID(m, 1); n != "A" {
		t.Errorf("nameByID(m, 1) = %s, want A", n)
	}
	if n := nameByID(m, 3); n != "" {
		t.Errorf("nameByID(m, 3) = %s, want empty string", n)
	}

	// Test kindOfTop
	if k := kindOfTop(arch, "S1"); k != "system" {
		t.Errorf("kindOfTop(S1) = %s, want system", k)
	}
	if k := kindOfTop(arch, "P1"); k != "person" {
		t.Errorf("kindOfTop(P1) = %s, want person", k)
	}
	if k := kindOfTop(arch, "Unknown"); k != "external" {
		t.Errorf("kindOfTop(Unknown) = %s, want external", k)
	}
}

func TestExport_Features(t *testing.T) {
	arch := &language.Architecture{
		Name: "Feature Arch",
		Style: map[string]string{
			"svg_direction": "LR",
		},
		Metadata: []*language.MetaEntry{
			{Key: "svg_direction", Value: strPtr("TB")}, // Should override style
		},
		Systems: []*language.System{
			{
				ID:         "S1",
				Label:      "System 1",
				DataStores: []*language.DataStore{{ID: "DS1"}},
				Queues:     []*language.Queue{{ID: "Q1"}},
			},
			{ID: "S2"},
		},
		Relations: []*language.Relation{
			{From: language.QualifiedIdent{Parts: []string{"S1"}}, To: language.QualifiedIdent{Parts: []string{"S2"}}},
			{From: language.QualifiedIdent{Parts: []string{"S1"}}, To: language.QualifiedIdent{Parts: []string{"External"}}}, // External node
		},
	}

	exporter := NewExporter()
	exporter.ShowGrid = true
	exporter.ShowLegend = true
	exporter.ShowTitle = true
	exporter.Theme = ProfessionalTheme()
	exporter.Theme.UseGradients = true
	exporter.Theme.UseShadows = true
	exporter.Theme.UseCurvedEdges = true

	svg, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	if !strings.Contains(svg, "Feature Arch") {
		t.Error("Expected title 'Feature Arch'")
	}
	if !strings.Contains(svg, "Legend") {
		t.Error("Expected Legend")
	}
	if !strings.Contains(svg, "stroke-dasharray") {
		t.Error("Expected dashed line for external node")
	}
	// Check for DataStore/Queue resolution (implicit via buildContainerSystemMap coverage)
}

func TestExportDeployment(t *testing.T) {
	dep := &language.DeploymentNode{
		Label: "Production",
		ContainerInstances: []*language.ContainerInstance{
			{ContainerID: "C1", Label: "Instance 1"},
		},
		Infrastructure: []*language.InfrastructureNode{
			{ID: "Infra1", Label: "Server 1"},
		},
	}
	arch := &language.Architecture{
		Name: "Dep Arch",
		Systems: []*language.System{
			{
				ID: "S1",
				Containers: []*language.Container{
					{ID: "C1", Label: "Container 1"},
				},
			},
		},
	}

	exporter := NewExporter()
	svg, err := exporter.ExportDeployment(arch, dep)
	if err != nil {
		t.Fatalf("ExportDeployment failed: %v", err)
	}
	if !strings.Contains(svg, "Production") {
		t.Error("Expected deployment label 'Production'")
	}
	if !strings.Contains(svg, "Instance 1") {
		t.Error("Expected instance label 'Instance 1'")
	}
	if !strings.Contains(svg, "Server 1") {
		t.Error("Expected infra label 'Server 1'")
	}
}

func TestExportContainerComponent_Relations(t *testing.T) {
	comp1 := &language.Component{ID: "Comp1", Label: "Component 1"}
	comp2 := &language.Component{ID: "Comp2", Label: "Component 2"}
	cont := &language.Container{
		ID:         "C1",
		Label:      "Container 1",
		Components: []*language.Component{comp1, comp2},
	}
	sys := &language.System{
		ID:         "S1",
		Label:      "System 1",
		Containers: []*language.Container{cont},
		Relations: []*language.Relation{
			{From: language.QualifiedIdent{Parts: []string{"Comp1"}}, To: language.QualifiedIdent{Parts: []string{"Comp2"}}, Label: strPtr("calls")},
		},
	}
	arch := &language.Architecture{
		Name:    "Test Arch",
		Systems: []*language.System{sys},
	}

	exporter := NewExporter()
	svg, err := exporter.ExportContainerComponent(arch, sys, cont)
	if err != nil {
		t.Fatalf("ExportContainerComponent failed: %v", err)
	}
	if !strings.Contains(svg, "calls") {
		t.Error("Expected relation label 'calls'")
	}
}

func TestExportAll(t *testing.T) {
	arch := &language.Architecture{
		Name:    "All Elements",
		Persons: []*language.Person{{ID: "P1", Label: "Person 1"}},
		Systems: []*language.System{
			{
				ID: "S1",
				Containers: []*language.Container{
					{
						ID: "C1",
						Components: []*language.Component{
							{ID: "Comp1", Label: "Component 1"},
						},
					},
				},
				DataStores: []*language.DataStore{{ID: "DS1"}},
				Queues:     []*language.Queue{{ID: "Q1"}},
			},
		},
		Relations: []*language.Relation{
			{From: language.QualifiedIdent{Parts: []string{"P1"}}, To: language.QualifiedIdent{Parts: []string{"S1"}}},
			{From: language.QualifiedIdent{Parts: []string{"S1"}}, To: language.QualifiedIdent{Parts: []string{"Unknown"}}},
		},
	}

	exporter := NewExporter()
	svg, err := exporter.ExportAll(arch)
	if err != nil {
		t.Fatalf("ExportAll failed: %v", err)
	}

	expected := []string{"Person 1", "S1", "C1", "Component 1", "DS1", "Q1", "Unknown"}
	for _, exp := range expected {
		if !strings.Contains(svg, exp) {
			t.Errorf("Expected %s in SVG", exp)
		}
	}
}

func TestStyleIconFor(t *testing.T) {
	arch := &language.Architecture{
		Systems: []*language.System{
			{
				ID: "S1",
				Containers: []*language.Container{
					{ID: "C1", Style: map[string]string{"icon": "server"}},
					{
						ID: "C2",
						Components: []*language.Component{
							{ID: "Comp1", Style: map[string]string{"icon": "box"}},
						},
					},
				},
				DataStores: []*language.DataStore{{ID: "DS1", Style: map[string]string{"icon": "db"}}},
				Queues:     []*language.Queue{{ID: "Q1", Style: map[string]string{"icon": "queue"}}},
			},
		},
	}

	tests := []struct {
		id   string
		want string
	}{
		{"C1", "server"},
		{"Comp1", "box"},
		{"DS1", "db"},
		{"Q1", "queue"},
		{"Unknown", ""},
	}

	for _, tt := range tests {
		if got := styleIconFor(arch, tt.id); got != tt.want {
			t.Errorf("styleIconFor(%s) = %s, want %s", tt.id, got, tt.want)
		}
	}
}

func TestExportSystemContainer_Complex(t *testing.T) {
	sys := &language.System{
		ID:    "S1",
		Label: "System 1",
		Containers: []*language.Container{
			{ID: "C1", Label: "Container 1"},
			{ID: "C2", Label: "Container 2"},
		},
		DataStores: []*language.DataStore{
			{ID: "DS1", Label: "Database"},
		},
		Queues: []*language.Queue{
			{ID: "Q1", Label: "Queue"},
		},
		Relations: []*language.Relation{
			{From: language.QualifiedIdent{Parts: []string{"C1"}}, To: language.QualifiedIdent{Parts: []string{"DS1"}}, Label: strPtr("reads")},
			{From: language.QualifiedIdent{Parts: []string{"C2"}}, To: language.QualifiedIdent{Parts: []string{"Q1"}}, Label: strPtr("writes")},
			{From: language.QualifiedIdent{Parts: []string{"C1"}}, To: language.QualifiedIdent{Parts: []string{"C2"}}, Label: strPtr("calls")},
		},
	}
	arch := &language.Architecture{
		Name:    "Complex System",
		Systems: []*language.System{sys},
	}

	exporter := NewExporter()
	svg, err := exporter.ExportSystemContainer(arch, sys)
	if err != nil {
		t.Fatalf("ExportSystemContainer failed: %v", err)
	}

	expected := []string{"Container 1", "Container 2", "Database", "Queue", "reads", "writes", "calls"}
	for _, exp := range expected {
		if !strings.Contains(svg, exp) {
			t.Errorf("Expected %s in SVG", exp)
		}
	}
}

func TestExportContainerComponent_Complex(t *testing.T) {
	comp1 := &language.Component{ID: "Comp1", Label: "Component 1"}
	comp2 := &language.Component{ID: "Comp2", Label: "Component 2"}
	comp3 := &language.Component{ID: "Comp3", Label: "Component 3"}
	cont := &language.Container{
		ID:         "C1",
		Label:      "Container 1",
		Components: []*language.Component{comp1, comp2, comp3},
	}
	sys := &language.System{
		ID:         "S1",
		Label:      "System 1",
		Containers: []*language.Container{cont},
		Relations: []*language.Relation{
			{From: language.QualifiedIdent{Parts: []string{"Comp1"}}, To: language.QualifiedIdent{Parts: []string{"Comp2"}}, Label: strPtr("calls")},
			{From: language.QualifiedIdent{Parts: []string{"Comp2"}}, To: language.QualifiedIdent{Parts: []string{"Comp3"}}, Label: strPtr("uses")},
			{From: language.QualifiedIdent{Parts: []string{"Comp3"}}, To: language.QualifiedIdent{Parts: []string{"Comp1"}}, Label: strPtr("notifies")},
		},
	}
	arch := &language.Architecture{
		Name:    "Complex Component",
		Systems: []*language.System{sys},
	}

	exporter := NewExporter()
	svg, err := exporter.ExportContainerComponent(arch, sys, cont)
	if err != nil {
		t.Fatalf("ExportContainerComponent failed: %v", err)
	}

	expected := []string{"Component 1", "Component 2", "Component 3", "calls", "uses", "notifies"}
	for _, exp := range expected {
		if !strings.Contains(svg, exp) {
			t.Errorf("Expected %s in SVG", exp)
		}
	}
}

func TestExportOverview_Complex(t *testing.T) {
	sys := &language.System{ID: "S1", Label: "System 1"}
	person := &language.Person{ID: "P1", Label: "User"}
	arch := &language.Architecture{
		Name:    "Overview System",
		Systems: []*language.System{sys},
		Persons: []*language.Person{person},
		Relations: []*language.Relation{
			{From: language.QualifiedIdent{Parts: []string{"P1"}}, To: language.QualifiedIdent{Parts: []string{"S1"}}, Label: strPtr("uses")},
		},
		Metadata: []*language.MetaEntry{
			{Key: "svg_direction", Value: strPtr("TB")},
		},
	}

	exporter := NewExporter()
	exporter.ShowGrid = true
	exporter.ShowLegend = true
	exporter.ShowTitle = true

	svg, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	expected := []string{
		"System 1", "User", "uses",
		"Legend", "Person", "System", // Legend items
	}
	for _, exp := range expected {
		if !strings.Contains(svg, exp) {
			t.Errorf("Expected %s in SVG", exp)
		}
	}
}
