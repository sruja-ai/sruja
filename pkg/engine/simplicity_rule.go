package engine

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

// SimplicityRule validates that users are using the right perspective
// (system vs domain) for their modeling goals, and suggests simpler alternatives.
type SimplicityRule struct{}

func (r *SimplicityRule) Name() string {
	return "SimplicityGuidance"
}

func (r *SimplicityRule) Validate(program *language.Program) []ValidationError {
	errors := []ValidationError{}

	arch := program.Architecture
	if arch == nil {
		return errors
	}

	// Check if domain is used for simple deployment modeling
	// A domain used only with containers/components (no aggregates/contexts) might be better as a system
	for _, domain := range arch.Domains {
		hasComplexDDD := false
		hasSimpleContainers := false

		// Check if domain has DDD constructs (contexts, aggregates, entities, valueObjects)
		if len(domain.Contexts) > 0 {
			for _, ctx := range domain.Contexts {
				if len(ctx.Aggregates) > 0 || len(ctx.Entities) > 0 || len(ctx.ValueObjects) > 0 {
					hasComplexDDD = true
					break
				}
			}
		}

		// Check if domain only has simple containers/components (deployment-focused)
		if len(domain.Components) > 0 && !hasComplexDDD {
			hasSimpleContainers = true
		}

		// Warn if domain is used for simple deployment modeling
		if hasSimpleContainers && !hasComplexDDD {
			errors = append(errors, ValidationError{
				Message: fmt.Sprintf(
					"💡 Domain '%s' appears to model deployment/technology. Consider using 'system' instead for deployment modeling. Use 'domain' for business logic and bounded contexts. Example: system %s { container WebApp container Database }",
					domain.ID, domain.ID,
				),
				Line:   0, // TODO: Get actual line from AST
				Column: 0,
			})
		}
	}

	// Check if system is used for pure domain modeling
	// A system with only aggregates/entities (no containers/components) might be better as a domain
	for _, system := range arch.Systems {
		hasContainers := len(system.Containers) > 0 || len(system.Components) > 0 || len(system.DataStores) > 0
		hasDomainConcepts := false

		// Check if system has domain concepts (contexts, aggregates)
		// Note: Systems can have contexts, but if they ONLY have contexts/aggregates, it's domain modeling
		if len(system.Contexts) > 0 {
			for _, ctx := range system.Contexts {
				if len(ctx.Aggregates) > 0 {
					hasDomainConcepts = true
					break
				}
			}
		}

		// Warn if system is used for pure domain modeling without deployment concerns
		if hasDomainConcepts && !hasContainers {
			errors = append(errors, ValidationError{
				Message: fmt.Sprintf(
					"💡 System '%s' appears to model business domain concepts only. Consider using 'domain' instead for domain modeling. Use 'system' for deployment and technology. Example: domain %s { context Orders { aggregate Order } }",
					system.ID, system.ID,
				),
				Line:   0, // TODO: Get actual line from AST
				Column: 0,
			})
		}
	}

	return errors
}

