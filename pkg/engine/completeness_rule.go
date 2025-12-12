package engine

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// CompletenessRule checks for missing optional but recommended elements
// like descriptions, technologies, and empty containers/systems.
type CompletenessRule struct{}

func (r *CompletenessRule) Name() string {
	return "CompletenessCheck"
}

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *CompletenessRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program.Architecture == nil {
		return nil
	}

	// Pre-allocate diagnostics slice with estimated capacity
	estimatedDiags := 20
	diags := make([]diagnostics.Diagnostic, 0, estimatedDiags)

	// Helper functions
	var checkComponent func(comp *language.Component)
	var checkContainer func(cont *language.Container)
	var checkSystem func(sys *language.System)

	checkComponent = func(comp *language.Component) {
		if comp.Description == nil || *comp.Description == "" {
			loc := comp.Location()
			// Build message efficiently
			var msgSb strings.Builder
			msgSb.Grow(len(comp.ID) + 30)
			msgSb.WriteString("Component '")
			msgSb.WriteString(comp.ID)
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

		// Check for orphans (no relations)
		// This is a bit more complex as we need to check all relations
		// For now, let's skip complex graph analysis here as OrphanRule might cover it,
		// but OrphanRule is usually an Error. We might want a "Suggestion" version.
	}

	checkContainer = func(cont *language.Container) {
		if cont.Description == nil || *cont.Description == "" {
			loc := cont.Location()
			// Build message efficiently
			var msgSb strings.Builder
			msgSb.Grow(len(cont.ID) + 30)
			msgSb.WriteString("Container '")
			msgSb.WriteString(cont.ID)
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

		// Check for technology in items
		hasTech := false
		if cont.Items != nil {
			for i := range cont.Items {
				item := &cont.Items[i]
				if item.Technology != nil && *item.Technology != "" {
					hasTech = true
					break
				}
			}
		}
		// Also check if technology is set directly if post-processing moves it there (it usually does not for this field in this parser version,
		// but checking Items is safe as per AST def). Wait, simpler check:
		// The AST definition has `Technology *string` in the `Container` struct comment as a post-processed field?
		// Let's check the container struct definition again.
		// In `ast_elements.go`: `type Container struct { ... Items []ContainerItem ... }`.
		// It does NOT have `Technology *string` in the struct fields list, only in comments?
		// Wait, looking closely at `ast_elements.go` again:
		// `type Container struct { ... Items []ContainerItem ... }`
		// `type ContainerItem struct { Technology *string ... }`
		// So we must iterate items. The previous code did that.

		if !hasTech {
			loc := cont.Location()
			// Build message efficiently
			var msgSb strings.Builder
			msgSb.Grow(len(cont.ID) + 60)
			msgSb.WriteString("Container '")
			msgSb.WriteString(cont.ID)
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

		if len(cont.Components) == 0 {
			loc := cont.Location()
			// Build message efficiently
			var msgSb strings.Builder
			msgSb.Grow(len(cont.ID) + 50)
			msgSb.WriteString("Container '")
			msgSb.WriteString(cont.ID)
			msgSb.WriteString("' has no components. Consider breaking it down.")
			diags = append(diags, diagnostics.Diagnostic{
				Code:     "SUGGESTION_EMPTY_CONTAINER",
				Severity: diagnostics.SeverityInfo,
				Message:  msgSb.String(),
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
			})
		}

		// Recurse into components
		for _, comp := range cont.Components {
			checkComponent(comp)
		}
	}

	checkSystem = func(sys *language.System) {
		// Check description
		if sys.Description == nil || *sys.Description == "" {
			loc := sys.Location()
			// Build message efficiently
			var msgSb strings.Builder
			msgSb.Grow(len(sys.ID) + 50)
			msgSb.WriteString("System '")
			msgSb.WriteString(sys.ID)
			msgSb.WriteString("' has no description. Adding one helps context.")
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

		// Check empty system (no containers/components)
		if len(sys.Containers) == 0 && len(sys.Components) == 0 {
			loc := sys.Location()
			// Build message efficiently
			var msgSb strings.Builder
			msgSb.Grow(len(sys.ID) + 50)
			msgSb.WriteString("System '")
			msgSb.WriteString(sys.ID)
			msgSb.WriteString("' is empty. Add Containers or Components.")
			diags = append(diags, diagnostics.Diagnostic{
				Code:     "SUGGESTION_EMPTY_SYSTEM",
				Severity: diagnostics.SeverityInfo,
				Message:  msgSb.String(),
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
			})
		}

		// Recurse
		for _, cont := range sys.Containers {
			checkContainer(cont)
		}
		for _, comp := range sys.Components {
			checkComponent(comp)
		}
	}

	// Process all top-level elements and their children
	for _, sys := range program.Architecture.Systems {
		checkSystem(sys)
	}

	for _, cont := range program.Architecture.Containers {
		checkContainer(cont)
	}

	for _, comp := range program.Architecture.Components {
		checkComponent(comp)
	}

	return diags
}
