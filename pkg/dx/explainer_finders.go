package dx

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// findElement finds an element by ID in Sruja Model.
func (e *Explainer) findElement(id string) interface{} {
	if e.program == nil || e.program.Model == nil {
		return nil
	}

	// Search for element in Sruja Model
	var findElement func(elem *language.ElementDef, currentFQN string) *language.ElementDef
	findElement = func(elem *language.ElementDef, currentFQN string) *language.ElementDef {
		if elem == nil {
			return nil
		}

		elemID := elem.GetID()
		if elemID == "" {
			return nil
		}

		fqn := elemID
		if currentFQN != "" {
			fqn = currentFQN + "." + elemID
		}

		// Check if this is the element we're looking for
		if fqn == id || elemID == id {
			return elem
		}

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					if found := findElement(bodyItem.Element, fqn); found != nil {
						return found
					}
				}
			}
		}

		return nil
	}

	// Search all top-level elements
	for _, item := range e.program.Model.Items {
		if item.ElementDef != nil {
			if found := findElement(item.ElementDef, ""); found != nil {
				return found
			}
		}
	}

	return nil
}

// extractRelationInfo extracts label and verb from a relation.
func extractRelationInfo(rel *language.Relation) (label, verb string) {
	if rel.Label != nil {
		label = *rel.Label
	}
	if rel.Verb != nil {
		verb = *rel.Verb
	}
	return label, verb
}

// processRelation processes a relation and adds it to the RelationsInfo if it matches the elementID.
func processRelation(rel *language.Relation, elementID string, info *RelationsInfo) {
	fromID := rel.From.String()
	toID := rel.To.String()

	if fromID == elementID {
		label, verb := extractRelationInfo(rel)
		info.Outgoing = append(info.Outgoing, &RelationInfo{
			From:      fromID,
			To:        toID,
			Label:     label,
			Type:      verb,
			Direction: "outgoing",
		})
	}
	if toID == elementID {
		label, verb := extractRelationInfo(rel)
		info.Incoming = append(info.Incoming, &RelationInfo{
			From:      fromID,
			To:        toID,
			Label:     label,
			Type:      verb,
			Direction: "incoming",
		})
	}
}

// findRelations finds all relations involving an element.
func (e *Explainer) findRelations(elementID string) RelationsInfo {
	var info RelationsInfo
	if e.program == nil || e.program.Model == nil {
		return info
	}

	// Collect all relations from Sruja Model
	// Use internal helper - we need to make collectElements exported or use it differently
	// For now, collect relations manually
	relations := []*language.Relation{}
	var collectRelations func(elem *language.ElementDef)
	collectRelations = func(elem *language.ElementDef) {
		body := elem.GetBody()
		if body == nil {
			return
		}
		for _, bodyItem := range body.Items {
			if bodyItem.Relation != nil {
				relations = append(relations, bodyItem.Relation)
			}
			if bodyItem.Element != nil {
				collectRelations(bodyItem.Element)
			}
		}
	}
	for _, item := range e.program.Model.Items {
		if item.Relation != nil {
			relations = append(relations, item.Relation)
		}
		if item.ElementDef != nil {
			collectRelations(item.ElementDef)
		}
	}

	// Process all relations
	for _, rel := range relations {
		processRelation(rel, elementID, &info)
	}

	return info
}

// extractMetadata extracts metadata from an element.
func (e *Explainer) extractMetadata(elem interface{}) map[string]string {
	// Estimate capacity: typically 3-8 metadata entries per element
	metadata := make(map[string]string, 8)

	// Handle ElementDef
	if elementDef, ok := elem.(*language.ElementDef); ok {
		body := elementDef.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Metadata != nil {
					for _, entry := range item.Metadata.Entries {
						if entry.Value != nil {
							metadata[entry.Key] = *entry.Value
						}
					}
				}
			}
		}
		return metadata
	}

	// Legacy support for old Architecture types
	switch v := elem.(type) {
	case *language.System:
		for _, meta := range v.Metadata {
			if meta.Value != nil {
				metadata[meta.Key] = *meta.Value
			}
		}
	case *language.Container:
		for _, meta := range v.Metadata {
			if meta.Value != nil {
				metadata[meta.Key] = *meta.Value
			}
		}
	case *language.Component:
		for _, meta := range v.Metadata {
			if meta.Value != nil {
				metadata[meta.Key] = *meta.Value
			}
		}
	}

	return metadata
}

// findRelatedADRs finds ADRs that mention the element.
func (e *Explainer) findRelatedADRs(elementID string) []string {
	var related []string
	if e.program == nil || e.program.Model == nil {
		return related
	}

	// Search for ADRs in Model items (now via ElementDef)
	for _, item := range e.program.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			if a.Kind == "adr" {
				// Simple check: if ADR title mentions the element ID
				if a.Title != nil && strings.Contains(*a.Title, elementID) {
					related = append(related, a.Name)
				}
			}
		}
	}

	return related
}

// findRelatedScenarios finds scenarios that involve the element.
func (e *Explainer) findRelatedScenarios(elementID string) []*ScenarioInfo {
	var related []*ScenarioInfo
	if e.program == nil || e.program.Model == nil {
		return related
	}

	// Search for scenarios in Model items (now via ElementDef)
	for _, item := range e.program.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			if a.Kind == "scenario" || a.Kind == "story" {
				// For now, check if title mentions the element
				if a.Title != nil && strings.Contains(*a.Title, elementID) {
					title := ""
					if a.Title != nil {
						title = *a.Title
					}
					related = append(related, &ScenarioInfo{
						ID:    a.Name,
						Label: title,
						Role:  "participant",
					})
				}
			}
		}
	}

	return related
}

// findDependencies finds all dependencies of an element.
func (e *Explainer) findDependencies(elementID string) []string {
	// Estimate capacity: typically few dependencies per element
	deps := make(map[string]bool, 16)
	relations := e.findRelations(elementID)

	// Add outgoing relation targets as dependencies
	for _, rel := range relations.Outgoing {
		deps[rel.To] = true
	}

	result := make([]string, 0, len(deps))
	for dep := range deps {
		result = append(result, dep)
	}

	return result
}
