package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

type UniqueIDRule struct{}

func (r *UniqueIDRule) Name() string {
	return "Unique IDs"
}

func (r *UniqueIDRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Model == nil {
		return nil
	}

	seenIDs := make(map[string]language.SourceLocation, 100)
	diags := make([]diagnostics.Diagnostic, 0, 10)

	checkID := func(id string, loc language.SourceLocation) {
		if id == "" {
			return
		}
		if existing, ok := seenIDs[id]; ok {
			// Build enhanced error message with precise location
			var msgSb strings.Builder
			msgSb.Grow(len(id) + 100)
			msgSb.WriteString("Duplicate identifier '")
			msgSb.WriteString(id)
			msgSb.WriteString("'")
			switch {
			case existing.File != "" && existing.File == loc.File:
				msgSb.WriteString(fmt.Sprintf(". First defined at line %d, column %d", existing.Line, existing.Column))
			case existing.File != "":
				msgSb.WriteString(fmt.Sprintf(". First defined in '%s' at line %d", existing.File, existing.Line))
			default:
				msgSb.WriteString(fmt.Sprintf(". Previously defined at line %d", existing.Line))
			}

			var suggestions []string
			suggestions = append(suggestions, fmt.Sprintf("Rename this element to a unique identifier (e.g., '%s2' or '%s_v2')", id, id))
			suggestions = append(suggestions, "Element IDs must be unique within the architecture")
			if existing.File != "" && existing.File != loc.File {
				suggestions = append(suggestions, "Consider using a namespace or prefix to avoid conflicts across files")
			}

			diags = append(diags, diagnostics.Diagnostic{
				Code:     diagnostics.CodeDuplicateIdentifier,
				Severity: diagnostics.SeverityError,
				Message:  msgSb.String(),
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
				Suggestions: suggestions,
			})
		} else {
			seenIDs[id] = loc
		}
	}

	// Collect all element IDs from Model
	var collectIDs func(elem *language.ElementDef, parentFQN string)
	collectIDs = func(elem *language.ElementDef, parentFQN string) {
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
			// Check both FQN and short ID for uniqueness if they differ
			checkID(id, elem.Location())
		}

		checkID(fqn, elem.Location())

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					collectIDs(bodyItem.Element, fqn)
				}
			}
		}
	}

	// Process all items in model
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			collectIDs(item.ElementDef, "")

		}
	}

	return diags
}
