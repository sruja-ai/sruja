// pkg/engine/slo_enforcement_rule.go
package engine

import (
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
	if program == nil || program.Model == nil {
		return nil
	}

	// Pre-allocate diagnostics slice
	diags := make([]diagnostics.Diagnostic, 0, 8)

	// Check if there are SLA-related requirements in the model
	hasSLA := false
	for _, item := range program.Model.Items {
		if item.Requirement != nil {
			if item.Requirement.Type != nil {
				reqType := strings.ToLower(*item.Requirement.Type)
				if reqType == "performance" || reqType == "nonfunctional" {
					if item.Requirement.Description != nil {
						desc := strings.ToLower(*item.Requirement.Description)
						if strings.Contains(desc, "sla") ||
							strings.Contains(desc, "availability") ||
							strings.Contains(desc, "uptime") ||
							strings.Contains(desc, "latency") {
							hasSLA = true
							break
						}
					}
				}
			}
		}
	}

	// SLO enforcement for LikeC4 elements would need to check metadata
	// For now, return empty - SLO blocks are not yet supported in LikeC4 syntax
	// TODO: Add SLO support to LikeC4 Model block if needed

	_ = hasSLA // Suppress unused variable warning

	return diags
}
