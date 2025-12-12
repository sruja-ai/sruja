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
	if program == nil || program.Architecture == nil {
		return nil
	}

	arch := program.Architecture
	// Estimate capacity based on architecture size
	estimatedIDs := estimateIDCount(arch)
	seenIDs := make(map[string]language.SourceLocation, estimatedIDs)
	diags := make([]diagnostics.Diagnostic, 0, estimatedIDs/10) // Assume ~10% duplicates

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
			if existing.File != "" && existing.File == loc.File {
				msgSb.WriteString(fmt.Sprintf(". First defined at line %d, column %d", existing.Line, existing.Column))
			} else if existing.File != "" {
				msgSb.WriteString(fmt.Sprintf(". First defined in '%s' at line %d", existing.File, existing.Line))
			} else {
				msgSb.WriteString(fmt.Sprintf(". Previously defined at line %d", existing.Line))
			}

			var suggestions []string
			suggestions = append(suggestions, fmt.Sprintf("Rename this element to a unique identifier (e.g., '%s2' or '%s_v2')", id, id))
			suggestions = append(suggestions, "Element IDs must be unique within the architecture")
			if existing.File != "" && existing.File != loc.File {
				suggestions = append(suggestions, fmt.Sprintf("Consider using a namespace or prefix to avoid conflicts across files"))
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

	for _, sys := range arch.Systems {
		checkID(sys.ID, sys.Location())
		for _, cont := range sys.Containers {
			checkID(cont.ID, cont.Location())
			for _, comp := range cont.Components {
				checkID(comp.ID, comp.Location())
			}
			for _, ds := range cont.DataStores {
				checkID(ds.ID, ds.Location())
			}
			for _, q := range cont.Queues {
				checkID(q.ID, q.Location())
			}
		}
		for _, comp := range sys.Components {
			checkID(comp.ID, comp.Location())
		}
		for _, ds := range sys.DataStores {
			checkID(ds.ID, ds.Location())
		}
		for _, q := range sys.Queues {
			checkID(q.ID, q.Location())
		}
	}

	for _, p := range arch.Persons {
		checkID(p.ID, p.Location())
	}
	for _, req := range arch.Requirements {
		checkID(req.ID, req.Location())
	}
	for _, adr := range arch.ADRs {
		checkID(adr.ID, adr.Location())
	}

	return diags
}

// estimateIDCount provides a rough estimate of total IDs for map pre-allocation.
func estimateIDCount(arch *language.Architecture) int {
	if arch == nil {
		return 16
	}
	count := len(arch.Persons) + len(arch.Requirements) + len(arch.ADRs)
	for _, sys := range arch.Systems {
		count += 1 + len(sys.Components) + len(sys.DataStores) + len(sys.Queues)
		for _, cont := range sys.Containers {
			count += 1 + len(cont.Components) + len(cont.DataStores) + len(cont.Queues)
		}
	}
	return count + 32 // Add buffer
}
