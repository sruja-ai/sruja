package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// LayerViolationRule enforces strict layering (e.g., Web -> API -> Database).
type LayerViolationRule struct{}

func (r *LayerViolationRule) Name() string {
	return "Layer Violation"
}

func (r *LayerViolationRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Model == nil {
		return nil
	}

	// Collect all relations from Model
	_, relations := collectLikeC4Elements(program.Model)

	// Pre-allocate diagnostics slice
	estimatedDiags := len(relations) / 10
	if estimatedDiags < 8 {
		estimatedDiags = 8
	}
	diags := make([]diagnostics.Diagnostic, 0, estimatedDiags)

	// Define layers (ordered from top to bottom)
	// In a real implementation, this should be configurable
	layers := []string{"web", "api", "service", "data", "database"}
	layerMap := make(map[string]int, len(layers))
	for i, l := range layers {
		layerMap[l] = i
	}

	// Helper to determine layer of an element
	getLayer := func(name string, metadata []*language.MetaEntry) string {
		// 1. Check metadata
		for _, m := range metadata {
			if m.Key == "layer" && m.Value != nil {
				return strings.ToLower(strings.Trim(*m.Value, "\""))
			}
		}
		// 2. Check name convention (simple heuristic)
		lowerName := strings.ToLower(name)
		for _, l := range layers {
			if strings.Contains(lowerName, l) {
				return l
			}
		}
		return ""
	}

	// Check all relations
	for _, rel := range relations {
		fromLayer := ""
		toLayer := ""

		// Find source element
		// This is a simplified lookup. In a real implementation, we'd use a symbol table.
		// For now, we'll just check top-level items or infer from name.
		// We can try to find the element in the architecture items to get its metadata.

		// Optimization: Build a map of element names to metadata first?
		// For this rule, let's just iterate items to find metadata if possible.
		// Or just rely on the name heuristic if metadata isn't easily accessible without a symbol table.
		// Let's try to find the element definition to get metadata.

		fromMeta := findMetadata(program, rel.From.String())
		toMeta := findMetadata(program, rel.To.String())

		fromLayer = getLayer(rel.From.String(), fromMeta)
		toLayer = getLayer(rel.To.String(), toMeta)

		if fromLayer != "" && toLayer != "" {
			fromIdx := layerMap[fromLayer]
			toIdx := layerMap[toLayer]

			// Rule: Can only call same layer or one layer below
			// Strict layering: fromIdx <= toIdx (assuming 0 is top, N is bottom? No, usually Web calls API)
			// Let's say:
			// Web (0) -> API (1) : OK (0 < 1)
			// API (1) -> Data (3) : OK (1 < 3) - Relaxed layering (can skip layers)
			// Data (3) -> Web (0) : VIOLATION (3 > 0)

			if fromIdx > toIdx {
				// Build enhanced error message with suggestions
				var msgSb strings.Builder
				fromID := rel.From.String()
				toID := rel.To.String()
				msgSb.Grow(len(fromID) + len(toID) + len(fromLayer) + len(toLayer) + 120)
				msgSb.WriteString("Layer violation: '")
				msgSb.WriteString(fromID)
				msgSb.WriteString("' (")
				msgSb.WriteString(fromLayer)
				msgSb.WriteString(") cannot depend on '")
				msgSb.WriteString(toID)
				msgSb.WriteString("' (")
				msgSb.WriteString(toLayer)
				msgSb.WriteString("). Dependencies must flow downwards (higher layers can only depend on lower layers).")

				var suggestions []string
				suggestions = append(suggestions, fmt.Sprintf("Reverse the dependency: '%s -> %s'", toID, fromID))
				suggestions = append(suggestions, "Or restructure to follow proper layering (e.g., Web -> API -> Data)")
				suggestions = append(suggestions, "If this is intentional, consider documenting the exception")

				diags = append(diags, diagnostics.Diagnostic{
					Code:     diagnostics.CodeLayerViolation,
					Severity: diagnostics.SeverityError,
					Message:  msgSb.String(),
					Location: diagnostics.SourceLocation{
						File:   rel.Location().File,
						Line:   rel.Location().Line,
						Column: rel.Location().Column,
					},
					Context: []string{
						fmt.Sprintf("%s -> %s", rel.From.String(), rel.To.String()),
					},
					Suggestions: suggestions,
				})
			}
		}
	}

	return diags
}

func findMetadata(program *language.Program, name string) []*language.MetaEntry {
	if program == nil || program.Model == nil {
		return nil
	}

	// Search for element by qualified name in LikeC4 Model
	var findElement func(elem *language.LikeC4ElementDef, currentFQN string) []*language.MetaEntry
	findElement = func(elem *language.LikeC4ElementDef, currentFQN string) []*language.MetaEntry {
		if elem == nil {
			return nil
		}

		id := elem.GetID()
		if id == "" {
			return nil
		}

		fqn := id
		if currentFQN != "" {
			fqn = buildQualifiedID(currentFQN, id)
		}

		// Check if this is the element we're looking for
		if fqn == name || id == name {
			// Extract metadata from body
			body := elem.GetBody()
			if body != nil {
				for _, item := range body.Items {
					if item.Metadata != nil {
						return item.Metadata.Entries
					}
				}
			}
			return nil
		}

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					if meta := findElement(bodyItem.Element, fqn); meta != nil {
						return meta
					}
				}
			}
		}

		return nil
	}

	// Search all top-level elements
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			if meta := findElement(item.ElementDef, ""); meta != nil {
				return meta
			}
		}
	}

	return nil
}
