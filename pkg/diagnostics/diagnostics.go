//nolint:gocritic // rangeValCopy acceptable
package diagnostics

import (
	"fmt"
	"strings"
)

// Severity represents the severity of a diagnostic.
type Severity string

const (
	SeverityError   Severity = "Error"
	SeverityWarning Severity = "Warning"
	SeverityInfo    Severity = "Info"
)

// SourceLocation represents a location in a source file.
type SourceLocation struct {
	File   string
	Line   int
	Column int
}

func (l SourceLocation) String() string {
	return fmt.Sprintf("%s:%d:%d", l.File, l.Line, l.Column)
}

// Diagnostic represents a single error or warning.
type Diagnostic struct {
	Code        string         // Unique error code (e.g., "E001")
	Severity    Severity       // Error, Warning, Info
	Message     string         // The main error message
	Location    SourceLocation // Where the error occurred
	Context     []string       // Surrounding lines of code
	Suggestions []string       // Actionable suggestions (e.g., "Did you mean 'system'?")
}

func (d Diagnostic) String() string {
	return fmt.Sprintf("[%s] %s: %s at %s", d.Code, d.Severity, d.Message, d.Location)
}

// ErrorReporter defines the interface for reporting diagnostics.
type ErrorReporter interface {
	Report(d Diagnostic)
	HasErrors() bool
	Diagnostics() []Diagnostic
}

// BasicErrorReporter is a simple implementation of ErrorReporter.
type BasicErrorReporter struct {
	diagnostics []Diagnostic
}

func NewBasicErrorReporter() *BasicErrorReporter {
	return &BasicErrorReporter{
		diagnostics: make([]Diagnostic, 0),
	}
}

//nolint:gocritic // Struct is small enough
func (r *BasicErrorReporter) Report(d Diagnostic) {
	r.diagnostics = append(r.diagnostics, d)
}

func (r *BasicErrorReporter) HasErrors() bool {
	for _, d := range r.diagnostics {
		if d.Severity == SeverityError {
			return true
		}
	}
	return false
}

func (r *BasicErrorReporter) Diagnostics() []Diagnostic {
	return r.diagnostics
}

// FormatDiagnostic returns a user-friendly string representation of the diagnostic.
// This simulates a "Rust-like" error message.
//
//nolint:gocritic // Struct is small enough
func FormatDiagnostic(d Diagnostic) string {
	var sb strings.Builder

	// Header: [E001] Error: Message
	sb.WriteString(fmt.Sprintf("[%s] %s: %s\n", d.Code, d.Severity, d.Message))
	sb.WriteString(fmt.Sprintf("  --> %s\n", d.Location))

	// Context snippet
	if len(d.Context) > 0 {
		sb.WriteString("\n")
		for _, line := range d.Context {
			sb.WriteString(fmt.Sprintf("  | %s\n", line))
		}
		sb.WriteString("\n")
	}

	// Suggestions
	if len(d.Suggestions) > 0 {
		sb.WriteString("  = Help: " + strings.Join(d.Suggestions, "\n          ") + "\n")
	}

	return sb.String()
}
