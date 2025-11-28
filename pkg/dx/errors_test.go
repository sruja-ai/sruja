// pkg/dx/errors_test.go
package dx

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
)

func TestEnhancedError_Format(t *testing.T) {
	err := &EnhancedError{
		Message: "Test error",
		Line:    10,
		Column:  5,
		File:    "test.sruja",
	}

	formatted := err.Format(false)
	if !strings.Contains(formatted, "Test error") {
		t.Error("Formatted error should contain message")
	}
	if !strings.Contains(formatted, "test.sruja:10:5") {
		t.Error("Formatted error should contain location")
	}
}

func TestEnhancedError_Format_WithContext(t *testing.T) {
	err := &EnhancedError{
		Message: "Test error",
		Line:    10,
		Column:  5,
		Context: "line 10: system API {}",
	}

	formatted := err.Format(false)
	if !strings.Contains(formatted, "Context:") {
		t.Error("Formatted error should contain context section")
	}
	if !strings.Contains(formatted, "line 10: system API {}") {
		t.Error("Formatted error should contain context text")
	}
}

func TestEnhancedError_Format_WithSuggestions(t *testing.T) {
	err := &EnhancedError{
		Message:     "Test error",
		Line:        10,
		Column:      5,
		Suggestions: []string{"Suggestion 1", "Suggestion 2", "Suggestion 3", "Suggestion 4"},
	}

	formatted := err.Format(false)
	if !strings.Contains(formatted, "Suggestions:") {
		t.Error("Formatted error should contain suggestions section")
	}
	if !strings.Contains(formatted, "Suggestion 1") {
		t.Error("Formatted error should contain first suggestion")
	}
	// Should limit to 3 suggestions
	if strings.Count(formatted, "→") > 3 {
		t.Error("Formatted error should limit suggestions to 3")
	}
}

func TestEnhancedError_Format_WithQuickFix(t *testing.T) {
	err := &EnhancedError{
		Message:  "Test error",
		Line:     10,
		Column:   5,
		QuickFix: "Add missing field",
	}

	formatted := err.Format(false)
	if !strings.Contains(formatted, "Quick fix:") {
		t.Error("Formatted error should contain quick fix section")
	}
	if !strings.Contains(formatted, "Add missing field") {
		t.Error("Formatted error should contain quick fix text")
	}
}

func TestEnhancedError_Format_NoFile(t *testing.T) {
	err := &EnhancedError{
		Message: "Test error",
		Line:    10,
		Column:  5,
	}

	formatted := err.Format(false)
	if !strings.Contains(formatted, "line 10, column 5") {
		t.Error("Formatted error should show line/column when file is empty")
	}
}

func TestErrorEnhancer_Enhance(t *testing.T) {
	fileLines := []string{
		"line 1",
		"line 2",
		"system API {}",
		"line 4",
		"line 5",
	}

	enhancer := NewErrorEnhancer("test.sruja", fileLines, nil)
	valErr := engine.ValidationError{
		Message: "Unknown element 'API'",
		Line:    3,
		Column:  8,
	}

	enhanced := enhancer.Enhance(valErr)
	if enhanced == nil {
		t.Fatal("Enhance should return an error")
	}
	if enhanced.Message != valErr.Message {
		t.Errorf("Expected message '%s', got '%s'", valErr.Message, enhanced.Message)
	}
	if enhanced.Line != 3 {
		t.Errorf("Expected line 3, got %d", enhanced.Line)
	}
	if enhanced.File != "test.sruja" {
		t.Errorf("Expected file 'test.sruja', got '%s'", enhanced.File)
	}
}

func TestErrorEnhancer_ExtractContext(t *testing.T) {
	fileLines := []string{
		"line 1",
		"line 2",
		"system API {}",
		"line 4",
		"line 5",
	}

	enhancer := NewErrorEnhancer("test.sruja", fileLines, nil)
	valErr := engine.ValidationError{
		Message: "Error",
		Line:    3,
		Column:  8,
	}

	enhanced := enhancer.Enhance(valErr)
	if enhanced.Context == "" {
		t.Error("Context should be extracted")
	}
	if !strings.Contains(enhanced.Context, "system API {}") {
		t.Error("Context should contain the error line")
	}
}

func TestErrorEnhancer_GenerateSuggestions_UnknownReference(t *testing.T) {
	fileLines := []string{"system API {}"}
	enhancer := NewErrorEnhancer("test.sruja", fileLines, nil)
	valErr := engine.ValidationError{
		Message: "Unknown element 'X'",
		Line:    1,
		Column:  1,
	}

	enhanced := enhancer.Enhance(valErr)
	if len(enhanced.Suggestions) == 0 {
		t.Error("Should generate suggestions for unknown reference errors")
	}
}

func TestErrorEnhancer_GenerateSuggestions_Duplicate(t *testing.T) {
	fileLines := []string{"system API {}"}
	enhancer := NewErrorEnhancer("test.sruja", fileLines, nil)
	valErr := engine.ValidationError{
		Message: "Duplicate ID 'API'",
		Line:    1,
		Column:  1,
	}

	enhanced := enhancer.Enhance(valErr)
	if len(enhanced.Suggestions) == 0 {
		t.Error("Should generate suggestions for duplicate errors")
	}
}

func TestErrorEnhancer_GenerateSuggestions_Cycle(t *testing.T) {
	fileLines := []string{"system API {}"}
	enhancer := NewErrorEnhancer("test.sruja", fileLines, nil)
	valErr := engine.ValidationError{
		Message: "Circular dependency detected",
		Line:    1,
		Column:  1,
	}

	enhanced := enhancer.Enhance(valErr)
	if len(enhanced.Suggestions) == 0 {
		t.Error("Should generate suggestions for cycle errors")
	}
}

func TestErrorEnhancer_GenerateSuggestions_MissingMetadata(t *testing.T) {
	fileLines := []string{"system API {}"}
	enhancer := NewErrorEnhancer("test.sruja", fileLines, nil)
	valErr := engine.ValidationError{
		Message: "Missing metadata 'owner'",
		Line:    1,
		Column:  1,
	}

	enhanced := enhancer.Enhance(valErr)
	if len(enhanced.Suggestions) == 0 {
		t.Error("Should generate suggestions for missing metadata errors")
	}
}

func TestErrorEnhancer_GenerateSuggestions_Import(t *testing.T) {
	fileLines := []string{"import \"test.sruja\""}
	enhancer := NewErrorEnhancer("test.sruja", fileLines, nil)
	valErr := engine.ValidationError{
		Message: "Cannot resolve import",
		Line:    1,
		Column:  1,
	}

	enhanced := enhancer.Enhance(valErr)
	if len(enhanced.Suggestions) == 0 {
		t.Error("Should generate suggestions for import errors")
	}
}

func TestErrorEnhancer_ExtractContext_EdgeCases(t *testing.T) {
	fileLines := []string{"line 1", "line 2"}
	enhancer := NewErrorEnhancer("test.sruja", fileLines, nil)

	// Test with line 1 (start of file)
	valErr1 := engine.ValidationError{
		Message: "Error",
		Line:    1,
		Column:  1,
	}
	enhanced1 := enhancer.Enhance(valErr1)
	if enhanced1.Context == "" {
		t.Error("Should extract context even at start of file")
	}

	// Test with line beyond file (should still extract available context)
	valErr2 := engine.ValidationError{
		Message: "Error",
		Line:    10,
		Column:  1,
	}
	enhanced2 := enhancer.Enhance(valErr2)
	// When line is beyond file, it will show available lines up to end
	// Context may be empty or show last lines
	_ = enhanced2.Context

	// Test with column beyond line length
	valErr3 := engine.ValidationError{
		Message: "Error",
		Line:    1,
		Column:  100,
	}
	enhanced3 := enhancer.Enhance(valErr3)
	if enhanced3.Context == "" {
		t.Error("Should handle column beyond line length")
	}
}

func TestErrorEnhancer_ExtractContext_EmptyFile(t *testing.T) {
	fileLines := []string{}
	enhancer := NewErrorEnhancer("test.sruja", fileLines, nil)
	valErr := engine.ValidationError{
		Message: "Error",
		Line:    1,
		Column:  1,
	}

	enhanced := enhancer.Enhance(valErr)
	if enhanced.Context != "" {
		t.Error("Context should be empty for empty file")
	}
}

func TestEnhancedError_Format_WithColor(t *testing.T) {
	err := &EnhancedError{
		Message: "Test error",
		Line:    10,
		Column:  5,
		File:    "test.sruja",
	}

	formatted := err.Format(true)
	if !strings.Contains(formatted, "Test error") {
		t.Error("Formatted error should contain message")
	}
	if !strings.Contains(formatted, "\033[31m") {
		t.Error("Formatted error with color should contain ANSI codes")
	}
}

func TestFormatErrors_WithColor(t *testing.T) {
	errors := []*EnhancedError{
		{
			Message: "Error 1",
			Line:    1,
			Column:  1,
		},
	}

	formatted := FormatErrors(errors, true)
	if !strings.Contains(formatted, "Error 1") {
		t.Error("FormatErrors should contain error")
	}
	if !strings.Contains(formatted, "\033[31m") {
		t.Error("FormatErrors with color should contain ANSI codes")
	}
}

func TestFormatErrors(t *testing.T) {
	errors := []*EnhancedError{
		{
			Message: "Error 1",
			Line:    1,
			Column:  1,
		},
		{
			Message: "Error 2",
			Line:    2,
			Column:  2,
		},
	}

	formatted := FormatErrors(errors, false)
	if !strings.Contains(formatted, "2 error(s)") {
		t.Error("FormatErrors should show error count")
	}
	if !strings.Contains(formatted, "Error 1") {
		t.Error("FormatErrors should contain first error")
	}
	if !strings.Contains(formatted, "Error 2") {
		t.Error("FormatErrors should contain second error")
	}
}

func TestFormatErrors_Empty(t *testing.T) {
	errors := []*EnhancedError{}
	formatted := FormatErrors(errors, false)
	if formatted != "" {
		t.Errorf("FormatErrors with empty slice should return empty string, got '%s'", formatted)
	}
}

func TestExtractRuleName(t *testing.T) {
	// Test through ErrorEnhancer.Enhance which uses extractRuleName
	fileLines := []string{"system API {}"}
	enhancer := NewErrorEnhancer("test.sruja", fileLines, nil)

	tests := []struct {
		msg      string
		expected string
	}{
		{"duplicate ID", "Unique IDs"},
		{"unknown element", "Valid References"},
		{"invalid reference", "Valid References"},
		{"circular dependency", "Cycle Detection"},
		{"cycle detected", "Cycle Detection"},
		{"orphan element", "Orphan Detection"},
		{"other error", "Validation"},
	}

	for _, tt := range tests {
		valErr := engine.ValidationError{
			Message: tt.msg,
			Line:    1,
			Column:  1,
		}
		enhanced := enhancer.Enhance(valErr)
		if enhanced.RuleName != tt.expected {
			t.Errorf("RuleName for %q = %q, want %q", tt.msg, enhanced.RuleName, tt.expected)
		}
	}
}
