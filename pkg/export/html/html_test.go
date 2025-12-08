package html

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestNewExporter(t *testing.T) {
	t.Setenv("SRUJA_VIEWER_URL", "") // Ensure clean environment variable
	exporter := NewExporter()
	// Mode check skipped as it depends on .env.local file presence
	if !exporter.EmbedJSON {
		t.Errorf("Expected EmbedJSON to be true by default")
	}
}

func TestExportFromArchitecture(t *testing.T) {
	t.Setenv("SRUJA_VIEWER_URL", "")
	// Create a simple architecture
	system := &language.System{
		ID: "System1",
	}
	arch := &language.Architecture{
		Name: "TestArch",
		Items: []language.ArchitectureItem{
			{
				System: system,
			},
		},
		Systems: []*language.System{system},
	}

	exporter := NewExporter()
	exporter.Mode = ModeSVG // Use SVG mode (default)
	htmlOutput, err := exporter.ExportFromArchitecture(arch)
	if err != nil {
		t.Fatalf("ExportFromArchitecture failed: %v", err)
	}

	// Check for key HTML elements
	if !strings.Contains(htmlOutput, "<!DOCTYPE html>") {
		t.Error("Output missing DOCTYPE")
	}
	if !strings.Contains(htmlOutput, "<title>Architecture: TestArch</title>") {
		t.Error("Output missing correct title")
	}
	// Check if SVG content is present (SVG mode)
	if !strings.Contains(htmlOutput, "<svg") {
		t.Errorf("Output missing SVG content. Output:\n%s", htmlOutput)
	}
	// Check if JSON data is embedded
	if !strings.Contains(htmlOutput, "System1") {
		t.Errorf("Output missing embedded JSON data (System1). Output:\n%s", htmlOutput)
	}
}

func TestExportFromJSON(t *testing.T) {
	jsonData := []byte(`{
		"metadata": {
			"name": "JSONArch"
		},
		"architecture": {
			"systems": [
				{
					"id": "sys1",
					"name": "System1",
					"type": "system"
				}
			]
		}
	}`)

	exporter := NewExporter()
	exporter.Mode = ModeSVG   // Use SVG mode
	exporter.EmbedJSON = true // Embed JSON
	htmlOutput, err := exporter.ExportFromJSON(jsonData)
	if err != nil {
		t.Fatalf("ExportFromJSON failed: %v", err)
	}

	if !strings.Contains(htmlOutput, "<title>Architecture: JSONArch</title>") {
		t.Error("Output missing correct title from JSON")
	}
	if !strings.Contains(htmlOutput, "System1") {
		t.Error("Output missing embedded JSON data")
	}
}

func TestExportModes(t *testing.T) {
	t.Setenv("SRUJA_VIEWER_URL", "")
	arch := &language.Architecture{Name: "ModeTest"}

	tests := []struct {
		name     string
		mode     ExportMode
		checkStr string
	}{
		{
			name:     "SVG",
			mode:     ModeSVG,
			checkStr: "<svg",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			exporter := NewExporter()
			exporter.Mode = tt.mode

			output, err := exporter.ExportFromArchitecture(arch)
			if err != nil {
				t.Fatalf("Export failed: %v", err)
			}

			if !strings.Contains(output, tt.checkStr) {
				t.Errorf("Expected output to contain %q, got ...", tt.checkStr)
			}
		})
	}
}

func TestSVGModeContent(t *testing.T) {
	t.Setenv("SRUJA_VIEWER_URL", "")
	arch := &language.Architecture{Name: "SVGTest"}
	exporter := NewExporter()
	exporter.Mode = ModeSVG
	output, err := exporter.ExportFromArchitecture(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}
	if !strings.Contains(output, "<svg") {
		t.Errorf("Expected SVG mode to contain SVG content")
	}
	if !strings.Contains(output, "sruja-data") {
		t.Errorf("Expected SVG mode to contain embedded JSON data")
	}
}

func TestSVGModeHasDSLContent(t *testing.T) {
	arch := &language.Architecture{Name: "DSLTest"}
	exporter := NewExporter()
	exporter.Mode = ModeSVG
	output, err := exporter.ExportFromArchitecture(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}
	// SVG mode should include DSL content in the DSL tab
	if !strings.Contains(output, "dsl-content") {
		t.Errorf("Expected SVG mode to contain DSL content element")
	}
	if !strings.Contains(output, "dsl-view") {
		t.Errorf("Expected SVG mode to contain DSL view")
	}
}

func TestSVGModeHasFilterTabs(t *testing.T) {
	arch := &language.Architecture{Name: "FilterTest"}
	exporter := NewExporter()
	exporter.Mode = ModeSVG

	output, err := exporter.ExportFromArchitecture(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	// SVG mode should have filter tabs in sidebar
	if !strings.Contains(output, "filter-tab") {
		t.Error("Output missing filter tabs")
	}
	if !strings.Contains(output, "filter-nav") {
		t.Error("Output missing filter navigation")
	}
}

func TestExport(t *testing.T) {
	tmpDir := t.TempDir()
	outFile := filepath.Join(tmpDir, "output.html")
	arch := &language.Architecture{Name: "ExportTest"}

	exporter := NewExporter()
	exporter.Mode = ModeSVG
	if err := exporter.Export(arch, outFile); err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	content, err := os.ReadFile(outFile)
	if err != nil {
		t.Fatalf("Failed to read output file: %v", err)
	}

	if !strings.Contains(string(content), "ExportTest") {
		t.Error("Output file missing architecture name")
	}
}

func TestExportSingleFile(t *testing.T) {
	// Single-file mode uses embedded bundles, not files from OutputDir
	// This test verifies that single-file mode works with embedded bundles
	arch := &language.Architecture{Name: "SingleFileTest"}
	exporter := NewExporter()
	exporter.Mode = ModeSingleFile

	output, err := exporter.ExportFromArchitecture(arch)
	// Single-file mode requires embedded bundles to be built
	// If not built, we expect an error
	if err != nil {
		// This is expected if bundles aren't built - skip test
		if strings.Contains(err.Error(), "embedded viewer JS is not built") ||
			strings.Contains(err.Error(), "failed to read embedded viewer JS") {
			t.Skip("Skipping test: embedded bundles not built (run 'make build-embed-viewer')")
		}
		t.Fatalf("ExportFromArchitecture failed: %v", err)
	}

	// If we get here, bundles are embedded - verify they're in output
	if !strings.Contains(output, "<!DOCTYPE html>") {
		t.Error("Output missing DOCTYPE")
	}
	if !strings.Contains(output, "SingleFileTest") {
		t.Error("Output missing architecture name")
	}
}
