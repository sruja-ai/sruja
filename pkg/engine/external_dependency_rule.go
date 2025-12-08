// pkg/engine/external_dependency_rule.go
package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
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

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *ExternalDependencyRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic

	arch := program.Architecture
	if arch == nil {
		return diags
	}

	// Build parent map: child ID -> parent ID
	parent := map[string]string{}

	// Build defined map for resolution (boolean map for quick lookup)
	defined := map[string]bool{}

	// Build parent relationships and defined elements map
	for _, sys := range arch.Systems {
		defined[sys.ID] = true
		for _, cont := range sys.Containers {
			contID := sys.ID + "." + cont.ID
			defined[contID] = true
			parent[contID] = sys.ID
			for _, comp := range cont.Components {
				compID := contID + "." + comp.ID
				defined[compID] = true
				parent[compID] = contID
			}
			for _, ds := range cont.DataStores {
				dsID := contID + "." + ds.ID
				defined[dsID] = true
				parent[dsID] = contID
			}
			for _, q := range cont.Queues {
				qID := contID + "." + q.ID
				defined[qID] = true
				parent[qID] = contID
			}
		}
		for _, comp := range sys.Components {
			compID := sys.ID + "." + comp.ID
			defined[compID] = true
			parent[compID] = sys.ID
		}
		for _, ds := range sys.DataStores {
			dsID := sys.ID + "." + ds.ID
			defined[dsID] = true
			parent[dsID] = sys.ID
		}
		for _, q := range sys.Queues {
			qID := sys.ID + "." + q.ID
			defined[qID] = true
			parent[qID] = sys.ID
		}
	}
	for _, p := range arch.Persons {
		defined[p.ID] = true
	}

	// Resolve unqualified ID to qualified ID within scope
	resolve := func(ref, scope string) string {
		refStr := ref
		// 1. Try absolute/global match
		if defined[refStr] {
			return refStr
		}

		// 2. Try relative to scope, walking up
		if scope != "" {
			candidate := scope + "." + refStr
			if defined[candidate] {
				return candidate
			}

			parts := strings.Split(scope, ".")
			for i := len(parts) - 1; i >= 0; i-- {
				prefix := strings.Join(parts[:i], ".")
				var candidate string
				if prefix == "" {
					candidate = refStr
				} else {
					candidate = prefix + "." + refStr
				}
				if defined[candidate] {
					return candidate
				}
			}
		}

		// 3. For architecture-level relations (scope=""), search all defined elements
		// to find matching IDs (e.g., "API" matches "App.API")
		if scope == "" {
			for id := range defined {
				parts := strings.Split(id, ".")
				if len(parts) > 0 && parts[len(parts)-1] == refStr {
					return id
				}
			}
		}

		return refStr // Return original if not resolved
	}

	// Check all relations
	checkRelation := func(rel *language.Relation, location language.SourceLocation, scope string) {
		if rel == nil {
			return
		}

		fromStr := rel.From.String()
		toStr := rel.To.String()

		// Resolve IDs to their fully qualified names
		fromID := resolve(fromStr, scope)
		toID := resolve(toStr, scope)

		// Check if 'from' is a child and 'to' is its parent
		if parentID, isChild := parent[fromID]; isChild {
			if parentID == toID {
				// Use original unqualified names for error message
				diags = append(diags, diagnostics.Diagnostic{
					Code:     diagnostics.CodeValidationRuleError,
					Severity: diagnostics.SeverityError,
					Message:  fmt.Sprintf("Element '%s' cannot depend on its parent '%s'. Dependencies must be external.", fromStr, toStr),
					Location: diagnostics.SourceLocation{
						File:   location.File,
						Line:   location.Line,
						Column: location.Column,
					},
				})
			}
		}
	}

	// Check architecture-level relations
	for _, rel := range arch.Relations {
		checkRelation(rel, rel.Location(), "")
	}

	// Check system-level relations
	for _, sys := range arch.Systems {
		for _, rel := range sys.Relations {
			checkRelation(rel, rel.Location(), sys.ID)
		}
		// Check container relations
		for _, cont := range sys.Containers {
			contID := sys.ID + "." + cont.ID
			for _, rel := range cont.Relations {
				checkRelation(rel, rel.Location(), contID)
			}
			// Check component relations
			for _, comp := range cont.Components {
				compID := contID + "." + comp.ID
				for _, rel := range comp.Relations {
					checkRelation(rel, rel.Location(), compID)
				}
			}
		}
		// Check component relations at system level
		for _, comp := range sys.Components {
			compID := sys.ID + "." + comp.ID
			for _, rel := range comp.Relations {
				checkRelation(rel, rel.Location(), compID)
			}
		}
	}

	return diags
}
