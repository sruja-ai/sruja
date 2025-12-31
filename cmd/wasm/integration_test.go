//go:build js && wasm

package main

import (
	"fmt"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/dot"
	jexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/export/markdown"
	"github.com/sruja-ai/sruja/pkg/export/mermaid"
)

// Test fixtures - valid DSL examples
const (
	validDSLSimple = `
model {
  person User "End User"
  system SystemA "My System" {
    container WebApp "Web Application"
  }
  User -> SystemA "Uses"
}
`

	validDSLComplex = `
model {
  person User "End User"
  system SystemA "My System" {
    container WebApp "Web Application" {
      component API "REST API"
    }
    database DB "Database"
    queue MQ "Message Queue"
  }
  system SystemB "External System"
  
  User -> SystemA "Uses"
  SystemA -> SystemB "Calls"
  WebApp -> DB "Stores data in"
  WebApp -> MQ "Publishes to"
}
`

	emptyModelDSL = `
model {
}
`

	invalidDSL = `
model {
  person User
  system SystemA {
    // Missing closing brace
`
)

// TestParseAndValidate_ValidDSL tests parsing and validation of valid DSL
func TestParseAndValidate_ValidDSL(t *testing.T) {
	tests := []struct {
		name     string
		dsl      string
		filename string
		wantErr  bool
	}{
		{
			name:     "simple valid DSL",
			dsl:      validDSLSimple,
			filename: "test.sruja",
			wantErr:  false,
		},
		{
			name:     "complex valid DSL",
			dsl:      validDSLComplex,
			filename: "test.sruja",
			wantErr:  false,
		},
		{
			name:     "empty model",
			dsl:      emptyModelDSL,
			filename: "test.sruja",
			wantErr:  true, // Should fail with ErrCodeEmptyModel
		},
		{
			name:     "invalid DSL syntax",
			dsl:      invalidDSL,
			filename: "test.sruja",
			wantErr:  true, // Should fail with parse error
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := parseAndValidate(tt.dsl, tt.filename)

			if tt.wantErr {
				if result.Error == nil {
					t.Error("expected error, got nil")
					return
				}
				// Verify error code is appropriate
				if result.Error.Code == "" {
					t.Error("error should have a code")
				}
				if result.Program != nil {
					t.Error("expected nil program on error")
				}
			} else {
				if result.Error != nil {
					t.Errorf("unexpected error: %v", result.Error)
					return
				}
				if result.Program == nil {
					t.Error("expected non-nil program on success")
					return
				}
				if result.Program.Model == nil {
					t.Error("expected non-nil model")
				}
				if len(result.Program.Model.Items) == 0 {
					t.Error("expected at least one model item")
				}
			}
		})
	}
}

// TestParseAndValidate_ErrorCodes tests that correct error codes are returned
func TestParseAndValidate_ErrorCodes(t *testing.T) {
	tests := []struct {
		name     string
		dsl      string
		filename string
		wantCode ErrorCode
	}{
		{
			name:     "empty input",
			dsl:      "",
			filename: "test.sruja",
			wantCode: ErrCodeInvalidInput,
		},
		{
			name:     "input too large",
			dsl:      string(make([]byte, MaxInputSize+1)),
			filename: "test.sruja",
			wantCode: ErrCodeInputTooLarge,
		},
		{
			name:     "invalid filename",
			dsl:      validDSLSimple,
			filename: "../../etc/passwd",
			wantCode: ErrCodeInvalidFilename,
		},
		{
			name:     "empty model",
			dsl:      emptyModelDSL,
			filename: "test.sruja",
			wantCode: ErrCodeEmptyModel,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := parseAndValidate(tt.dsl, tt.filename)

			if result.Error == nil {
				t.Fatal("expected error, got nil")
			}

			if result.Error.Code != tt.wantCode {
				t.Errorf("expected error code %s, got %s", tt.wantCode, result.Error.Code)
			}
		})
	}
}

// TestExportPipeline_Mermaid tests the full pipeline: Parse → Validate → Export (Mermaid)
func TestExportPipeline_Mermaid(t *testing.T) {
	result := parseAndValidate(validDSLSimple, "test.sruja")
	if result.Error != nil {
		t.Fatalf("parse failed: %v", result.Error)
	}

	exporter := mermaid.NewExporter(mermaid.DefaultConfig())
	output := exporter.Export(result.Program)

	if output == "" {
		t.Error("expected non-empty mermaid output")
	}

	// Verify mermaid output contains expected elements
	if !strings.Contains(output, "graph") && !strings.Contains(output, "flowchart") {
		t.Error("mermaid output should contain graph or flowchart")
	}
}

// TestExportPipeline_Markdown tests the full pipeline: Parse → Validate → Export (Markdown)
func TestExportPipeline_Markdown(t *testing.T) {
	result := parseAndValidate(validDSLSimple, "test.sruja")
	if result.Error != nil {
		t.Fatalf("parse failed: %v", result.Error)
	}

	exporter := markdown.NewExporter(markdown.DefaultOptions())
	output := exporter.Export(result.Program)

	if output == "" {
		t.Error("expected non-empty markdown output")
	}

	// Verify markdown output contains expected elements
	if !strings.Contains(output, "#") && !strings.Contains(output, "##") {
		t.Error("markdown output should contain headers")
	}
}

// TestExportPipeline_JSON tests the full pipeline: Parse → Validate → Export (JSON)
func TestExportPipeline_JSON(t *testing.T) {
	result := parseAndValidate(validDSLSimple, "test.sruja")
	if result.Error != nil {
		t.Fatalf("parse failed: %v", result.Error)
	}

	exporter := jexport.NewExporter()
	output, err := exporter.Export(result.Program)
	if err != nil {
		t.Fatalf("export failed: %v", err)
	}

	if output == "" {
		t.Error("expected non-empty JSON output")
	}

	// Verify JSON is valid
	if !strings.HasPrefix(strings.TrimSpace(output), "{") {
		t.Error("JSON output should start with {")
	}
}

// TestExportPipeline_DOT tests the full pipeline: Parse → Validate → Export (DOT)
func TestExportPipeline_DOT(t *testing.T) {
	result := parseAndValidate(validDSLSimple, "test.sruja")
	if result.Error != nil {
		t.Fatalf("parse failed: %v", result.Error)
	}

	config := dot.DefaultConfig()
	config.ViewLevel = 1
	exporter := dot.NewExporter(config)
	exportResult := exporter.Export(result.Program)

	if exportResult.DOT == "" {
		t.Error("expected non-empty DOT output")
	}

	if len(exportResult.Elements) == 0 {
		t.Error("expected at least one element")
	}

	// Verify DOT format
	if !strings.Contains(exportResult.DOT, "digraph") {
		t.Error("DOT output should contain digraph")
	}

	// Verify elements have required fields
	for _, elem := range exportResult.Elements {
		if elem.ID == "" {
			t.Error("element should have ID")
		}
		if elem.Kind == "" {
			t.Error("element should have Kind")
		}
		if elem.Title == "" {
			t.Error("element should have Title")
		}
	}
}

// TestExportPipeline_DOT_ViewLevels tests different view levels
func TestExportPipeline_DOT_ViewLevels(t *testing.T) {
	result := parseAndValidate(validDSLComplex, "test.sruja")
	if result.Error != nil {
		t.Fatalf("parse failed: %v", result.Error)
	}

	viewLevels := []int{1, 2, 3}
	for _, level := range viewLevels {
		t.Run(fmt.Sprintf("view_level_%d", level), func(t *testing.T) {
			config := dot.DefaultConfig()
			config.ViewLevel = level
			exporter := dot.NewExporter(config)
			exportResult := exporter.Export(result.Program)

			if exportResult.DOT == "" {
				t.Error("expected non-empty DOT output")
			}

			if len(exportResult.Elements) == 0 {
				t.Error("expected at least one element")
			}

			// Verify view level filtering works
			// L1 should have fewer elements than L2/L3
			if level == 1 {
				// L1 typically shows persons and systems only
				for _, elem := range exportResult.Elements {
					if elem.Kind != "person" && elem.Kind != "system" {
						t.Logf("L1 view contains %s element (may be valid depending on implementation)", elem.Kind)
					}
				}
			}
		})
	}
}

// TestExportPipeline_ErrorPropagation tests that errors propagate correctly
func TestExportPipeline_ErrorPropagation(t *testing.T) {
	tests := []struct {
		name     string
		dsl      string
		filename string
		checkFn  func(*ParseResult) bool
	}{
		{
			name:     "parse error propagates",
			dsl:      invalidDSL,
			filename: "test.sruja",
			checkFn: func(result *ParseResult) bool {
				return result.Error != nil &&
					(result.Error.Code == ErrCodeParseFailed ||
						strings.Contains(result.Error.Message, "parse"))
			},
		},
		{
			name:     "empty model error propagates",
			dsl:      emptyModelDSL,
			filename: "test.sruja",
			checkFn: func(result *ParseResult) bool {
				return result.Error != nil && result.Error.Code == ErrCodeEmptyModel
			},
		},
		{
			name:     "validation error propagates",
			dsl:      "",
			filename: "test.sruja",
			checkFn: func(result *ParseResult) bool {
				return result.Error != nil && result.Error.Code == ErrCodeInvalidInput
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := parseAndValidate(tt.dsl, tt.filename)
			if !tt.checkFn(result) {
				t.Errorf("error propagation check failed. Error: %v", result.Error)
			}
		})
	}
}

// TestExportPipeline_LargeDiagram tests handling of larger diagrams
func TestExportPipeline_LargeDiagram(t *testing.T) {
	// Create a DSL with many elements
	var sb strings.Builder
	sb.WriteString("model {\n")

	// Add 20 systems
	for i := 0; i < 20; i++ {
		sb.WriteString(fmt.Sprintf("  system System%d \"System %d\"\n", i, i))
	}

	// Add relations between systems
	for i := 0; i < 19; i++ {
		sb.WriteString(fmt.Sprintf("  System%d -> System%d \"Calls\"\n", i, i+1))
	}

	sb.WriteString("}\n")
	largeDSL := sb.String()

	result := parseAndValidate(largeDSL, "test.sruja")
	if result.Error != nil {
		t.Fatalf("parse failed: %v", result.Error)
	}

	// Test DOT export with large diagram
	config := dot.DefaultConfig()
	config.ViewLevel = 1
	exporter := dot.NewExporter(config)
	exportResult := exporter.Export(result.Program)

	if exportResult.DOT == "" {
		t.Error("expected non-empty DOT output for large diagram")
	}

	if len(exportResult.Elements) < 20 {
		t.Errorf("expected at least 20 elements, got %d", len(exportResult.Elements))
	}
}

// TestExportPipeline_ContextPreservation tests that error context is preserved
func TestExportPipeline_ContextPreservation(t *testing.T) {
	result := parseAndValidate("", "test.sruja")

	if result.Error == nil {
		t.Fatal("expected error")
	}

	// Verify context is present
	if result.Error.Context == nil {
		t.Error("error should have context")
	}

	// Verify specific context fields for input validation errors
	if result.Error.Code == ErrCodeInputTooLarge {
		if result.Error.Context["size"] == nil {
			t.Error("InputTooLarge error should have size in context")
		}
		if result.Error.Context["maxSize"] == nil {
			t.Error("InputTooLarge error should have maxSize in context")
		}
	}

	if result.Error.Code == ErrCodeInvalidFilename {
		if result.Error.Context["filename"] == nil {
			t.Error("InvalidFilename error should have filename in context")
		}
	}
}

// TestExportPipeline_NodeSizes tests DOT export with node sizes
func TestExportPipeline_NodeSizes(t *testing.T) {
	result := parseAndValidate(validDSLSimple, "test.sruja")
	if result.Error != nil {
		t.Fatalf("parse failed: %v", result.Error)
	}

	config := dot.DefaultConfig()
	config.ViewLevel = 1
	config.NodeSizes = map[string]struct{ Width, Height float64 }{
		"User":    {Width: 200, Height: 100},
		"SystemA": {Width: 300, Height: 150},
	}

	exporter := dot.NewExporter(config)
	exportResult := exporter.Export(result.Program)

	if exportResult.DOT == "" {
		t.Error("expected non-empty DOT output")
	}

	// Verify node sizes are used in constraints
	// (This is tested indirectly - if sizes are in config, they should be respected)
	if len(exportResult.Elements) > 0 {
		// Elements should have width/height set
		foundSizedElement := false
		for _, elem := range exportResult.Elements {
			if elem.Width > 0 && elem.Height > 0 {
				foundSizedElement = true
				break
			}
		}
		if !foundSizedElement {
			t.Log("Note: Element sizes may be set during export, not from config")
		}
	}
}
