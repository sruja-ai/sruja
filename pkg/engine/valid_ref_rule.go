package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

type ValidReferenceRule struct{}

func (r *ValidReferenceRule) Name() string {
	return "Valid References"
}

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *ValidReferenceRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic
	if program == nil {
		return diags
	}
	defined := map[string]bool{}

	arch := program.Architecture
	if arch == nil {
		return diags
	}

	for _, sys := range arch.Systems {
		defined[sys.ID] = true
		for _, cont := range sys.Containers {
			contID := sys.ID + "." + cont.ID
			defined[contID] = true
			for _, comp := range cont.Components {
				defined[contID+"."+comp.ID] = true
			}
			for _, ds := range cont.DataStores {
				defined[contID+"."+ds.ID] = true
			}
			for _, q := range cont.Queues {
				defined[contID+"."+q.ID] = true
			}
		}
		for _, comp := range sys.Components {
			defined[sys.ID+"."+comp.ID] = true
		}
		for _, ds := range sys.DataStores {
			defined[sys.ID+"."+ds.ID] = true
		}
		for _, q := range sys.Queues {
			defined[sys.ID+"."+q.ID] = true
		}
	}
	// Add top-level elements
	for _, cont := range arch.Containers {
		defined[cont.ID] = true
		for _, comp := range cont.Components {
			defined[cont.ID+"."+comp.ID] = true
		}
		for _, ds := range cont.DataStores {
			defined[cont.ID+"."+ds.ID] = true
		}
		for _, q := range cont.Queues {
			defined[cont.ID+"."+q.ID] = true
		}
	}
	for _, comp := range arch.Components {
		defined[comp.ID] = true
	}
	for _, ds := range arch.DataStores {
		defined[ds.ID] = true
	}
	for _, q := range arch.Queues {
		defined[q.ID] = true
	}
	for _, p := range arch.Persons {
		defined[p.ID] = true
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

		return ""
	}

	checkRel := func(rel *language.Relation, scope string) {
		if rel == nil {
			return
		}
		if resolve(rel.From.String(), scope) == "" {
			loc := rel.Location()
			diags = append(diags, diagnostics.Diagnostic{
				Code:     diagnostics.CodeReferenceNotFound,
				Severity: diagnostics.SeverityError,
				Message:  fmt.Sprintf("Reference to undefined element '%s'", rel.From.String()),
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
			})
		}
		if resolve(rel.To.String(), scope) == "" {
			loc := rel.Location()
			diags = append(diags, diagnostics.Diagnostic{
				Code:     diagnostics.CodeReferenceNotFound,
				Severity: diagnostics.SeverityError,
				Message:  fmt.Sprintf("Reference to undefined element '%s'", rel.To.String()),
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
			})
		}
	}
	for _, r := range arch.Relations {
		checkRel(r, "")
	}
	for _, s := range arch.Systems {
		for _, r := range s.Relations {
			checkRel(r, s.ID)
		}
		for _, c := range s.Containers {
			contID := s.ID + "." + c.ID
			for _, r := range c.Relations {
				checkRel(r, contID)
			}
		}
		for _, comp := range s.Components {
			compID := s.ID + "." + comp.ID
			for _, r := range comp.Relations {
				checkRel(r, compID)
			}
		}
	}

	// Current Requirement struct has no Implements field; skip implements checks
	return diags
}
