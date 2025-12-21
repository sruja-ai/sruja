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

	// Validate unique requirement IDs
	requirementIDs := make(map[string]*language.Requirement)
	adrIDs := make(map[string]*language.ADR)
	policyIDs := make(map[string]*language.Policy)
	scenarioIDs := make(map[string]*language.Scenario)
	flowIDs := make(map[string]*language.Flow)
	contractIDs := make(map[string]*language.Contract)

	// Iterate through all model items to collect and validate IDs
	for _, item := range model.Items {
		// Validate unique requirement IDs
		if item.Requirement != nil {
			req := item.Requirement
			if existing, found := requirementIDs[req.ID]; found {
				diags = append(diags, diagnostics.Diagnostic{
					Severity: diagnostics.SeverityError,
					Message:  fmt.Sprintf("Duplicate requirement ID '%s'", req.ID),
					Location: diagnostics.SourceLocation{
						File:   req.Location().File,
						Line:   req.Location().Line,
						Column: req.Location().Column,
					},
					Code: diagnostics.CodeDuplicateIdentifier,
					Suggestions: []string{
						fmt.Sprintf("Requirement '%s' is already defined at line %d", req.ID, existing.Location().Line),
						"Use a unique ID for each requirement",
					},
				})
			} else {
				requirementIDs[req.ID] = req
			}
		}

		// Validate unique ADR IDs
		if item.ADR != nil {
			adr := item.ADR
			if existing, found := adrIDs[adr.ID]; found {
				loc := adr.Location()
				diags = append(diags, diagnostics.Diagnostic{
					Severity: diagnostics.SeverityError,
					Message:  fmt.Sprintf("Duplicate ADR ID '%s'", adr.ID),
					Location: diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column},
					Code:     "duplicate-adr-id",
					Suggestions: []string{
						fmt.Sprintf("ADR '%s' is already defined at %s", adr.ID, existing.Location()),
						"Use a unique ID for each ADR",
					},
				})
			} else {
				adrIDs[adr.ID] = adr
			}
		}

		// Validate unique policy IDs
		if item.Policy != nil {
			policy := item.Policy
			if existing, found := policyIDs[policy.ID]; found {
				loc := policy.Location()
				diags = append(diags, diagnostics.Diagnostic{
					Severity: diagnostics.SeverityError,
					Message:  fmt.Sprintf("Duplicate policy ID '%s'", policy.ID),
					Location: diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column},
					Code:     "duplicate-policy-id",
					Suggestions: []string{
						fmt.Sprintf("Policy '%s' is already defined at %s", policy.ID, existing.Location()),
						"Use a unique ID for each policy",
					},
				})
			} else {
				policyIDs[policy.ID] = policy
			}
		}

		// Validate unique scenario IDs
		if item.Scenario != nil {
			scenario := item.Scenario
			if scenario.ID == "" {
				continue
			}
			loc := scenario.Location()
			if existing, found := scenarioIDs[scenario.ID]; found {
				diags = append(diags, diagnostics.Diagnostic{
					Severity: diagnostics.SeverityError,
					Message:  fmt.Sprintf("Duplicate scenario ID '%s'", scenario.ID),
					Location: diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column},
					Code:     "duplicate-scenario-id",
					Suggestions: []string{
						fmt.Sprintf("Scenario '%s' is already defined at %s", scenario.ID, existing.Location()),
						"Use a unique ID for each scenario",
					},
				})
			} else {
				scenarioIDs[scenario.ID] = scenario
			}
		}

		// Validate unique flow IDs
		if item.Flow != nil {
			flow := item.Flow
			if flow.ID == "" {
				continue
			}
			loc := flow.Location()
			if existing, found := flowIDs[flow.ID]; found {
				diags = append(diags, diagnostics.Diagnostic{
					Severity: diagnostics.SeverityError,
					Message:  fmt.Sprintf("Duplicate flow ID '%s'", flow.ID),
					Location: diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column},
					Code:     "duplicate-flow-id",
					Suggestions: []string{
						fmt.Sprintf("Flow '%s' is already defined at %s", flow.ID, existing.Location()),
						"Use a unique ID for each flow",
					},
				})
			} else {
				flowIDs[flow.ID] = flow
			}
		}

		// Validate unique contract IDs
		if item.ContractsBlock != nil {
			for _, contract := range item.ContractsBlock.Contracts {
				if contract == nil {
					continue
				}
				loc := contract.Location()
				if existing, found := contractIDs[contract.ID]; found {
					diags = append(diags, diagnostics.Diagnostic{
						Severity: diagnostics.SeverityError,
						Message:  fmt.Sprintf("Duplicate contract ID '%s'", contract.ID),
						Location: diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column},
						Code:     "duplicate-contract-id",
						Suggestions: []string{
							fmt.Sprintf("Contract '%s' is already defined at %s", contract.ID, existing.Location()),
							"Use a unique ID for each contract",
						},
					})
				} else {
					contractIDs[contract.ID] = contract
				}
			}
		}
	}

	return diags
}
