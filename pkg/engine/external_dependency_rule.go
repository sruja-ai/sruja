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

func (r *ExternalDependencyRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Model == nil {
		return nil
	}

	// Collect all defined elements (map[fqn]Def) and relations
	defined, relations := collectElements(program.Model)

	// Pre-allocate diagnostics slice
	estimatedDiags := len(relations) / 10
	if estimatedDiags < 8 {
		estimatedDiags = 8
	}
	diags := make([]diagnostics.Diagnostic, 0, estimatedDiags)

	// Helper to get parent ID from FQN
	getParentID := func(fqn string) string {
		if idx := strings.LastIndexByte(fqn, '.'); idx != -1 {
			return fqn[:idx]
		}
		return ""
	}

	// Resolve unqualified ID to qualified ID within scope
	resolve := func(ref, scope string) string {
		// 1. Try absolute/global match
		if defined[ref] != nil {
			return ref
		}

		// 2. Try relative to scope, walking up
		if scope != "" {
			candidate := scope + "." + ref
			if defined[candidate] != nil {
				return candidate
			}

			parts := strings.Split(scope, ".")
			for i := len(parts) - 1; i >= 0; i-- {
				prefix := strings.Join(parts[:i], ".")
				var candidate string
				if prefix == "" {
					candidate = ref
				} else {
					candidate = prefix + "." + ref
				}
				if defined[candidate] != nil {
					return candidate
				}
			}
		}

		// 3. For architecture-level relations (scope=""), search all defined elements
		if scope == "" {
			for id := range defined {
				if strings.HasSuffix(id, "."+ref) || id == ref {
					// Conservative match: ensure suffix matches and preceeded by dot or is exact match
					parts := strings.Split(id, ".")
					if len(parts) > 0 && parts[len(parts)-1] == ref {
						return id
					}
				}
			}
		}

		return ref // Return original if not resolved
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
		parentID := getParentID(fromID)
		if parentID != "" && parentID == toID {
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

	return diags
}
