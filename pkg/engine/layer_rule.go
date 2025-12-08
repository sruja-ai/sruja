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
	var diags []diagnostics.Diagnostic

	if program == nil || program.Architecture == nil {
		return diags
	}

	// Define layers (ordered from top to bottom)
	// In a real implementation, this should be configurable
	layers := []string{"web", "api", "service", "data", "database"}
	layerMap := make(map[string]int)
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
	for _, rel := range program.Architecture.Relations {
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
				diags = append(diags, diagnostics.Diagnostic{
					Code:     diagnostics.CodeLayerViolation,
					Severity: diagnostics.SeverityError,
					Message:  fmt.Sprintf("Layer violation: '%s' (%s) cannot depend on '%s' (%s). Dependencies must flow downwards.", rel.From.String(), fromLayer, rel.To.String(), toLayer),
					Location: diagnostics.SourceLocation{
						File:   rel.Location().File,
						Line:   rel.Location().Line,
						Column: rel.Location().Column,
					},
					Context: []string{
						fmt.Sprintf("%s -> %s", rel.From.String(), rel.To.String()),
					},
					Suggestions: []string{
						"Reverse the dependency direction",
						"Introduce an interface or event bus to invert the dependency",
						"Move the functionality to a lower layer",
					},
				})
			}
		}
	}

	return diags
}

func findMetadata(program *language.Program, name string) []*language.MetaEntry {
	// Simple linear search for top-level elements
	// This handles simple cases. Nested elements would require a full traversal or symbol table.
	for i := range program.Architecture.Items {
		item := &program.Architecture.Items[i]
		if item.Container != nil && item.Container.ID == name {
			return item.Container.Metadata
		}
		if item.Component != nil && item.Component.ID == name {
			return item.Component.Metadata
		}
		if item.System != nil && item.System.ID == name {
			return item.System.Metadata
		}
		if item.DataStore != nil && item.DataStore.ID == name {
			return item.DataStore.Metadata
		}
	}
	return nil
}
