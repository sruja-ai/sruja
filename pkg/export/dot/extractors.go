package dot

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// Relation represents an extracted relation for DOT generation.
type Relation struct {
	From  string
	To    string
	Label string
}

// extractAllElements extracts all elements from the program into a flat list.
func (e *Exporter) extractAllElements(prog *language.Program) []*Element {
	if prog == nil || prog.Model == nil {
		return nil
	}

	var elements []*Element

	var extractFromElementDef func(elem *language.ElementDef, parentID string)
	extractFromElementDef = func(elem *language.ElementDef, parentID string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		// Build full ID with parent prefix
		fullID := id
		if parentID != "" {
			fullID = parentID + "." + id
		}

		kind := strings.ToLower(elem.GetKind())
		// Normalize kinds
		switch kind {
		case "database", "db", "storage":
			kind = "datastore"
		case "mq":
			kind = "queue"
		case "actor":
			kind = "person"
		}

		title := getString(elem.GetTitle())
		if title == "" {
			title = id
		}

		// Get technology from body if available
		var technology string
		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Technology != nil {
					technology = *item.Technology
					break
				}
			}
		}

		// Get description if available
		var description string
		if body != nil {
			for _, item := range body.Items {
				if item.Description != nil {
					description = *item.Description
					break
				}
			}
		}

		// Create element first to measure content
		newElem := &Element{
			ID:          fullID,
			Kind:        kind,
			Title:       title,
			Technology:  technology,
			Description: description,
			ParentID:    parentID,
		}

		// Measure actual content using font metrics
		width, height := MeasureNodeContent(newElem)
		newElem.Width = int(width)
		newElem.Height = int(height)

		elements = append(elements, newElem)

		// Recursively extract children
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					extractFromElementDef(bodyItem.Element, fullID)
				}
			}
		}
	}

	// Extract from model items
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil {
			extractFromElementDef(item.ElementDef, "")
		}
	}

	return elements
}

// extractRelationsFromModel extracts all relations from the program.
func extractRelationsFromModel(prog *language.Program) []*Relation {
	if prog == nil || prog.Model == nil {
		return nil
	}

	var relations []*Relation
	var contextStack []string // Track parent hierarchy for context resolution

	// Helper to extract relations from a list of model items
	var extractFromItems func(items []language.ModelItem)
	// Helper to extract relations from a list of body items (needed because types differ)
	var extractFromBodyItems func(items []*language.BodyItem)

	extractFromItems = func(items []language.ModelItem) {
		for _, item := range items {
			// 1. Direct Relation
			if item.Relation != nil {
				processRelation(item.Relation, contextStack, &relations)
			}

			// 2. Element Definition (recurse into body)
			if item.ElementDef != nil {
				id := item.ElementDef.GetID()
				if id != "" {
					// Push ID to context stack
					contextStack = append(contextStack, id)

					// Recurse
					body := item.ElementDef.GetBody()
					if body != nil {
						extractFromBodyItems(body.Items)
					}

					// Pop ID
					contextStack = contextStack[:len(contextStack)-1]
				}
			}
		}
	}

	extractFromBodyItems = func(items []*language.BodyItem) {
		for _, item := range items {
			// 1. Direct Relation within body
			if item.Relation != nil {
				processRelation(item.Relation, contextStack, &relations)
			}

			// 2. Nested Element (recurse)
			if item.Element != nil {
				id := item.Element.GetID()
				if id != "" {
					// Push ID to context stack
					contextStack = append(contextStack, id)

					// Recurse
					body := item.Element.GetBody()
					if body != nil {
						extractFromBodyItems(body.Items)
					}

					// Pop ID
					contextStack = contextStack[:len(contextStack)-1]
				}
			}
		}
	}

	// Start extraction from root model items
	extractFromItems(prog.Model.Items)

	return relations
}

// processRelation processes a single relation AST node and adds it to the relations list.
func processRelation(rel *language.Relation, _ []string, relations *[]*Relation) {
	if rel == nil {
		return
	}

	// Resolve From/To IDs
	// Note: We might need to resolve relative paths based on contextStack in the future,
	// but for now we assume the parser gives us fully qualified or globally unique IDs,
	// or that the Exporter.getVisibleAncestorWithContext handles the short-name resolution.
	// However, storing the context in the relation could be useful.

	from := rel.From.String()
	to := rel.To.String()

	// If we are deep in the stack, we might need to qualify these IDs if they aren't fully qualified?
	// The current Parser usually handles ID resolution, but if `rel.From` is just "api",
	// and we are inside "system1", it likely means "system1.api".
	// For now, we rely on the Exporter's resolution capability which uses visible ancestors.
	// But strictly speaking, we should probably resolve them here if the AST doesn't.

	// Build label
	label := getString(rel.Verb)
	if l := getString(rel.Label); l != "" {
		if label != "" {
			label = label + " [" + l + "]"
		} else {
			label = l
		}
	}

	*relations = append(*relations, &Relation{
		From:  from,
		To:    to,
		Label: label,
	})
}

// calculateNodeSize is deprecated - use MeasureNodeContent instead.
// Kept for backward compatibility during migration.
//
//nolint:unused // Deprecated but kept for backward compatibility
func (e *Exporter) calculateNodeSize(kind, title, technology string) (width, height int) {
	// Base sizes by kind
	switch kind {
	case "person":
		width, height = 200, 180
	case "system":
		width, height = 220, 140
	case "container":
		width, height = 200, 120
	case "component":
		width, height = 180, 100
	case "datastore", "queue":
		width, height = 200, 100
	default:
		width, height = e.Config.DefaultNodeWidth, e.Config.DefaultNodeHeight
	}

	// Adjust width based on title length (rough heuristic: ~8px per char)
	titleWidth := len(title)*8 + 40
	if titleWidth > width {
		if titleWidth > 400 {
			width = 400
		} else {
			width = titleWidth
		}
	}

	// Add height for technology tag
	if technology != "" {
		height += 20
	}

	return width, height
}

// getString safely dereferences a string pointer.
func getString(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}
