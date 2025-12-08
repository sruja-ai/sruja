// pkg/export/markdown/consistency_helpers.go
// Package markdown provides helper functions for data consistency section.
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// extractConsistencyMetadata extracts consistency info from container metadata
func extractConsistencyMetadata(arch *language.Architecture) []string {
	var notes []string
	for _, sys := range arch.Systems {
		for _, container := range sys.Containers {
			for _, meta := range container.Metadata {
				if (meta.Key == "consistency" || meta.Key == "transaction") && meta.Value != nil {
					notes = append(notes, fmt.Sprintf("- **%s.%s**: %s", sys.ID, container.ID, *meta.Value))
				}
			}
		}
	}
	return notes
}

// extractConsistencyFromRelations extracts consistency info from relations
func extractConsistencyFromRelations(arch *language.Architecture) []string {
	var notes []string
	for _, rel := range arch.Relations {
		if rel.Verb == nil {
			continue
		}
		verbLower := strings.ToLower(*rel.Verb)
		if isStrongConsistencyVerb(verbLower) {
			notes = append(notes, fmt.Sprintf("- **%s → %s**: Strong consistency (ACID transaction)", rel.From.String(), rel.To.String()))
		}
	}
	return notes
}

// isStrongConsistencyVerb checks if a verb indicates strong consistency
func isStrongConsistencyVerb(verb string) bool {
	return strings.Contains(verb, "write") ||
		strings.Contains(verb, "update") ||
		strings.Contains(verb, "create")
}



