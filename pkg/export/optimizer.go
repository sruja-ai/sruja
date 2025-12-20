// pkg/export/optimizer.go
package export

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// TokenOptimizer optimizes program output for token limits
type TokenOptimizer struct {
	limit int
}

// NewTokenOptimizer creates a new token optimizer
func NewTokenOptimizer(limit int) *TokenOptimizer {
	return &TokenOptimizer{limit: limit}
}

// EstimateTokens provides a rough token estimate (4 chars per token)
func EstimateTokens(content string) int {
	return len(content) / 4
}

// ShouldOptimize returns true if optimization is needed
func (o *TokenOptimizer) ShouldOptimize(currentTokens int) bool {
	return o.limit > 0 && currentTokens > o.limit
}

// TruncateDescription truncates a description to fit token budget
func (o *TokenOptimizer) TruncateDescription(desc string, maxTokens int) string {
	if maxTokens <= 0 {
		return desc
	}
	maxChars := maxTokens * 4
	if len(desc) <= maxChars {
		return desc
	}
	// Truncate and add ellipsis
	truncated := desc[:maxChars-3]
	// Try to truncate at word boundary
	if lastSpace := strings.LastIndex(truncated, " "); lastSpace > maxChars/2 {
		truncated = truncated[:lastSpace]
	}
	return truncated + "..."
}

// PrioritizeElements sorts elements by importance for token optimization
// Returns: systems, containers, components in priority order
func PrioritizeElements(program *language.Program) (systems, containers, components []*language.LikeC4ElementDef) {
	if program == nil || program.Model == nil {
		return nil, nil, nil
	}

	var collectElements func(elem *language.LikeC4ElementDef)
	collectElements = func(elem *language.LikeC4ElementDef) {
		if elem == nil {
			return
		}

		switch elem.GetKind() {
		case "system":
			systems = append(systems, elem)
		case "container":
			containers = append(containers, elem)
		case "component":
			components = append(components, elem)
		}

		// Recurse into children
		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					collectElements(item.Element)
				}
			}
		}
	}

	// Collect from top level
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			collectElements(item.ElementDef)
		}
	}

	return systems, containers, components
}

// FilterByScope filters program elements based on scope
func FilterByScope(program *language.Program, scopeType, scopeID string) *language.Program {
	if scopeType == "full" || scopeID == "" {
		return program
	}

	// Create a filtered program
	filtered := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{},
		},
	}

	if program == nil || program.Model == nil {
		return filtered
	}

	// Find the scoped element
	var findElement func(elem *language.LikeC4ElementDef, targetKind, targetID string) *language.LikeC4ElementDef
	findElement = func(elem *language.LikeC4ElementDef, targetKind, targetID string) *language.LikeC4ElementDef {
		if elem == nil {
			return nil
		}

		if elem.GetKind() == targetKind && elem.GetID() == targetID {
			return elem
		}

		// Search in children
		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					if found := findElement(item.Element, targetKind, targetID); found != nil {
						return found
					}
				}
			}
		}

		return nil
	}

	// Search for the scoped element
	var scopedElement *language.LikeC4ElementDef
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			if found := findElement(item.ElementDef, scopeType, scopeID); found != nil {
				scopedElement = found
				break
			}
		}
	}

	if scopedElement == nil {
		// Element not found, return empty program
		return filtered
	}

	// Add the scoped element and its children to filtered program
	// For now, we'll include the parent system if scoping to container/component
	// This is a simplified version - full implementation would need to handle
	// relationships and include related elements
	filtered.Model.Items = append(filtered.Model.Items, language.ModelItem{
		ElementDef: scopedElement,
	})

	// Also include related items (requirements, ADRs) from original program
	for _, item := range program.Model.Items {
		if item.Requirement != nil || item.ADR != nil {
			filtered.Model.Items = append(filtered.Model.Items, item)
		}
	}

	return filtered
}
