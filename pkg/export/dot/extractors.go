package dot

import (
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

	var extractFromElementDef func(elem *language.LikeC4ElementDef, parentID string)
	extractFromElementDef = func(elem *language.LikeC4ElementDef, parentID string) {
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

		kind := elem.GetKind()
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

		// Calculate dimensions based on kind and content
		width, height := e.calculateNodeSize(kind, title, technology)

		elements = append(elements, &Element{
			ID:         fullID,
			Kind:       kind,
			Title:      title,
			Technology: technology,
			ParentID:   parentID,
			Width:      width,
			Height:     height,
		})

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
	for _, item := range prog.Model.Items {
		if item.Relation != nil {
			rel := item.Relation

			// Build label from Verb or Label
			// Verb is like "calls", Label is like "HTTP" (technology/description)
			label := getString(rel.Verb)
			if l := getString(rel.Label); l != "" {
				if label != "" {
					label = label + " [" + l + "]"
				} else {
					label = l
				}
			}

			relations = append(relations, &Relation{
				From:  rel.From.String(),
				To:    rel.To.String(),
				Label: label,
			})
		}
	}

	return relations
}

// calculateNodeSize calculates node dimensions based on content.
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
