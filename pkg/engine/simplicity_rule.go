package engine

import (
	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// SimplicityRule validates that users are using the right perspective
// (system vs domain) for their modeling goals, and suggests simpler alternatives.
type SimplicityRule struct{}

func (r *SimplicityRule) Name() string {
	return "SimplicityGuidance"
}

func (r *SimplicityRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Architecture == nil {
		return nil
	}
	// Pre-allocate diagnostics slice (currently empty but ready for future use)
	diags := make([]diagnostics.Diagnostic, 0, 0)

	// Domain/ContextBlock validation removed - DDD features, deferred to Phase 2
	// DomainBlock structure doesn't match expected fields (no Contexts, Components, etc.)
	// If DomainBlock is needed in future, implement proper structure then
	// for _, domain := range arch.Domains {
	// 	hasComplexDDD := false
	// 	hasSimpleContainers := false
	// 	if len(domain.Contexts) > 0 {
	// 		for _, ctx := range domain.Contexts {
	// 			if len(ctx.Aggregates) > 0 || len(ctx.Entities) > 0 || len(ctx.ValueObjects) > 0 {
	// 				hasComplexDDD = true
	// 				break
	// 			}
	// 		}
	// 	}
	// 	if len(domain.Components) > 0 && !hasComplexDDD {
	// 		hasSimpleContainers = true
	// 	}
	// 	if hasSimpleContainers && !hasComplexDDD {
	// 		errors = append(errors, ValidationError{...})
	// 	}
	// }

	// System.Contexts removed - DDD feature, deferred to Phase 2
	// for _, system := range arch.Systems {
	// 	hasContainers := len(system.Containers) > 0 || len(system.Components) > 0 || len(system.DataStores) > 0
	// 	hasDomainConcepts := false
	// 	if len(system.Contexts) > 0 {
	// 		for _, ctx := range system.Contexts {
	// 			if len(ctx.Aggregates) > 0 {
	// 				hasDomainConcepts = true
	// 				break
	// 			}
	// 		}
	// 	}
	// 	if hasDomainConcepts && !hasContainers {
	// 		errors = append(errors, ValidationError{...})
	// 	}
	// }

	return diags
}
