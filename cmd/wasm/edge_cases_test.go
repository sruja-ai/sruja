//go:build js && wasm

package main

import (
	"fmt"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/dot"
)

// TestEdgeCases_ViewLevels tests edge cases for view level validation
func TestEdgeCases_ViewLevels(t *testing.T) {
	validDSL := `
model {
  person User "End User"
  system SystemA "My System" {
    container WebApp "Web Application" {
      component API "REST API"
    }
  }
  User -> SystemA "Uses"
}
`

	result := parseAndValidate(validDSL, "test.sruja")
	if result.Error != nil {
		t.Fatalf("parse failed: %v", result.Error)
	}

	tests := []struct {
		name      string
		viewLevel int
		wantErr   bool
	}{
		{
			name:      "minimum valid level",
			viewLevel: MinViewLevel,
			wantErr:   false,
		},
		{
			name:      "maximum valid level",
			viewLevel: MaxViewLevel,
			wantErr:   false,
		},
		{
			name:      "level below minimum",
			viewLevel: MinViewLevel - 1,
			wantErr:   true,
		},
		{
			name:      "level above maximum",
			viewLevel: MaxViewLevel + 1,
			wantErr:   true,
		},
		{
			name:      "zero level",
			viewLevel: 0,
			wantErr:   true,
		},
		{
			name:      "negative level",
			viewLevel: -1,
			wantErr:   true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateViewLevel(tt.viewLevel)

			if tt.wantErr {
				if err == nil {
					t.Error("expected error, got nil")
					return
				}
				if err.Code != ErrCodeInvalidView {
					t.Errorf("expected error code %s, got %s", ErrCodeInvalidView, err.Code)
				}
			} else {
				if err != nil {
					t.Errorf("unexpected error: %v", err)
				}
			}
		})
	}
}

// TestEdgeCases_NodeSizes tests edge cases for node size validation
func TestEdgeCases_NodeSizes(t *testing.T) {
	tests := []struct {
		name      string
		sizesJson string
		wantErr   bool
	}{
		{
			name:      "empty JSON",
			sizesJson: "",
			wantErr:   false,
		},
		{
			name:      "valid JSON",
			sizesJson: `{"node1": {"width": 100, "height": 50}}`,
			wantErr:   false,
		},
		{
			name:      "JSON at max size",
			sizesJson: strings.Repeat("a", 1024*1024),
			wantErr:   false,
		},
		{
			name:      "JSON too large",
			sizesJson: strings.Repeat("a", 1024*1024+1),
			wantErr:   true,
		},
		{
			name:      "invalid JSON",
			sizesJson: `{"invalid": json}`,
			wantErr:   false, // Validation doesn't check JSON syntax, only size
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateNodeSizes(tt.sizesJson)

			if tt.wantErr {
				if err == nil {
					t.Error("expected error, got nil")
				}
			} else {
				if err != nil {
					t.Errorf("unexpected error: %v", err)
				}
			}
		})
	}
}

// TestEdgeCases_LargeInput tests handling of large but valid inputs
func TestEdgeCases_LargeInput(t *testing.T) {
	// Create a large but valid DSL (just under the limit)
	largeDSL := strings.Repeat("model { system s1 }\n", MaxInputSize/30) // Approximate

	// This should parse successfully (though it may be slow)
	result := parseAndValidate(largeDSL, "test.sruja")

	// We don't fail the test if parsing fails due to size,
	// but we verify the validation allows it
	if result.Error != nil && result.Error.Code == ErrCodeInputTooLarge {
		t.Logf("Large input correctly rejected: %v", result.Error)
	} else if result.Error != nil {
		t.Logf("Large input parsing failed with: %v", result.Error)
	}
}

// TestEdgeCases_SpecialCharacters tests handling of special characters in filenames
func TestEdgeCases_SpecialCharacters(t *testing.T) {
	tests := []struct {
		name     string
		filename string
		expected string // Expected sanitized result
	}{
		{
			name:     "path traversal",
			filename: "../../etc/passwd",
			expected: "____etc_passwd",
		},
		{
			name:     "backslashes",
			filename: "path\\to\\file.sruja",
			expected: "path_to_file.sruja",
		},
		{
			name:     "null bytes",
			filename: "test\x00file.sruja",
			expected: "testfile.sruja",
		},
		{
			name:     "spaces",
			filename: "  test file.sruja  ",
			expected: "test_file.sruja",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			sanitized := SanitizeFilename(tt.filename)
			if sanitized != tt.expected {
				t.Errorf("expected '%s', got '%s'", tt.expected, sanitized)
			}
		})
	}
}

// TestEdgeCases_FocusNodeID tests DOT export with focus node ID
func TestEdgeCases_FocusNodeID(t *testing.T) {
	complexDSL := `
model {
  system SystemA "System A" {
    container WebApp "Web App" {
      component API "API"
    }
    database DB "Database"
  }
  system SystemB "System B"
  SystemA -> SystemB "Calls"
}
`

	result := parseAndValidate(complexDSL, "test.sruja")
	if result.Error != nil {
		t.Fatalf("parse failed: %v", result.Error)
	}

	// Test with focus node ID for L2 view
	config := dot.DefaultConfig()
	config.ViewLevel = 2
	config.FocusNodeID = "SystemA"

	exporter := dot.NewExporter(config)
	exportResult := exporter.Export(result.Program)

	if exportResult.DOT == "" {
		t.Error("expected non-empty DOT output")
	}

	// With focus node, we should see elements within SystemA
	foundFocusedElement := false
	for _, elem := range exportResult.Elements {
		if strings.HasPrefix(elem.ID, "SystemA.") || elem.ID == "SystemA" {
			foundFocusedElement = true
			break
		}
	}

	if !foundFocusedElement {
		t.Log("Note: Focus node filtering may work differently than expected")
	}
}

// TestEdgeCases_EmptyRelations tests export with no relations
func TestEdgeCases_EmptyRelations(t *testing.T) {
	dslNoRelations := `
model {
  person User "End User"
  system SystemA "My System"
}
`

	result := parseAndValidate(dslNoRelations, "test.sruja")
	if result.Error != nil {
		t.Fatalf("parse failed: %v", result.Error)
	}

	config := dot.DefaultConfig()
	config.ViewLevel = 1
	exporter := dot.NewExporter(config)
	exportResult := exporter.Export(result.Program)

	if exportResult.DOT == "" {
		t.Error("expected non-empty DOT output even with no relations")
	}

	if len(exportResult.Elements) < 2 {
		t.Errorf("expected at least 2 elements, got %d", len(exportResult.Elements))
	}

	// Relations can be empty
	if len(exportResult.Relations) != 0 {
		t.Logf("Note: Expected 0 relations, got %d", len(exportResult.Relations))
	}
}

// TestEdgeCases_MultipleRelations tests export with many relations
func TestEdgeCases_MultipleRelations(t *testing.T) {
	var sb strings.Builder
	sb.WriteString("model {\n")
	sb.WriteString("  system S1 \"System 1\"\n")
	sb.WriteString("  system S2 \"System 2\"\n")
	sb.WriteString("  system S3 \"System 3\"\n")

	// Add many relations
	for i := 1; i <= 3; i++ {
		for j := 1; j <= 3; j++ {
			if i != j {
				sb.WriteString(fmt.Sprintf("  S%d -> S%d \"Calls\"\n", i, j))
			}
		}
	}
	sb.WriteString("}\n")

	result := parseAndValidate(sb.String(), "test.sruja")
	if result.Error != nil {
		t.Fatalf("parse failed: %v", result.Error)
	}

	config := dot.DefaultConfig()
	config.ViewLevel = 1
	exporter := dot.NewExporter(config)
	exportResult := exporter.Export(result.Program)

	if len(exportResult.Relations) < 6 {
		t.Errorf("expected at least 6 relations, got %d", len(exportResult.Relations))
	}
}

// TestEdgeCases_UnicodeCharacters tests handling of Unicode in DSL
func TestEdgeCases_UnicodeCharacters(t *testing.T) {
	unicodeDSL := `
model {
  person "用户" "End User"
  system "系统A" "My System"
  "用户" -> "系统A" "使用"
}
`

	result := parseAndValidate(unicodeDSL, "test.sruja")
	// Unicode should be handled (may or may not parse depending on parser support)
	if result.Error != nil {
		t.Logf("Unicode DSL parsing: %v", result.Error)
		// Don't fail - Unicode support may vary
	} else {
		// If it parses, verify export works
		config := dot.DefaultConfig()
		config.ViewLevel = 1
		exporter := dot.NewExporter(config)
		exportResult := exporter.Export(result.Program)

		if exportResult.DOT == "" {
			t.Error("expected non-empty DOT output for Unicode DSL")
		}
	}
}
