package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

type ScenarioFQNRule struct{}

func (r *ScenarioFQNRule) Name() string { return "ScenarioReferenceValidation" }

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *ScenarioFQNRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic
	if program == nil || program.Architecture == nil {
		return diags
	}

	arch := program.Architecture

	// Build set of defined qualified IDs
	defined := map[string]bool{}
	// Build map of suffix -> []fullID for smart resolution
	suffixMap := map[string][]string{}

	addID := func(id string) {
		defined[id] = true
		parts := strings.Split(id, ".")
		suffix := parts[len(parts)-1]
		suffixMap[suffix] = append(suffixMap[suffix], id)
	}

	// Top-level elements
	for _, cont := range arch.Containers {
		addID(cont.ID)
		for _, comp := range cont.Components {
			addID(cont.ID + "." + comp.ID)
		}
		for _, ds := range cont.DataStores {
			addID(cont.ID + "." + ds.ID)
		}
		for _, q := range cont.Queues {
			addID(cont.ID + "." + q.ID)
		}
	}
	for _, comp := range arch.Components {
		addID(comp.ID)
	}
	for _, ds := range arch.DataStores {
		addID(ds.ID)
	}
	for _, q := range arch.Queues {
		addID(q.ID)
	}
	for _, p := range arch.Persons {
		addID(p.ID)
	}

	// Nested elements under systems
	for _, sys := range arch.Systems {
		addID(sys.ID)
		for _, cont := range sys.Containers {
			contID := sys.ID + "." + cont.ID
			addID(contID)
			for _, comp := range cont.Components {
				addID(contID + "." + comp.ID)
			}
			for _, ds := range cont.DataStores {
				addID(contID + "." + ds.ID)
			}
			for _, q := range cont.Queues {
				addID(contID + "." + q.ID)
			}
		}
		for _, comp := range sys.Components {
			addID(sys.ID + "." + comp.ID)
		}
		for _, ds := range sys.DataStores {
			addID(sys.ID + "." + ds.ID)
		}
		for _, q := range sys.Queues {
			addID(sys.ID + "." + q.ID)
		}
	}

	validateRef := func(ref string, loc language.SourceLocation) {
		if ref == "" {
			return
		}

		// 1. Exact match (Fully Qualified or Global)
		if defined[ref] {
			return
		}

		// 2. Unqualified match
		var suffix string
		if !strings.Contains(ref, ".") {
			suffix = ref
		} else {
			// If it has dots but isn't exact match, it might be a partial path?
			// Current DSL logic doesn't really support "middle" partials well, usually just suffix.
			// Treat as unknown or check suffix?
			// Let's stick to simple suffix logic for now.
			parts := strings.Split(ref, ".")
			suffix = parts[len(parts)-1]
		}

		matches := suffixMap[suffix]

		if len(matches) == 0 {
			diags = append(diags, diagnostics.Diagnostic{
				Code:     diagnostics.CodeReferenceNotFound,
				Severity: diagnostics.SeverityError,
				Message:  fmt.Sprintf("Reference to undefined element '%s'", ref),
				Location: diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column},
			})
			return
		}

		if len(matches) == 1 {
			// Unique resolution found!
			// We allow this usage.
			// Implicitly resolves to matches[0]
			return
		}

		// Ambiguous match
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeValidationRuleError,
			Severity: diagnostics.SeverityError,
			Message:  fmt.Sprintf("Ambiguous reference '%s' matches multiple elements: %s (use fully qualified name)", ref, strings.Join(matches, ", ")),
			Location: diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column},
		})
	}

	checkStep := func(step *language.ScenarioStep) {
		if step == nil {
			return
		}
		validateRef(step.From.String(), step.Location())
		validateRef(step.To.String(), step.Location())
	}

	for _, s := range arch.Scenarios {
		for _, step := range s.Steps {
			checkStep(step)
		}
	}
	for _, f := range arch.Flows {
		for _, step := range f.Steps {
			checkStep(step)
		}
	}

	return diags
}
