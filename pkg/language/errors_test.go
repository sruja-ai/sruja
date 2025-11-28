// pkg/language/errors_test.go
package language_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParseError_Error(t *testing.T) {
	loc := language.SourceLocation{
		File:   "test.sruja",
		Line:   5,
		Column: 12,
	}
	err := language.NewParseError(loc, "Expected ':' after identifier")

	errorStr := err.Error()
	if !strings.Contains(errorStr, "test.sruja") {
		t.Error("Error should contain file name")
	}
	if !strings.Contains(errorStr, "Expected ':'") {
		t.Error("Error should contain message")
	}
}

func TestParseError_Severity(t *testing.T) {
	loc := language.SourceLocation{File: "test.sruja", Line: 1, Column: 1}
	err := language.NewParseError(loc, "Test error")

	if err.Severity != language.ErrorSeverityError {
		t.Errorf("Expected ErrorSeverityError, got %v", err.Severity)
	}
}

func TestValidationError_Error(t *testing.T) {
	loc := language.SourceLocation{
		File:   "test.sruja",
		Line:   10,
		Column: 5,
	}
	err := language.NewValidationError(loc, "semantic/duplicate-id", "Element 'API' is already defined")

	errorStr := err.Error()
	if !strings.Contains(errorStr, "test.sruja") {
		t.Error("Error should contain file name")
	}
	if !strings.Contains(errorStr, "semantic/duplicate-id") {
		t.Error("Error should contain rule ID")
	}
	if !strings.Contains(errorStr, "Element 'API'") {
		t.Error("Error should contain message")
	}
}

func TestValidationError_Severity(t *testing.T) {
	loc := language.SourceLocation{File: "test.sruja", Line: 1, Column: 1}
	err := language.NewValidationError(loc, "rule1", "Test error")

	if err.Severity != language.ErrorSeverityError {
		t.Errorf("Expected ErrorSeverityError, got %v", err.Severity)
	}
}

func TestCompilationError_Error(t *testing.T) {
	loc := language.SourceLocation{
		File:   "test.sruja",
		Line:   3,
		Column: 1,
	}
	err := language.NewCompilationError(loc, "System 'API' is missing required 'id' field")

	errorStr := err.Error()
	if !strings.Contains(errorStr, "test.sruja") {
		t.Error("Error should contain file name")
	}
	if !strings.Contains(errorStr, "System 'API'") {
		t.Error("Error should contain message")
	}
}

func TestCompilationError_Severity(t *testing.T) {
	loc := language.SourceLocation{File: "test.sruja", Line: 1, Column: 1}
	err := language.NewCompilationError(loc, "Test error")

	if err.Severity != language.ErrorSeverityError {
		t.Errorf("Expected ErrorSeverityError, got %v", err.Severity)
	}
}

func TestErrorList_Add(t *testing.T) {
	var el language.ErrorList
	loc := language.SourceLocation{File: "test.sruja", Line: 1, Column: 1}

	err1 := language.NewParseError(loc, "Error 1")
	err2 := language.NewValidationError(loc, "rule1", "Error 2")

	el.Add(err1)
	el.Add(err2)

	if len(el.Errors) != 2 {
		t.Errorf("Expected 2 errors, got %d", len(el.Errors))
	}
}

func TestErrorList_HasErrors(t *testing.T) {
	var el language.ErrorList

	if el.HasErrors() {
		t.Error("Empty error list should not have errors")
	}

	loc := language.SourceLocation{File: "test.sruja", Line: 1, Column: 1}
	el.Add(language.NewParseError(loc, "Error"))

	if !el.HasErrors() {
		t.Error("Error list with errors should return true")
	}
}

func TestErrorList_Error_NoErrors(t *testing.T) {
	var el language.ErrorList
	errorStr := el.Error()

	if errorStr != "no errors" {
		t.Errorf("Expected 'no errors', got %q", errorStr)
	}
}

func TestErrorList_Error_SingleError(t *testing.T) {
	var el language.ErrorList
	loc := language.SourceLocation{File: "test.sruja", Line: 1, Column: 1}
	err := language.NewParseError(loc, "Test error")
	el.Add(err)

	errorStr := el.Error()
	if !strings.Contains(errorStr, "Test error") {
		t.Errorf("Error should contain error message, got %q", errorStr)
	}
}

func TestErrorList_Error_MultipleErrors(t *testing.T) {
	var el language.ErrorList
	loc := language.SourceLocation{File: "test.sruja", Line: 1, Column: 1}

	el.Add(language.NewParseError(loc, "Error 1"))
	el.Add(language.NewValidationError(loc, "rule1", "Error 2"))
	el.Add(language.NewCompilationError(loc, "Error 3"))

	errorStr := el.Error()
	if !strings.Contains(errorStr, "3 errors") {
		t.Errorf("Error should mention 3 errors, got %q", errorStr)
	}
	if !strings.Contains(errorStr, "Error 1") {
		t.Error("Error should contain first error")
	}
}

func TestErrorSeverity_Constants(t *testing.T) {
	if language.ErrorSeverityError != "error" {
		t.Errorf("Expected 'error', got %q", language.ErrorSeverityError)
	}
	if language.ErrorSeverityWarning != "warning" {
		t.Errorf("Expected 'warning', got %q", language.ErrorSeverityWarning)
	}
	if language.ErrorSeverityInfo != "info" {
		t.Errorf("Expected 'info', got %q", language.ErrorSeverityInfo)
	}
}
