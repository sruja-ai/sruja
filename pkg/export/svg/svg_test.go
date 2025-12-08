package svg

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExport_Basic(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test System",
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

	if !strings.Contains(output, "<svg") {
		t.Error("Expected SVG output")
	}
	if !strings.Contains(output, "Test System") {
		t.Error("Expected architecture name in output")
	}
	if !strings.Contains(output, "System 1") {
		t.Error("Expected system label in output")
	}
}

func TestExport_WithContainer(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test System",
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
		},
	}

	exporter := NewExporter()
	// Test ExportSystemContainer
	sys := arch.Systems[0]
	output, err := exporter.ExportSystemContainer(arch, sys)
	if err != nil {
		t.Fatalf("ExportSystemContainer failed: %v", err)
	}

	if !strings.Contains(output, "<svg") {
		t.Error("Expected SVG output")
	}
	if !strings.Contains(output, "Container 1") {
		t.Error("Expected container label in output")
	}
}
