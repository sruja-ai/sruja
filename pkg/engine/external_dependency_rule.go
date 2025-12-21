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
	if program == nil || program.Model == nil {
		return nil
	}

	// Collect all elements from LikeC4 Model
	defined, relations := collectLikeC4Elements(program.Model)

	// Pre-allocate diagnostics slice
	estimatedDiags := len(relations) / 10
	if estimatedDiags < 8 {
		estimatedDiags = 8
	}
	diags := make([]diagnostics.Diagnostic, 0, estimatedDiags)

	// Build parent map: child ID -> parent ID
	parent := make(map[string]string, len(defined))

	// Build parent relationships from LikeC4 elements
	var buildParentMap func(elem *language.LikeC4ElementDef, parentFQN string)
	buildParentMap = func(elem *language.LikeC4ElementDef, parentFQN string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		fqn := id
		if parentFQN != "" {
			fqn = buildQualifiedID(parentFQN, id)
			parent[fqn] = parentFQN
		}

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					buildParentMap(bodyItem.Element, fqn)
				}
			}
		}
	}

	// Build parent map from all top-level elements
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			buildParentMap(item.ElementDef, "")
		}
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

	// Check all relations with their scopes
	relationsWithScope := collectAllRelations(program.Model)

	for _, relScope := range relationsWithScope {
		rel := relScope.Relation
		scope := relScope.Scope

		fromStr := rel.From.String()
		toStr := rel.To.String()

		// Resolve IDs to their fully qualified names
		fromID := resolve(fromStr, scope)
		toID := resolve(toStr, scope)

		// Check if 'from' is a child and 'to' is its parent
		if parentID, isChild := parent[fromID]; isChild {
			if parentID == toID {
				// Use original unqualified names for error message
				loc := rel.Location()
				diags = append(diags, diagnostics.Diagnostic{
					Code:     diagnostics.CodeValidationRuleError,
					Severity: diagnostics.SeverityError,
					Message:  fmt.Sprintf("Element '%s' cannot depend on its parent '%s'. Dependencies must be external.", fromStr, toStr),
					Suggestions: []string{
						fmt.Sprintf("Remove the dependency from '%s' to '%s'", fromStr, toStr),
						"If this is an external dependency, ensure the parent is marked as external",
						"Consider restructuring to avoid parent-child dependencies",
					},
					Location: diagnostics.SourceLocation{
						File:   loc.File,
						Line:   loc.Line,
						Column: loc.Column,
					},
				})
			}
		}
	}

	return diags
}
