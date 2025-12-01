// pkg/engine/external_dependency_rule.go
package engine

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

// ExternalDependencyRule validates that child elements (containers, components, etc.)
// cannot depend on their parent elements (systems, containers).
//
// This rule enforces the C4 model principle that dependencies should be external
// (to other systems/containers), not internal (to parent elements).
type ExternalDependencyRule struct{}

func (r *ExternalDependencyRule) Name() string {
	return "ExternalDependency"
}

func (r *ExternalDependencyRule) Validate(program *language.Program) []ValidationError {
	errors := []ValidationError{}

	arch := program.Architecture
	if arch == nil {
		return errors
	}

	// Build parent map: child ID -> parent ID
	parent := map[string]string{}

	// Track all defined elements
	defined := map[string]language.Element{}

	// Build parent relationships and element map
	for _, sys := range arch.Systems {
		defined[sys.ID] = sys
		for _, cont := range sys.Containers {
			defined[cont.ID] = cont
			parent[cont.ID] = sys.ID
			for _, comp := range cont.Components {
				defined[comp.ID] = comp
				parent[comp.ID] = cont.ID
			}
			for _, ds := range cont.DataStores {
				defined[ds.ID] = ds
				parent[ds.ID] = cont.ID
			}
			for _, q := range cont.Queues {
				defined[q.ID] = q
				parent[q.ID] = cont.ID
			}
		}
		for _, comp := range sys.Components {
			defined[comp.ID] = comp
			parent[comp.ID] = sys.ID
		}
		for _, ds := range sys.DataStores {
			defined[ds.ID] = ds
			parent[ds.ID] = sys.ID
		}
		for _, q := range sys.Queues {
			defined[q.ID] = q
			parent[q.ID] = sys.ID
		}
	}

	// Check all relations
	checkRelation := func(rel *language.Relation, location language.SourceLocation) {
		if rel == nil {
			return
		}

		fromID := rel.From
		toID := rel.To

		// Check if 'from' is a child and 'to' is its parent
		if parentID, isChild := parent[fromID]; isChild {
			if parentID == toID {
				errors = append(errors, ValidationError{
					Message: fmt.Sprintf("Element '%s' cannot depend on its parent '%s'. Dependencies must be external.", fromID, toID),
					Line:    location.Line,
					Column:  location.Column,
				})
			}
		}
	}

	// Check architecture-level relations
	for _, rel := range arch.Relations {
		checkRelation(rel, rel.Location())
	}

	// Check system-level relations
	for _, sys := range arch.Systems {
		for _, rel := range sys.Relations {
			checkRelation(rel, rel.Location())
		}
		// Check container relations
		for _, cont := range sys.Containers {
			for _, rel := range cont.Relations {
				checkRelation(rel, rel.Location())
			}
			// Check component relations
			for _, comp := range cont.Components {
				for _, rel := range comp.Relations {
					checkRelation(rel, rel.Location())
				}
			}
		}
		// Check component relations at system level
		for _, comp := range sys.Components {
			for _, rel := range comp.Relations {
				checkRelation(rel, rel.Location())
			}
		}
	}

	return errors
}


