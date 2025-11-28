package d2

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
		"direction: right",
		"Sys1: \"System 1\\n\\nA test system\" {",
		"shape: package",
		"tooltip: \"A test system\"",
		"}",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}

func TestExport_ContainerAndRelations(t *testing.T) {
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
					},
				},
			},
			{
				ID:    "Sys2",
				Label: "System 2",
			},
		},
		Relations: []*language.Relation{
			{
				From:  "Sys1.Cont1",
				To:    "Sys2",
				Verb:  stringPtr("Uses"),
				Label: stringPtr("HTTP"),
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	expected := []string{
		"Sys1: \"System 1\" {",
		"Cont1: \"Container 1\" {",
		"Sys2: \"System 2\" {",
		"Sys1.Cont1 -> Sys2: \"Uses: HTTP\"",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}

func TestExport_FullFeatures(t *testing.T) {
	arch := &language.Architecture{
		Name: "FullArch",
		Persons: []*language.Person{
			{ID: "User", Label: "User"},
		},
		Systems: []*language.System{
			{
				ID: "Sys1",
				DataStores: []*language.DataStore{
					{ID: "DB", Label: "Database"},
				},
				Queues: []*language.Queue{
					{ID: "Q", Label: "Queue"},
				},
			},
		},
		DeploymentNodes: []*language.DeploymentNode{
			{
				ID:    "Prod",
				Label: "Production",
				Children: []*language.DeploymentNode{
					{
						ID:    "Server",
						Label: "Server",
						ContainerInstances: []*language.ContainerInstance{
							{ContainerID: "Sys1.DB"},
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
		"User: \"User\" {",
		"shape: person",
		"DB: \"Database\" {",
		"shape: cylinder",
		"Q: \"Queue\" {",
		"shape: queue",
		"Prod: \"Production\" {",
		"Server: \"Server\" {",
		"Sys1.DB_Instance: {",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}

func TestExport_Component(t *testing.T) {
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
								ID:          "Comp1",
								Label:       "Component 1",
								Description: stringPtr("A test component"),
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
		"Comp1: \"Component 1\\n\\nA test component\" {",
		"shape: class",
		"tooltip: \"A test component\"",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}

func TestExport_Component_NoDescription(t *testing.T) {
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

	if !strings.Contains(output, "Comp1: \"Component 1\"") {
		t.Error("Should export component without description")
	}
	if strings.Contains(output, "tooltip") {
		t.Error("Should not include tooltip when description is nil")
	}
}

func TestExport_Requirements(t *testing.T) {
	arch := &language.Architecture{
		Name: "ReqArch",
		Requirements: []*language.Requirement{
			{ID: "R1", Type: "functional", Description: "Top level req"},
		},
		Systems: []*language.System{
			{
				ID: "Sys1",
				Requirements: []*language.Requirement{
					{ID: "R2", Type: "security", Description: "System level req"},
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
		"layers: {",
		"\"Requirements\"",
		"}",
		"\"Requirements\": {",
		"R1: \"functional: Top level req\" {",
		"shape: page",
		"Sys1.R2: \"security: System level req\" {",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't.\nOutput:\n%s", exp, output)
		}
	}
}

func stringPtr(s string) *string {
	return &s
}

func TestExport_Scenario(t *testing.T) {
	dsl := `
architecture "Scenario Test" {
	system Sys1 "System 1"
	person User "User"

	scenario "Login Flow" {
		User -> Sys1 "Credentials"
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	program.Architecture.PostProcess()

	exporter := NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	expected := []string{
		"layers: {",
		"\"Login Flow\"",
		"\"Architecture\"",
		"}",
		"\"Login Flow\": {",
		"_Title: \"Login Flow\" {",
		"shape: text",
		"User -> Sys1: \"1. Credentials\"",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain:\n%s\nBut got:\n%s", exp, output)
		}
	}
}

func TestExport_Scenario_WithIDAndDescription(t *testing.T) {
	dsl := `
architecture "Scenario Test" {
	system Sys1 "System 1"
	person User "User"

	scenario AuthFlow "Authentication" "Handles OAuth2" {
		User -> Sys1 "Credentials"
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	program.Architecture.PostProcess()

	exporter := NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	expected := []string{
		"layers: {",
		"\"Authentication\"",
		"\"Architecture\"",
		"}",
		"\"Authentication\": {",
		"_Title: \"Authentication\" {",
		"shape: text",
		"User -> Sys1: \"1. Credentials\"",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain:\n%s\nBut got:\n%s", exp, output)
		}
	}
}
