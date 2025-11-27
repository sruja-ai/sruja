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
		"Sys1: \"System 1\" {",
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
		DynamicViews: []*language.DynamicView{
			{
				Title: "Login",
				Steps: []*language.DynamicViewStep{
					{From: "User", To: "Sys1", Description: stringPtr("Logins")},
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
		"scenario: \"Login\" {",
		"User -> Sys1: \"1. Logins\"",
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
