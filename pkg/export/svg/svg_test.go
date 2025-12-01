// pkg/export/svg/svg_test.go
package svg

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExport_SimpleSystem(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID:          "Sys1",
				Label:       "System 1",
				Description: stringPtr("A test system"),
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	expected := []string{
		"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>",
		"<svg",
		"xmlns=\"http://www.w3.org/2000/svg\"",
		"TestArch Architecture",
		`id="Sys1"`, // Element ID from D2 processing
		"System 1",
		"data-content-id",
		"<script",
		"level1", // Level 1 view
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}

func TestExport_ContainerAndComponents(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID:    "Sys1",
				Label: "System 1",
				Containers: []*language.Container{
					{
						ID:    "Cont1",
						Label: "Container 1",
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
		`id="Cont1"`, // Element ID from D2 processing
		"Container 1",
		`id="Comp1"`, // Component ID
		"Component 1",
		"Technology",
		"Go",
		"view-Sys1",  // Level 2 view
		"view-Cont1", // Level 3 view
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}

func TestExport_Persons(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Persons: []*language.Person{
			{
				ID:          "User",
				Label:       "User",
				Description: stringPtr("End user"),
			},
		},
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

	expected := []string{
		"User",            // Person label should be present
		"data-content-id", // Interactive elements have this
		"level1",          // Level 1 view should exist
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}

func TestExport_DataStoresAndQueues(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				DataStores: []*language.DataStore{
					{
						ID:         "DB",
						Label:      "Database",
						Technology: stringPtr("PostgreSQL"),
					},
				},
				Queues: []*language.Queue{
					{
						ID:         "MQ",
						Label:      "Message Queue",
						Technology: stringPtr("RabbitMQ"),
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
		`id="DB"`, // Element ID from D2 processing
		"Database",
		"PostgreSQL",
		"RabbitMQ",
		"Technology Stack",
		"view-Sys1", // Level 2 view should exist for system with containers/datastores
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}

func TestExport_Requirements(t *testing.T) {
	arch := &language.Architecture{
		Name: "ReqArch",
		Requirements: []*language.Requirement{
			{
				ID:          "R1",
				Type:        stringPtr("functional"),
				Description: stringPtr("Req 1"),
			},
		},
		Systems: []*language.System{
			{
				ID: "Sys1",
				Requirements: []*language.Requirement{
					{
						ID:          "R2",
						Type:        stringPtr("security"),
						Description: stringPtr("Req 2"),
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
		"All Requirements",
		"R1 (functional): Req 1",
		"Sys1.R2 (security): Req 2",
		"requirements_summary",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}

func TestExport_ADRs(t *testing.T) {
	arch := &language.Architecture{
		Name: "ADRArch",
		ADRs: []*language.ADR{
			{
				ID:    "ADR001",
				Title: stringPtr("Use Microservices"),
				Body: &language.ADRBody{
					Status:   stringPtr("Accepted"),
					Decision: stringPtr("We will use microservices architecture"),
				},
			},
		},
		Systems: []*language.System{
			{
				ID: "Sys1",
				ADRs: []*language.ADR{
					{
						ID:    "ADR002",
						Title: stringPtr("Use REST API"),
						Body: &language.ADRBody{
							Status: stringPtr("Proposed"),
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
		"All Architectural Decision Records",
		"ADR001",
		"Use Microservices",
		"Status: Accepted",
		"Decision: We will use microservices architecture",
		"adrs_summary",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}

func TestExport_C4Levels(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID:    "Sys1",
				Label: "System 1",
				Containers: []*language.Container{
					{
						ID:    "Cont1",
						Label: "Container 1",
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

	expected := []string{
		"Level 1: System Context",
		"Level 2: Containers",
		"Level 3: Components",
		"switchLevel",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}
