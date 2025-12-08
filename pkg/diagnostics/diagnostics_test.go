package diagnostics

import (
	"strings"
	"testing"
)

func TestSourceLocation_String(t *testing.T) {
	loc := SourceLocation{
		File:   "test.sruja",
		Line:   10,
		Column: 5,
	}
	expected := "test.sruja:10:5"
	if got := loc.String(); got != expected {
		t.Errorf("SourceLocation.String() = %q, want %q", got, expected)
	}
}

func TestDiagnostic_String(t *testing.T) {
	diag := Diagnostic{
		Code:     CodeSyntaxError,
		Severity: SeverityError,
		Message:  "test error",
		Location: SourceLocation{File: "test.sruja", Line: 1, Column: 1},
	}
	expected := "[E101] Error: test error at test.sruja:1:1"
	if got := diag.String(); got != expected {
		t.Errorf("Diagnostic.String() = %q, want %q", got, expected)
	}
}

func TestBasicErrorReporter(t *testing.T) {
	reporter := NewBasicErrorReporter()

	if reporter.HasErrors() {
		t.Error("New reporter should not have errors")
	}
	if len(reporter.Diagnostics()) != 0 {
		t.Error("New reporter should have empty diagnostics")
	}

	// Report Info
	reporter.Report(Diagnostic{
		Code:     CodeOrphanElement,
		Severity: SeverityInfo,
		Message:  "info message",
	})

	if reporter.HasErrors() {
		t.Error("Reporter should not have errors with only Info diagnostics")
	}
	if len(reporter.Diagnostics()) != 1 {
		t.Error("Reporter should have 1 diagnostic")
	}

	// Report Error
	reporter.Report(Diagnostic{
		Code:     CodeSyntaxError,
		Severity: SeverityError,
		Message:  "error message",
	})

	if !reporter.HasErrors() {
		t.Error("Reporter should have errors after reporting Error diagnostic")
	}
	if len(reporter.Diagnostics()) != 2 {
		t.Error("Reporter should have 2 diagnostics")
	}
}

func TestFormatDiagnostic(t *testing.T) {
	diag := Diagnostic{
		Code:     CodeSyntaxError,
		Severity: SeverityError,
		Message:  "unexpected token 'foo'",
		Location: SourceLocation{File: "test.sruja", Line: 5, Column: 10},
		Context: []string{
			"system A {",
			"  foo",
			"}",
		},
		Suggestions: []string{
			"Did you mean 'component'?",
			"Did you mean 'container'?",
		},
	}

	formatted := FormatDiagnostic(diag)

	checks := []string{
		"[E101] Error: unexpected token 'foo'",
		"--> test.sruja:5:10",
		"| system A {",
		"|   foo",
		"| }",
		"= Help: Did you mean 'component'?",
		"          Did you mean 'container'?",
	}

	for _, check := range checks {
		if !strings.Contains(formatted, check) {
			t.Errorf("FormatDiagnostic output missing %q", check)
		}
	}
}
