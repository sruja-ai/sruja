// pkg/engine/slo_rule.go
package engine

import (
	"fmt"
	"regexp"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// SLOValidationRule validates SLO block syntax and values.
type SLOValidationRule struct{}

func (r *SLOValidationRule) Name() string {
	return "SLO Validation"
}

// Validate checks SLO blocks for valid formats and values.
func (r *SLOValidationRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Model == nil {
		return nil
	}
	// Pre-allocate diagnostics slice with estimated capacity
	diags := make([]diagnostics.Diagnostic, 0, 10)

	var validateInElement func(elem *language.ElementDef)
	validateInElement = func(elem *language.ElementDef) {
		body := elem.GetBody()
		if body == nil {
			return
		}

		for _, item := range body.Items {
			if item.SLO != nil {
				diags = append(diags, r.validateSLOBlock(item.SLO, item.SLO.Location())...)
			}
			if item.Element != nil {
				validateInElement(item.Element)
			}
		}
	}

	// Process top-level items
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			validateInElement(item.ElementDef)
		}
	}

	return diags
}

func (r *SLOValidationRule) validateSLOBlock(slo *language.SLOBlock, loc language.SourceLocation) []diagnostics.Diagnostic {
	// Pre-allocate diagnostics slice with estimated capacity
	estimatedDiags := 5
	diags := make([]diagnostics.Diagnostic, 0, estimatedDiags)

	// Validate availability SLO
	if slo.Availability != nil {
		diags = append(diags, r.validateAvailability(slo.Availability, loc)...)
	}

	// Validate latency SLO
	if slo.Latency != nil {
		diags = append(diags, r.validateLatency(slo.Latency, loc)...)
	}

	// Validate error rate SLO
	if slo.ErrorRate != nil {
		diags = append(diags, r.validateErrorRate(slo.ErrorRate, loc)...)
	}

	// Validate throughput SLO
	if slo.Throughput != nil {
		diags = append(diags, r.validateThroughput(slo.Throughput, loc)...)
	}

	// Check if at least one SLO type is defined
	if slo.Availability == nil && slo.Latency == nil && slo.ErrorRate == nil && slo.Throughput == nil {
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeValidationRuleError,
			Severity: diagnostics.SeverityWarning,
			Message:  "SLO block should define at least one SLO type (availability, latency, errorRate, or throughput)",
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	return diags
}

func (r *SLOValidationRule) validateAvailability(avail *language.SLOAvailability, loc language.SourceLocation) []diagnostics.Diagnostic {
	// Pre-allocate diagnostics slice with estimated capacity
	estimatedDiags := 3
	diags := make([]diagnostics.Diagnostic, 0, estimatedDiags)

	// Validate target format (should be percentage like "99.9%" or "99.99%")
	if avail.Target != nil && !r.isValidPercentage(*avail.Target) {
		// Build message efficiently
		var msgSb strings.Builder
		msgSb.Grow(len(*avail.Target) + 60)
		msgSb.WriteString("Availability target '")
		msgSb.WriteString(*avail.Target)
		msgSb.WriteString("' must be a percentage (e.g., '99.9%')")
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeValidationRuleError,
			Severity: diagnostics.SeverityError,
			Message:  msgSb.String(),
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	// Validate window format (should be time period)
	if avail.Window != nil && !r.isValidTimeWindow(*avail.Window) {
		var msgSb strings.Builder
		msgSb.Grow(len(*avail.Window) + 70)
		msgSb.WriteString("Availability window '")
		msgSb.WriteString(*avail.Window)
		msgSb.WriteString("' should be a time period (e.g., '30 days', '7 days')")
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeValidationRuleError,
			Severity: diagnostics.SeverityWarning,
			Message:  msgSb.String(),
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	// Validate current if present
	if avail.Current != nil && !r.isValidPercentage(*avail.Current) {
		var msgSb strings.Builder
		msgSb.Grow(len(*avail.Current) + 60)
		msgSb.WriteString("Availability current '")
		msgSb.WriteString(*avail.Current)
		msgSb.WriteString("' must be a percentage (e.g., '99.95%')")
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeValidationRuleError,
			Severity: diagnostics.SeverityError,
			Message:  msgSb.String(),
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	return diags
}

func (r *SLOValidationRule) validateLatency(latency *language.SLOLatency, loc language.SourceLocation) []diagnostics.Diagnostic {
	// Pre-allocate diagnostics slice with estimated capacity
	estimatedDiags := 5
	diags := make([]diagnostics.Diagnostic, 0, estimatedDiags)

	// Validate p95 format (should be duration like "200ms", "1s", "500ms")
	if latency.P95 != nil && !r.isValidDuration(*latency.P95) {
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeValidationRuleError,
			Severity: diagnostics.SeverityError,
			Message:  fmt.Sprintf("Latency p95 '%s' must be a duration (e.g., '200ms', '1s')", *latency.P95),
			Suggestions: []string{
				"Use duration format: '200ms', '1s', '500ms', '2s'",
				"Common values: '100ms' (fast), '200ms' (good), '500ms' (acceptable)",
			},
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	// Validate p99 format
	if latency.P99 != nil && !r.isValidDuration(*latency.P99) {
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeValidationRuleError,
			Severity: diagnostics.SeverityError,
			Message:  fmt.Sprintf("Latency p99 '%s' must be a duration (e.g., '500ms', '2s')", *latency.P99),
			Suggestions: []string{
				"Use duration format: '500ms', '2s', '1s', '3s'",
				"p99 should typically be 2-5x higher than p95",
			},
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	// Validate window
	if latency.Window != nil && !r.isValidTimeWindow(*latency.Window) {
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeValidationRuleError,
			Severity: diagnostics.SeverityWarning,
			Message:  fmt.Sprintf("Latency window '%s' should be a time period (e.g., '7 days', '30 days')", *latency.Window),
			Suggestions: []string{
				"Use time period format: '7 days', '30 days', '1 week', '1 month'",
				"Common windows: '7 days' (weekly), '30 days' (monthly)",
			},
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	// Validate current if present
	if latency.Current != nil {
		if latency.Current.P95 != nil && !r.isValidDuration(*latency.Current.P95) {
			diags = append(diags, diagnostics.Diagnostic{
				Code:        diagnostics.CodeValidationRuleError,
				Severity:    diagnostics.SeverityError,
				Message:     fmt.Sprintf("Latency current p95 '%s' must be a duration", *latency.Current.P95),
				Suggestions: []string{"Use duration format: '200ms', '1s', '500ms'"},
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
			})
		}
		if latency.Current.P99 != nil && !r.isValidDuration(*latency.Current.P99) {
			diags = append(diags, diagnostics.Diagnostic{
				Code:        diagnostics.CodeValidationRuleError,
				Severity:    diagnostics.SeverityError,
				Message:     fmt.Sprintf("Latency current p99 '%s' must be a duration", *latency.Current.P99),
				Suggestions: []string{"Use duration format: '500ms', '2s', '1s'"},
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
			})
		}
	}

	return diags
}

func (r *SLOValidationRule) validateErrorRate(er *language.SLOErrorRate, loc language.SourceLocation) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic

	// Validate target format (should be percentage)
	if er.Target != nil && !r.isValidPercentage(*er.Target) {
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeValidationRuleError,
			Severity: diagnostics.SeverityError,
			Message:  fmt.Sprintf("Error rate target '%s' must be a percentage (e.g., '0.1%%')", *er.Target),
			Suggestions: []string{
				"Use percentage format: '0.1%', '0.01%', '1%'",
				"Common targets: '0.1%' (99.9% success), '0.01%' (99.99% success)",
			},
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	// Validate window
	if er.Window != nil && !r.isValidTimeWindow(*er.Window) {
		diags = append(diags, diagnostics.Diagnostic{
			Code:        diagnostics.CodeValidationRuleError,
			Severity:    diagnostics.SeverityWarning,
			Message:     fmt.Sprintf("Error rate window '%s' should be a time period", *er.Window),
			Suggestions: []string{"Use time period format: '7 days', '30 days', '1 week'"},
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	// Validate current if present
	if er.Current != nil && !r.isValidPercentage(*er.Current) {
		diags = append(diags, diagnostics.Diagnostic{
			Code:        diagnostics.CodeValidationRuleError,
			Severity:    diagnostics.SeverityError,
			Message:     fmt.Sprintf("Error rate current '%s' must be a percentage", *er.Current),
			Suggestions: []string{"Use percentage format: '0.1%', '0.5%', '1%'"},
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	return diags
}

func (r *SLOValidationRule) validateThroughput(tp *language.SLOThroughput, loc language.SourceLocation) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic

	// Validate target format (should be rate like "10000 req/s", "1000/s")
	if tp.Target != nil && !r.isValidRate(*tp.Target) {
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeValidationRuleError,
			Severity: diagnostics.SeverityWarning,
			Message:  fmt.Sprintf("Throughput target '%s' should be a rate (e.g., '10000 req/s', '1000/s')", *tp.Target),
			Suggestions: []string{
				"Use rate format: '10000 req/s', '1000/s', '500 ops/min'",
				"Specify units: 'req/s', '/s', 'ops/min', 'requests/hour'",
			},
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	// Validate window
	if tp.Window != nil && !r.isValidTimeWindow(*tp.Window) {
		diags = append(diags, diagnostics.Diagnostic{
			Code:        diagnostics.CodeValidationRuleError,
			Severity:    diagnostics.SeverityWarning,
			Message:     fmt.Sprintf("Throughput window '%s' should be a time period", *tp.Window),
			Suggestions: []string{"Use time period format: '1 hour', '1 day', '1 week'"},
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	// Validate current if present
	if tp.Current != nil && !r.isValidRate(*tp.Current) {
		diags = append(diags, diagnostics.Diagnostic{
			Code:        diagnostics.CodeValidationRuleError,
			Severity:    diagnostics.SeverityWarning,
			Message:     fmt.Sprintf("Throughput current '%s' should be a rate", *tp.Current),
			Suggestions: []string{"Use rate format: '5000 req/s', '1000/s', '200 ops/min'"},
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		})
	}

	return diags
}

var (
	percentageRegex = regexp.MustCompile(`^\d+(\.\d+)?%$`)
	durationRegex   = regexp.MustCompile(`^\d+(\.\d+)?(ms|s|m|h)$`)
	timeWindowRegex = regexp.MustCompile(`^\d+\s+(day|days|hour|hours|week|weeks|month|months)$`)
	rateRegex       = regexp.MustCompile(`^\d+(\s+\w+)?/\w+$`)
)

// isValidPercentage checks if a string is a valid percentage (e.g., "99.9%", "99.99%")
func (r *SLOValidationRule) isValidPercentage(s string) bool {
	return percentageRegex.MatchString(s)
}

// isValidDuration checks if a string is a valid duration (e.g., "200ms", "1s", "500ms")
func (r *SLOValidationRule) isValidDuration(s string) bool {
	return durationRegex.MatchString(s)
}

// isValidTimeWindow checks if a string is a valid time window (e.g., "30 days", "7 days", "1 hour")
func (r *SLOValidationRule) isValidTimeWindow(s string) bool {
	return timeWindowRegex.MatchString(strings.ToLower(s))
}

// isValidRate checks if a string is a valid rate (e.g., "10000 req/s", "1000/s")
func (r *SLOValidationRule) isValidRate(s string) bool {
	return rateRegex.MatchString(s)
}
