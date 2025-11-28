// pkg/export/d2/d2_edge_cases_test.go
package d2

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExport_Container_NoDescription(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				Containers: []*language.Container{
					{
						ID:    "Cont1",
						Label: "Container 1",
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

	if !strings.Contains(output, "Cont1: \"Container 1\"") {
		t.Error("Should export container without description")
	}
	if strings.Contains(output, "tooltip") {
		t.Error("Should not include tooltip when description is nil")
	}
}

func TestExport_Container_WithRelations(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				Containers: []*language.Container{
					{
						ID: "Cont1",
						Relations: []*language.Relation{
							{
								From: "Cont1",
								To:   "Cont2",
							},
						},
					},
					{
						ID:    "Cont2",
						Label: "Container 2",
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

	if !strings.Contains(output, "Cont1 -> Cont2") {
		t.Error("Should export container relations")
	}
}

func TestExport_Relation_VerbOnly(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{ID: "Sys1", Label: "System 1"},
			{ID: "Sys2", Label: "System 2"},
		},
		Relations: []*language.Relation{
			{
				From: "Sys1",
				To:   "Sys2",
				Verb: stringPtr("Uses"),
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	if !strings.Contains(output, "Sys1 -> Sys2: \"Uses\"") {
		t.Error("Should export relation with verb only")
	}
}

func TestExport_Relation_LabelOnly(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{ID: "Sys1", Label: "System 1"},
			{ID: "Sys2", Label: "System 2"},
		},
		Relations: []*language.Relation{
			{
				From:  "Sys1",
				To:    "Sys2",
				Label: stringPtr("HTTP"),
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	if !strings.Contains(output, "Sys1 -> Sys2: \"HTTP\"") {
		t.Error("Should export relation with label only")
	}
}

func TestExport_Relation_NoVerbNoLabel(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{ID: "Sys1", Label: "System 1"},
			{ID: "Sys2", Label: "System 2"},
		},
		Relations: []*language.Relation{
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

	if !strings.Contains(output, "Sys1 -> Sys2") {
		t.Error("Should export relation without verb or label")
	}
	if strings.Contains(output, "Sys1 -> Sys2:") {
		t.Error("Should not include colon when there's no label")
	}
}

func TestExport_DeploymentNode_WithInfrastructure(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		DeploymentNodes: []*language.DeploymentNode{
			{
				ID:    "Prod",
				Label: "Production",
				Infrastructure: []*language.InfrastructureNode{
					{
						ID:          "LB",
						Label:       "Load Balancer",
						Description: stringPtr("Main LB"),
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

	if !strings.Contains(output, "LB: \"Load Balancer\\n\\nMain LB\"") {
		t.Error("Should export infrastructure nodes")
	}
	if !strings.Contains(output, "tooltip: \"Main LB\"") {
		t.Error("Should include infrastructure description")
	}
}

func TestExport_DeploymentNode_WithContainerInstance(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				Containers: []*language.Container{
					{ID: "Cont1", Label: "Container 1"},
				},
			},
		},
		DeploymentNodes: []*language.DeploymentNode{
			{
				ID:    "Prod",
				Label: "Production",
				ContainerInstances: []*language.ContainerInstance{
					{ContainerID: "Sys1.Cont1"},
				},
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	if !strings.Contains(output, "Sys1.Cont1_Instance") {
		t.Error("Should export container instances")
	}
}

func TestExport_DeploymentNode_WithInstanceID(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
				Containers: []*language.Container{
					{ID: "Cont1", Label: "Container 1"},
				},
			},
		},
		DeploymentNodes: []*language.DeploymentNode{
			{
				ID:    "Prod",
				Label: "Production",
				ContainerInstances: []*language.ContainerInstance{
					{
						ContainerID: "Sys1.Cont1",
						InstanceID:  stringPtr("1"),
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

	if !strings.Contains(output, "Sys1.Cont1_1") {
		t.Error("Should use instance ID when provided")
	}
}

func TestExport_DeploymentNode_NestedChildren(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		DeploymentNodes: []*language.DeploymentNode{
			{
				ID:    "Prod",
				Label: "Production",
				Children: []*language.DeploymentNode{
					{
						ID:    "Server",
						Label: "Server",
						Children: []*language.DeploymentNode{
							{
								ID:    "VM",
								Label: "VM",
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

	if !strings.Contains(output, "Server: \"Server\"") {
		t.Error("Should export nested deployment nodes")
	}
	if !strings.Contains(output, "VM: \"VM\"") {
		t.Error("Should export deeply nested deployment nodes")
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

	if !strings.Contains(output, "Sys1: \"System 1\"") {
		t.Error("Should export system without description")
	}
	if strings.Contains(output, "tooltip") {
		t.Error("Should not include tooltip when description is nil")
	}
}
