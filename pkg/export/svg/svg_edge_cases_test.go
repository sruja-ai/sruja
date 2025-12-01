// pkg/export/svg/svg_edge_cases_test.go
package svg

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExport_EmptyArchitecture(t *testing.T) {
	arch := &language.Architecture{
		Name: "EmptyArch",
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	expected := []string{
		"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>",
		"<svg",
		"EmptyArch Architecture",
		"Interactive C4 Model",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't", exp)
		}
	}
}

func TestExport_System_NoDescription(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID:    "Sys1",
				Label: "System 1",
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	if !strings.Contains(output, `id="Sys1"`) {
		t.Error("Should export system without description")
	}
	if !strings.Contains(output, "System 1") {
		t.Error("Should include system label")
	}
}

func TestExport_Component_NoTechnology(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				Containers: []*language.Container{
					{
						ID: "Cont1",
						Components: []*language.Component{
							{
								ID:    "Comp1",
								Label: "Component 1",
							},
						},
					},
				},
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	if !strings.Contains(output, `id="Comp1"`) {
		t.Error("Should export component without technology")
	}
	if strings.Contains(output, "Technology Stack") {
		t.Error("Should not include technology summary when no technology is specified")
	}
}

func TestExport_Container_NoRequirements(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				Containers: []*language.Container{
					{
						ID:          "Cont1",
						Label:       "Container 1",
						Description: stringPtr("A container"),
					},
				},
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	if !strings.Contains(output, `id="Cont1"`) {
		t.Error("Should export container without requirements")
	}
	if !strings.Contains(output, "Container 1") {
		t.Error("Should include container label")
	}
}

func TestExport_MultipleSystems(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID:    "Sys1",
				Label: "System 1",
			},
			{
				ID:    "Sys2",
				Label: "System 2",
			},
			{
				ID:    "Sys3",
				Label: "System 3",
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	expected := []string{
		`id="Sys1"`,
		`id="Sys2"`,
		`id="Sys3"`,
		"System 1",
		"System 2",
		"System 3",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't", exp)
		}
	}
}

func TestExport_Relations(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Persons: []*language.Person{
			{ID: "User", Label: "User"},
		},
		Systems: []*language.System{
			{ID: "Sys1", Label: "System 1"},
			{ID: "Sys2", Label: "System 2"},
		},
		Relations: []*language.Relation{
			{
				From:  "User",
				To:    "Sys1",
				Verb:  stringPtr("Uses"),
				Label: stringPtr("HTTP"),
			},
			{
				From: "Sys1",
				To:   "Sys2",
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	// D2 generates arrows/connections for relations - check for path elements or connections
	if !strings.Contains(output, "<path") && !strings.Contains(output, "connection") && !strings.Contains(output, "edge") {
		t.Error("Should include connections/edges for relations")
	}
}

func TestExport_TechnologySummary_OnlyComponents(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				Containers: []*language.Container{
					{
						ID: "Cont1",
						Components: []*language.Component{
							{
								ID:         "Comp1",
								Label:      "Component 1",
								Technology: stringPtr("Go"),
							},
						},
					},
				},
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	expected := []string{
		"Technology Stack",
		"Go (Component: Component 1)",
		"technology_summary",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't", exp)
		}
	}
}

func TestExport_TechnologySummary_Deduplication(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				Containers: []*language.Container{
					{
						ID: "Cont1",
						Components: []*language.Component{
							{
								ID:         "Comp1",
								Label:      "Component 1",
								Technology: stringPtr("Go"),
							},
							{
								ID:         "Comp2",
								Label:      "Component 2",
								Technology: stringPtr("Go"),
							},
						},
					},
				},
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	// Count occurrences of "Go"
	count := strings.Count(output, "Go (Component:")
	if count != 1 {
		t.Errorf("Expected technology 'Go' to appear once in summary, got %d", count)
	}
}
