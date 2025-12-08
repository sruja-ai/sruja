package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

type OrphanDetectionRule struct{}

func (r *OrphanDetectionRule) Name() string {
	return "OrphanDetection"
}

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *OrphanDetectionRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic
	definedLoc := map[string]language.SourceLocation{}
	defined := map[string]bool{}
	used := map[string]bool{}

	if program == nil {
		return diags
	}
	arch := program.Architecture
	if arch == nil {
		return diags
	}

	parent := map[string]string{}

	for _, sys := range arch.Systems {
		defined[sys.ID] = true
		definedLoc[sys.ID] = sys.Location()
		if len(sys.Requirements) > 0 || len(sys.ADRs) > 0 || len(sys.Contracts) > 0 {
			used[sys.ID] = true
		}
		for _, cont := range sys.Containers {
			contID := sys.ID + "." + cont.ID
			defined[contID] = true
			definedLoc[contID] = cont.Location()
			parent[contID] = sys.ID
			if len(cont.Requirements) > 0 || len(cont.ADRs) > 0 || len(cont.Contracts) > 0 {
				used[contID] = true
			}
			for _, comp := range cont.Components {
				compID := contID + "." + comp.ID
				defined[compID] = true
				definedLoc[compID] = comp.Location()
				parent[compID] = contID
				if len(comp.Requirements) > 0 || len(comp.ADRs) > 0 {
					used[compID] = true
				}
			}
			for _, ds := range cont.DataStores {
				dsID := contID + "." + ds.ID
				defined[dsID] = true
				definedLoc[dsID] = ds.Location()
				parent[dsID] = contID
			}
			for _, q := range cont.Queues {
				qID := contID + "." + q.ID
				defined[qID] = true
				definedLoc[qID] = q.Location()
				parent[qID] = contID
			}
		}
		for _, comp := range sys.Components {
			compID := sys.ID + "." + comp.ID
			defined[compID] = true
			definedLoc[compID] = comp.Location()
			parent[compID] = sys.ID
			if len(comp.Requirements) > 0 || len(comp.ADRs) > 0 {
				used[compID] = true
			}
		}
		for _, ds := range sys.DataStores {
			dsID := sys.ID + "." + ds.ID
			defined[dsID] = true
			definedLoc[dsID] = ds.Location()
			parent[dsID] = sys.ID
		}
		for _, q := range sys.Queues {
			qID := sys.ID + "." + q.ID
			defined[qID] = true
			definedLoc[qID] = q.Location()
			parent[qID] = sys.ID
		}
	}
	// Add top-level elements
	for _, cont := range arch.Containers {
		defined[cont.ID] = true
		definedLoc[cont.ID] = cont.Location()
		if len(cont.Requirements) > 0 || len(cont.ADRs) > 0 || len(cont.Contracts) > 0 {
			used[cont.ID] = true
		}
		for _, comp := range cont.Components {
			compID := cont.ID + "." + comp.ID
			defined[compID] = true
			definedLoc[compID] = comp.Location()
			parent[compID] = cont.ID
			if len(comp.Requirements) > 0 || len(comp.ADRs) > 0 {
				used[compID] = true
			}
		}
		for _, ds := range cont.DataStores {
			dsID := cont.ID + "." + ds.ID
			defined[dsID] = true
			definedLoc[dsID] = ds.Location()
			parent[dsID] = cont.ID
		}
		for _, q := range cont.Queues {
			qID := cont.ID + "." + q.ID
			defined[qID] = true
			definedLoc[qID] = q.Location()
			parent[qID] = cont.ID
		}
	}
	for _, comp := range arch.Components {
		defined[comp.ID] = true
		definedLoc[comp.ID] = comp.Location()
		if len(comp.Requirements) > 0 || len(comp.ADRs) > 0 {
			used[comp.ID] = true
		}
	}
	for _, ds := range arch.DataStores {
		defined[ds.ID] = true
		definedLoc[ds.ID] = ds.Location()
	}
	for _, q := range arch.Queues {
		defined[q.ID] = true
		definedLoc[q.ID] = q.Location()
	}
	for _, p := range arch.Persons {
		defined[p.ID] = true
		definedLoc[p.ID] = p.Location()
	}

	resolve := func(ref, scope string) string {
		// 1. Try absolute/global match
		if defined[ref] {
			return ref
		}

		// 2. Try relative to scope, walking up
		if scope != "" {
			candidate := scope + "." + ref
			if defined[candidate] {
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
				if defined[candidate] {
					return candidate
				}
			}
		}

		// 3. For architecture-level relations (scope=""), search all defined elements
		// to find matching IDs (e.g., "API" matches "Order.API")
		if scope == "" {
			for id := range defined {
				parts := strings.Split(id, ".")
				if len(parts) > 0 && parts[len(parts)-1] == ref {
					return id
				}
			}
		}

		return "" // Return empty if not resolved
	}

	markRel := func(from, to, scope string) {
		resolvedFrom := resolve(from, scope)
		if resolvedFrom != "" {
			used[resolvedFrom] = true
		}

		resolvedTo := resolve(to, scope)
		if resolvedTo != "" {
			used[resolvedTo] = true
		}
	}

	for _, r := range arch.Relations {
		markRel(r.From.String(), r.To.String(), "")
	}
	for _, s := range arch.Systems {
		for _, r := range s.Relations {
			markRel(r.From.String(), r.To.String(), s.ID)
		}
		for _, c := range s.Containers {
			contID := s.ID + "." + c.ID
			for _, r := range c.Relations {
				markRel(r.From.String(), r.To.String(), contID)
			}
		}
		for _, comp := range s.Components {
			compID := s.ID + "." + comp.ID
			for _, r := range comp.Relations {
				markRel(r.From.String(), r.To.String(), compID)
			}
		}
	}

	// Scenarios and Flows handling removed - Flow type not yet defined, Scenarios not in System
	// TODO: Implement Flow type, then add Flow checking
	// Scenarios are in Architecture - can be added back if needed
	// Helper to check steps
	// checkSteps := func(steps []*language.ScenarioStep) {
	// 	for _, step := range steps {
	// 		if step.To != "" {
	// 			markRel(step.From, step.To)
	// 		} else {
	// 			used[step.From] = true
	// 		}
	// 	}
	// 	}
	// for _, s := range arch.Scenarios {
	// 	checkSteps(s.Steps)
	// }

	// Propagate usage to parents
	// Iterate multiple times or just ensure we cover the depth (max 3: Component -> Container -> System)
	// Simple loop over defined elements to propagate
	for i := 0; i < 3; i++ {
		for id := range defined {
			if used[id] {
				if pID, ok := parent[id]; ok {
					used[pID] = true
				}
			}
		}
	}

	for id := range defined {
		if !used[id] {
			loc := definedLoc[id]
			// Extract short ID (last part) for error message
			parts := strings.Split(id, ".")
			shortID := parts[len(parts)-1]
			diags = append(diags, diagnostics.Diagnostic{
				Code:     diagnostics.CodeOrphanElement,
				Severity: diagnostics.SeverityWarning,
				Message:  fmt.Sprintf("Orphan element '%s' is defined but never used in any relation.", shortID),
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
