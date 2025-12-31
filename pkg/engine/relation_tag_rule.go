package engine

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

type RelationTagRule struct{}

func (r *RelationTagRule) Name() string {
	return "Relation Tags"
}

var allowedTags = map[string]bool{
	"reads":     true,
	"writes":    true,
	"sends":     true,
	"uses":      true,
	"calls":     true,
	"processes": true,
	"queries":   true,
	"updates":   true,
	"deletes":   true,
	"creates":   true,
	"triggers":  true,
	"notifies":  true,
}

//nolint:gocyclo // Validation logic is complex
func (r *RelationTagRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Model == nil {
		return nil
	}
	// Pre-allocate diagnostics slice with estimated capacity
	diags := make([]diagnostics.Diagnostic, 0, 10)

	checkTags := func(rel *language.Relation) {
		if rel == nil {
			return
		}
		for _, tag := range rel.Tags {
			lowerTag := strings.ToLower(tag)
			if !allowedTags[lowerTag] {
				// Build message efficiently
				var msgSb strings.Builder
				msgSb.Grow(len(tag) + 120)
				msgSb.WriteString("Invalid relation tag '")
				msgSb.WriteString(tag)
				msgSb.WriteString("'. Allowed tags are: Reads, Writes, Sends, Uses, Calls, Processes, Queries, Updates, Deletes, Creates, Triggers, Notifies.")
				diags = append(diags, diagnostics.Diagnostic{
					Code:     diagnostics.CodeValidationRuleError,
					Severity: diagnostics.SeverityWarning,
					Message:  msgSb.String(),
					Location: diagnostics.SourceLocation{
						File:   rel.Location().File,
						Line:   rel.Location().Line,
						Column: rel.Location().Column,
					},
				})
			}
		}
	}

	// Collect all relations from Model
	_, relations := collectElements(program.Model)

	// Check all relations
	for _, rel := range relations {
		checkTags(rel)
	}

	return diags
}
