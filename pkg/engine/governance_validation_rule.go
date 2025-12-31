package engine

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// GovernanceValidationRule validates governance features (requirements, ADRs, policies, etc.)
// for unique IDs and data quality.
type GovernanceValidationRule struct{}

// Name returns the name of this validation rule.
func (r *GovernanceValidationRule) Name() string {
	return "Governance Validation"
}

// governanceItem represents a parsed governance element with its location
type governanceItem struct {
	ID   string
	Kind string
	Pos  language.SourceLocation
}

// Validate checks governance features for:
// - Unique requirement IDs
// - Unique ADR IDs
// - Unique policy IDs
// - Unique scenario/flow IDs
// - Unique contract IDs
func (r *GovernanceValidationRule) Validate(prog *language.Program) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic

	if prog == nil || prog.Model == nil {
		return diags
	}

	model := prog.Model

	// Track IDs by kind
	seenIDs := make(map[string]map[string]governanceItem) // kind -> id -> item
	seenIDs["requirement"] = make(map[string]governanceItem)
	seenIDs["adr"] = make(map[string]governanceItem)
	seenIDs["policy"] = make(map[string]governanceItem)
	seenIDs["scenario"] = make(map[string]governanceItem)
	seenIDs["flow"] = make(map[string]governanceItem)

	seenIDs["contract"] = make(map[string]governanceItem)

	// Iterate through all model items to collect and validate IDs
	for _, item := range model.Items {
		// Governance elements are now parsed through ElementDef
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			kind := a.Kind

			// Check if this is a governance kind
			kindMap, isGovernance := seenIDs[kind]
			if !isGovernance {
				// Also check for "story" which maps to "scenario"
				if kind == "story" {
					kind = "scenario"
					kindMap = seenIDs["scenario"]
					isGovernance = true
				}
			}

			if isGovernance {
				id := a.Name
				if id == "" {
					continue
				}
				loc := item.ElementDef.Location()
				if existing, found := kindMap[id]; found {
					diags = append(diags, diagnostics.Diagnostic{
						Severity: diagnostics.SeverityError,
						Message:  fmt.Sprintf("Duplicate %s ID '%s'", kind, id),
						Location: diagnostics.SourceLocation{
							File:   loc.File,
							Line:   loc.Line,
							Column: loc.Column,
						},
						Code: diagnostics.CodeDuplicateIdentifier,
						Suggestions: []string{
							fmt.Sprintf("%s '%s' is already defined at line %d", kind, id, existing.Pos.Line),
							fmt.Sprintf("Use a unique ID for each %s", kind),
						},
					})
				} else {
					kindMap[id] = governanceItem{
						ID:   id,
						Kind: kind,
						Pos:  loc,
					}
				}
			}
		}
	}

	return diags
}
