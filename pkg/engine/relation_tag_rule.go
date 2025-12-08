package engine

import (
	"fmt"
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
	var diags []diagnostics.Diagnostic
	arch := program.Architecture
	if arch == nil {
		return diags
	}

	checkTags := func(rel *language.Relation) {
		if rel == nil {
			return
		}
		for _, tag := range rel.Tags {
			lowerTag := strings.ToLower(tag)
			if !allowedTags[lowerTag] {
				diags = append(diags, diagnostics.Diagnostic{
					Code:     diagnostics.CodeValidationRuleError,
					Severity: diagnostics.SeverityWarning,
					Message:  fmt.Sprintf("Invalid relation tag '%s'. Allowed tags are: Reads, Writes, Sends, Uses, Calls, Processes, Queries, Updates, Deletes, Creates, Triggers, Notifies.", tag),
					Location: diagnostics.SourceLocation{
						File:   rel.Location().File,
						Line:   rel.Location().Line,
						Column: rel.Location().Column,
					},
				})
			}
		}
	}

	// Check top-level relations
	for _, rel := range arch.Relations {
		checkTags(rel)
	}

	// Check system relations
	for _, sys := range arch.Systems {
		for _, rel := range sys.Relations {
			checkTags(rel)
		}
		for _, item := range sys.Items {
			if item.Relation != nil {
				checkTags(item.Relation)
			}
		}

		for _, c := range sys.Containers {
			for i := range c.Items {
				item := &c.Items[i]
				if item.Relation != nil {
					checkTags(item.Relation)
				}
			}
			for _, comp := range c.Components {
				for _, rel := range comp.Relations {
					checkTags(rel)
				}
				for _, item := range comp.Items {
					if item.Relation != nil {
						checkTags(item.Relation)
					}
				}
			}
		}
	}

	return diags
}
