// pkg/engine/likec4_helpers.go
// Helper functions for working with LikeC4 AST in validation/scoring
package engine

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// buildQualifiedID builds a qualified ID from parts (e.g., "system.container")
func buildQualifiedID(parts ...string) string {
	if len(parts) == 0 {
		return ""
	}
	if len(parts) == 1 {
		return parts[0]
	}
	totalLen := len(parts) - 1 // for dots
	for _, p := range parts {
		totalLen += len(p)
	}
	buf := make([]byte, 0, totalLen)
	buf = append(buf, parts[0]...)
	for i := 1; i < len(parts); i++ {
		buf = append(buf, '.')
		buf = append(buf, parts[i]...)
	}
	return string(buf)
}

// collectLikeC4Elements collects all element IDs from a LikeC4 Model block
// Returns a map of qualified ID -> true, and a list of all relations
func collectLikeC4Elements(model *language.ModelBlock) (map[string]bool, []*language.Relation) {
	defined := make(map[string]bool)
	relations := []*language.Relation{}

	if model == nil {
		return defined, relations
	}

	// Recursively collect elements and their nested children
	var collectElement func(elem *language.LikeC4ElementDef, parentPrefix string)
	collectElement = func(elem *language.LikeC4ElementDef, parentPrefix string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		qualifiedID := id
		if parentPrefix != "" {
			qualifiedID = buildQualifiedID(parentPrefix, id)
		}
		defined[qualifiedID] = true

		// Collect relations from element body
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Relation != nil {
					relations = append(relations, bodyItem.Relation)
				}
				// Recursively collect nested elements
				if bodyItem.Element != nil {
					collectElement(bodyItem.Element, qualifiedID)
				}
			}
		}
	}

	// Process all items in the model
	for _, item := range model.Items {
		// Collect top-level relations
		if item.Relation != nil {
			relations = append(relations, item.Relation)
		}
		// Collect elements
		if item.ElementDef != nil {
			collectElement(item.ElementDef, "")
		}
	}

	return defined, relations
}

// getElementScope returns the scope (parent qualified ID) for a relation
// by finding which element contains it
func getElementScope(model *language.ModelBlock, relation *language.Relation) string {
	if model == nil || relation == nil {
		return ""
	}

	var findScope func(elem *language.LikeC4ElementDef, currentScope string) string
	findScope = func(elem *language.LikeC4ElementDef, currentScope string) string {
		if elem == nil {
			return ""
		}

		id := elem.GetID()
		if id == "" {
			return ""
		}

		qualifiedID := id
		if currentScope != "" {
			qualifiedID = buildQualifiedID(currentScope, id)
		}

		// Check if this element's body contains the relation
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Relation == relation {
					return qualifiedID
				}
				// Recursively search nested elements
				if bodyItem.Element != nil {
					if scope := findScope(bodyItem.Element, qualifiedID); scope != "" {
						return scope
					}
				}
			}
		}

		return ""
	}

	// Search top-level elements
	for _, item := range model.Items {
		if item.ElementDef != nil {
			if scope := findScope(item.ElementDef, ""); scope != "" {
				return scope
			}
		}
	}

	// If not found in any element, it's a top-level relation
	return ""
}

// collectAllRelations collects all relations from a LikeC4 Model block with their scopes
func collectAllRelations(model *language.ModelBlock) []struct {
	Relation *language.Relation
	Scope    string
} {
	result := []struct {
		Relation *language.Relation
		Scope    string
	}{}

	if model == nil {
		return result
	}

	var collectFromElement func(elem *language.LikeC4ElementDef, parentScope string)
	collectFromElement = func(elem *language.LikeC4ElementDef, parentScope string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		currentScope := id
		if parentScope != "" {
			currentScope = buildQualifiedID(parentScope, id)
		}

		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Relation != nil {
					result = append(result, struct {
						Relation *language.Relation
						Scope    string
					}{Relation: bodyItem.Relation, Scope: currentScope})
				}
				if bodyItem.Element != nil {
					collectFromElement(bodyItem.Element, currentScope)
				}
			}
		}
	}

	// Collect top-level relations and relations from top-level elements
	for _, item := range model.Items {
		if item.Relation != nil {
			result = append(result, struct {
				Relation *language.Relation
				Scope    string
			}{Relation: item.Relation, Scope: ""})
		}
		if item.ElementDef != nil {
			collectFromElement(item.ElementDef, "")
		}
	}

	return result
}
