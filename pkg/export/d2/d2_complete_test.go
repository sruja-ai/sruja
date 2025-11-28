// pkg/export/d2/d2_complete_test.go
package d2

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExport_System_WithDescription(t *testing.T) {
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

	if !strings.Contains(output, "tooltip: \"A test system\"") {
		t.Error("Should include tooltip when system has description")
	}
}

func TestExport_Container_WithDescription(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				Containers: []*language.Container{
					{
						ID:          "Cont1",
						Label:       "Container 1",
						Description: stringPtr("A test container"),
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

	if !strings.Contains(output, "tooltip: \"A test container\"") {
		t.Error("Should include tooltip when container has description")
	}
}

func TestExport_DataStore_WithDescription(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				DataStores: []*language.DataStore{
					{
						ID:          "DB",
						Label:       "Database",
						Description: stringPtr("Main database"),
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

	if !strings.Contains(output, "tooltip: \"Main database\"") {
		t.Error("Should include tooltip when datastore has description")
	}
}

func TestExport_Queue_WithDescription(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				Queues: []*language.Queue{
					{
						ID:          "Q",
						Label:       "Queue",
						Description: stringPtr("Event queue"),
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

	if !strings.Contains(output, "tooltip: \"Event queue\"") {
		t.Error("Should include tooltip when queue has description")
	}
}

func TestExport_DeploymentNode_NoDescription(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		DeploymentNodes: []*language.DeploymentNode{
			{
				ID:    "Prod",
				Label: "Production",
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	if !strings.Contains(output, "Prod: \"Production\"") {
		t.Error("Should export deployment node without description")
	}
	if strings.Contains(output, "tooltip") {
		t.Error("Should not include tooltip when description is nil")
	}
}
