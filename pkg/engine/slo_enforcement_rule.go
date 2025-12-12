// pkg/engine/slo_enforcement_rule.go
package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// SLOEnforcementRule suggests SLO blocks when SLA requirements exist.
// This is a contextual enforcement rule that guides developers to add SLOs
// when they have SLA requirements, but doesn't force them.
type SLOEnforcementRule struct{}

func (r *SLOEnforcementRule) Name() string {
	return "SLO Enforcement"
}

// Validate suggests SLO blocks when systems have SLA requirements but no SLO block.
func (r *SLOEnforcementRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Architecture == nil {
		return nil
	}

	// Pre-allocate diagnostics slice
	estimatedDiags := len(program.Architecture.Systems) * 2
	if estimatedDiags < 8 {
		estimatedDiags = 8
	}
	diags := make([]diagnostics.Diagnostic, 0, estimatedDiags)

	arch := program.Architecture

	// Root-level policy: use architecture requirements to suggest SLOs
	hasSLA := r.archHasSLARequirement(arch)
	if hasSLA {
		for _, sys := range arch.Systems {
			if sys.SLO == nil {
				diags = append(diags, r.suggestSLO(sys, "system")...)
			}
			for _, cont := range sys.Containers {
				if cont.SLO == nil {
					diags = append(diags, r.suggestSLO(cont, "container")...)
				}
			}
		}
	}

	return diags
}

// hasSLARequirement checks if a system/container has SLA-related requirements.
func (r *SLOEnforcementRule) archHasSLARequirement(arch *language.Architecture) bool {
	for _, req := range arch.Requirements {
		if req.Type != nil {
			reqType := strings.ToLower(*req.Type)
			// Check for performance/availability requirements
			if reqType == "performance" || reqType == "nonfunctional" {
				if req.Description != nil {
					desc := strings.ToLower(*req.Description)
					// Look for SLA-related keywords
					if strings.Contains(desc, "sla") ||
						strings.Contains(desc, "availability") ||
						strings.Contains(desc, "uptime") ||
						strings.Contains(desc, "latency") ||
						strings.Contains(desc, "response time") ||
						strings.Contains(desc, "error rate") ||
						strings.Contains(desc, "throughput") {
						return true
					}
				}
			}
		}
	}

	return false
}

// suggestSLO creates a diagnostic suggesting to add an SLO block.
func (r *SLOEnforcementRule) suggestSLO(element interface{}, elementType string) []diagnostics.Diagnostic {
	var loc language.SourceLocation
	var name string

	switch e := element.(type) {
	case *language.System:
		loc = e.Location()
		name = e.ID
	case *language.Container:
		loc = e.Location()
		name = e.ID
	default:
		return nil
	}

	return []diagnostics.Diagnostic{
		{
			Code:     diagnostics.CodeBestPractice,
			Severity: diagnostics.SeverityInfo, // Info-level suggestion, not error
			Message:  fmt.Sprintf("%s '%s' has SLA requirements but no SLO block defined. Consider adding 'slo { ... }' block to document service level objectives.", elementType, name),
			Location: diagnostics.SourceLocation{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
			Suggestions: []string{
				"Add SLO block to document service level objectives",
				"Example: slo { availability { target \"99.9%\" window \"30 days\" } }",
			},
		},
	}
}
