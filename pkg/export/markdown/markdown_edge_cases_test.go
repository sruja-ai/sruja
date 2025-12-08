package markdown_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/markdown"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestMarkdownExporter_EdgeCases(t *testing.T) {
	t.Run("Nil Architecture", func(t *testing.T) {
		exporter := markdown.NewExporter()
		output, err := exporter.Export(nil)
		if err == nil {
			t.Error("Expected error for nil architecture")
		}
		if output != "" {
			t.Errorf("Expected empty output, got %q", output)
		}
	})

	t.Run("Empty Architecture", func(t *testing.T) {
		arch := &language.Architecture{Name: "Empty"}
		exporter := markdown.NewExporter()
		output, err := exporter.Export(arch)
		if err != nil {
			t.Fatalf("Failed to export: %v", err)
		}
		if !strings.Contains(output, "# Empty") {
			t.Errorf("Expected header '# Empty', got:\n%s", output)
		}
		// Should not contain other sections
		if strings.Contains(output, "## Systems") {
			t.Error("Should not contain Systems section")
		}
	})

	t.Run("System with No Items", func(t *testing.T) {
		arch := &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{ID: "S1", Label: "System 1"},
			},
		}
		exporter := markdown.NewExporter()
		output, err := exporter.Export(arch)
		if err != nil {
			t.Fatalf("Failed to export: %v", err)
		}
		if !strings.Contains(output, "System 1") {
			t.Error("Expected System 1 label")
		}
		// Should not have containers list if empty
		if strings.Contains(output, "### Containers") {
			t.Error("Should not contain Containers subsection")
		}
	})

	t.Run("Special Characters", func(t *testing.T) {
		arch := &language.Architecture{
			Name: "Special & Characters",
			Systems: []*language.System{
				{ID: "S1", Label: "System <1>"},
			},
		}
		exporter := markdown.NewExporter()
		output, err := exporter.Export(arch)
		if err != nil {
			t.Fatalf("Failed to export: %v", err)
		}
		// Markdown exporter usually doesn't escape HTML unless configured,
		// but let's check it renders the string at least.
		if !strings.Contains(output, "Special & Characters") {
			t.Error("Expected architecture name with special chars")
		}
		if !strings.Contains(output, "System <1>") {
			t.Error("Expected system label with special chars")
		}
	})

	t.Run("Missing Optional Fields", func(t *testing.T) {
		arch := &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID:    "S1",
					Label: "System 1",
					// No Description
				},
			},
		}
		exporter := markdown.NewExporter()
		_, err := exporter.Export(arch)
		if err != nil {
			t.Fatalf("Failed to export: %v", err)
		}
		// Should not print "Description" header or empty description block
		// This depends on template implementation, but generally we expect clean output

	})
}
