package engine

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
	"golang.org/x/text/cases"
	lang "golang.org/x/text/language"
)

// CompletenessRule checks for missing optional but recommended elements
// like descriptions, technologies, and empty containers/systems.
type CompletenessRule struct{}

func (r *CompletenessRule) Name() string {
	return "CompletenessCheck"
}

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *CompletenessRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Model == nil {
		return nil
	}

	// Pre-allocate diagnostics slice with estimated capacity
	estimatedDiags := 20
	diags := make([]diagnostics.Diagnostic, 0, estimatedDiags)

	// Helper to check Sruja model elements
	var checkElement func(elem *language.ElementDef, parentFQN string, hasNested bool)
	checkElement = func(elem *language.ElementDef, parentFQN string, _ bool) {
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
		}

		loc := elem.Location()
		kind := elem.GetKind()

		// Extract description and technology from body
		description := ""
		technology := ""
		nestedCount := 0

		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Description != nil {
					description = *item.Description
				}
				if item.Technology != nil {
					technology = *item.Technology
				}
				if item.Element != nil {
					nestedCount++
				}
			}
		}

		// Check for missing description
		if description == "" {
			var msgSb strings.Builder
			msgSb.Grow(len(id) + 50)
			msgSb.WriteString(cases.Title(lang.Und).String(kind))
			msgSb.WriteString(" '")
			msgSb.WriteString(id)
			msgSb.WriteString("' has no description.")
			diags = append(diags, diagnostics.Diagnostic{
				Code:     "SUGGESTION_MISSING_DESC",
				Severity: diagnostics.SeverityInfo,
				Message:  msgSb.String(),
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
			})
		}

		// Check for missing technology (for containers)
		if kind == "container" && technology == "" {
			var msgSb strings.Builder
			msgSb.Grow(len(id) + 60)
			msgSb.WriteString("Container '")
			msgSb.WriteString(id)
			msgSb.WriteString("' has no technology defined (e.g., 'Go', 'React').")
			diags = append(diags, diagnostics.Diagnostic{
				Code:     "SUGGESTION_MISSING_TECH",
				Severity: diagnostics.SeverityInfo,
				Message:  msgSb.String(),
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
			})
		}

		// Check for empty containers/systems
		if (kind == "container" || kind == "system") && nestedCount == 0 {
			var msgSb strings.Builder
			msgSb.Grow(len(id) + 50)
			msgSb.WriteString(cases.Title(lang.Und).String(kind))
			msgSb.WriteString(" '")
			msgSb.WriteString(id)
			if kind == "container" {
				msgSb.WriteString("' has no components. Consider breaking it down.")
			} else {
				msgSb.WriteString("' is empty. Add Containers or Components.")
			}
			diags = append(diags, diagnostics.Diagnostic{
				Code:     "SUGGESTION_EMPTY_" + strings.ToUpper(kind),
				Severity: diagnostics.SeverityInfo,
				Message:  msgSb.String(),
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
			})
		}

		// Recurse into nested elements
		body = elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					checkElement(bodyItem.Element, fqn, true)
				}
			}
		}
	}

	// Process all top-level elements
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			checkElement(item.ElementDef, "", false)
		}
	}

	return diags
}
