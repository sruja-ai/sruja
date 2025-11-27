// pkg/kernel/diagnostics.go
// Package kernel provides enhanced diagnostics and error handling.
package kernel

import (
	"fmt"
	"sort"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/model"
)

// DiagnosticSeverity represents the severity of a diagnostic.
type DiagnosticSeverity string

const (
	SeverityError   DiagnosticSeverity = "error"
	SeverityWarning DiagnosticSeverity = "warning"
	SeverityInfo    DiagnosticSeverity = "info"
	SeverityHint    DiagnosticSeverity = "hint"
)

// collectDiagnostics collects diagnostics from various sources and formats them.
func (k *Kernel) collectDiagnostics(validationErrors []engine.ValidationError, parseError error) []Diagnostic {
	var diagnostics []Diagnostic

	// Add parse error if present
	if parseError != nil {
		diagnostics = append(diagnostics, Diagnostic{
			Severity: string(SeverityError),
			Message:  fmt.Sprintf("Parse error: %v", parseError),
		})
	}

	// Add validation errors
	for _, ve := range validationErrors {
		diagnostics = append(diagnostics, k.convertValidationError(ve))
	}

	// Sort diagnostics by severity (errors first)
	sort.Slice(diagnostics, func(i, j int) bool {
		severityOrder := map[string]int{
			"error":   0,
			"warning": 1,
			"info":    2,
			"hint":    3,
		}
		return severityOrder[diagnostics[i].Severity] < severityOrder[diagnostics[j].Severity]
	})

	return diagnostics
}

// convertValidationError converts engine.ValidationError to Diagnostic.
func (k *Kernel) convertValidationError(ve engine.ValidationError) Diagnostic {
	diag := Diagnostic{
		Severity: string(SeverityError),
		Message:  ve.Message,
	}

	// Add location if available
	if ve.Line > 0 {
		diag.Location = &model.Location{
			File:   "", // File will be set from cell ID
			Line:   ve.Line,
			Column: ve.Column,
		}
	}

	return diag
}

// groupDiagnosticsByElement groups diagnostics by element ID.
func groupDiagnosticsByElement(diagnostics []Diagnostic) map[string][]Diagnostic {
	grouped := make(map[string][]Diagnostic)

	for _, diag := range diagnostics {
		key := diag.ElementID
		if key == "" {
			key = "_general"
		}
		grouped[key] = append(grouped[key], diag)
	}

	return grouped
}

// filterDiagnosticsBySeverity filters diagnostics by severity.
func filterDiagnosticsBySeverity(diagnostics []Diagnostic, severity DiagnosticSeverity) []Diagnostic {
	var filtered []Diagnostic
	for _, diag := range diagnostics {
		if DiagnosticSeverity(diag.Severity) == severity {
			filtered = append(filtered, diag)
		}
	}
	return filtered
}

// countDiagnosticsBySeverity counts diagnostics by severity.
func countDiagnosticsBySeverity(diagnostics []Diagnostic) map[string]int {
	counts := make(map[string]int)
	for _, diag := range diagnostics {
		counts[diag.Severity]++
	}
	return counts
}
