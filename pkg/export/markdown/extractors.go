package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// Helper functions to extract elements from Model
func extractSystemsFromModel(prog *language.Program) []*language.System {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var systems []*language.System
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "system" {
			if sys := extractSystemFromElement(item.ElementDef); sys != nil {
				systems = append(systems, sys)
			}
		}
	}
	return systems
}

func extractSystemFromElement(elem *language.LikeC4ElementDef) *language.System {
	if elem == nil || elem.GetKind() != "system" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	sys := &language.System{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
	body := elem.GetBody()
	if body != nil {
		for _, bodyItem := range body.Items {
			if bodyItem.Description != nil {
				sys.Description = bodyItem.Description
			}
			if bodyItem.Element != nil {
				if cont := extractContainerFromElement(bodyItem.Element); cont != nil {
					sys.Containers = append(sys.Containers, cont)
				}
				if ds := extractDataStoreFromElement(bodyItem.Element); ds != nil {
					sys.DataStores = append(sys.DataStores, ds)
				}
				if q := extractQueueFromElement(bodyItem.Element); q != nil {
					sys.Queues = append(sys.Queues, q)
				}
			}
		}
	}
	return sys
}

func extractContainerFromElement(elem *language.LikeC4ElementDef) *language.Container {
	if elem == nil || elem.GetKind() != "container" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	cont := &language.Container{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
	body := elem.GetBody()
	if body != nil {
		for _, bodyItem := range body.Items {
			if bodyItem.Description != nil {
				cont.Description = bodyItem.Description
			}
			if bodyItem.Element != nil {
				if comp := extractComponentFromElement(bodyItem.Element); comp != nil {
					cont.Components = append(cont.Components, comp)
				}
			}
		}
	}
	return cont
}

func extractComponentFromElement(elem *language.LikeC4ElementDef) *language.Component {
	if elem == nil || elem.GetKind() != "component" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	return &language.Component{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
}

func extractDataStoreFromElement(elem *language.LikeC4ElementDef) *language.DataStore {
	if elem == nil {
		return nil
	}
	kind := elem.GetKind()
	if kind != "database" && kind != "datastore" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	return &language.DataStore{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
}

func extractQueueFromElement(elem *language.LikeC4ElementDef) *language.Queue {
	if elem == nil || elem.GetKind() != "queue" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	return &language.Queue{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
}

func extractPersonsFromModel(prog *language.Program) []*language.Person {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var persons []*language.Person
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "person" {
			id := item.ElementDef.GetID()
			if id != "" {
				persons = append(persons, &language.Person{
					ID:    id,
					Label: getString(item.ElementDef.GetTitle()),
				})
			}
		}
	}
	return persons
}

func extractRequirementsFromModel(prog *language.Program) []*language.Requirement {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var requirements []*language.Requirement
	for _, item := range prog.Model.Items {
		if item.Requirement != nil {
			requirements = append(requirements, item.Requirement)
		}
	}
	return requirements
}

func extractADRsFromModel(prog *language.Program) []*language.ADR {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var adrs []*language.ADR
	for _, item := range prog.Model.Items {
		if item.ADR != nil {
			adrs = append(adrs, item.ADR)
		}
	}
	return adrs
}

func getString(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

// extractRelationsFromModel extracts all relations from the model
func extractRelationsFromModel(prog *language.Program) []*language.Relation {
	if prog == nil || prog.Model == nil {
		return nil
	}

	var relations []*language.Relation

	// Collect top-level relations
	for _, item := range prog.Model.Items {
		if item.Relation != nil {
			relations = append(relations, item.Relation)
		}
	}

	// Collect relations from element bodies
	var collectFromElement func(elem *language.LikeC4ElementDef)
	collectFromElement = func(elem *language.LikeC4ElementDef) {
		if elem == nil {
			return
		}
		body := elem.GetBody()
		if body == nil {
			return
		}

		for _, bodyItem := range body.Items {
			if bodyItem.Relation != nil {
				relations = append(relations, bodyItem.Relation)
			}
			if bodyItem.Element != nil {
				collectFromElement(bodyItem.Element)
			}
		}
	}

	for _, item := range prog.Model.Items {
		if item.ElementDef != nil {
			collectFromElement(item.ElementDef)
		}
	}

	return relations
}

// prioritizeRelationships sorts relationships by importance
// Priority: direct relationships > relationships with labels > simple references
func prioritizeRelationships(relations []*language.Relation) []*language.Relation {
	if len(relations) == 0 {
		return relations
	}

	// Create a copy to avoid modifying original
	prioritized := make([]*language.Relation, len(relations))
	copy(prioritized, relations)

	// Simple prioritization: relationships with labels/verbs are more important
	// This is a basic implementation - can be enhanced with more sophisticated scoring
	for i := 0; i < len(prioritized)-1; i++ {
		for j := i + 1; j < len(prioritized); j++ {
			scoreI := relationshipScore(prioritized[i])
			scoreJ := relationshipScore(prioritized[j])
			if scoreJ > scoreI {
				prioritized[i], prioritized[j] = prioritized[j], prioritized[i]
			}
		}
	}

	return prioritized
}

// relationshipScore calculates importance score for a relationship
func relationshipScore(rel *language.Relation) int {
	score := 0

	// Relationships with labels are more important
	if rel.Label != nil && *rel.Label != "" {
		score += 10
	}

	// Relationships with verbs are more important
	if rel.Verb != nil && *rel.Verb != "" {
		score += 5
	}
	if rel.VerbRaw != nil && rel.VerbRaw.Value != "" {
		score += 5
	}

	// Relationships with tags are more important
	if len(rel.Tags) > 0 {
		score += len(rel.Tags) * 2
	}

	return score
}

// extractScenariosAndFlowsFromModel extracts scenarios and flows from the model
func extractScenariosAndFlowsFromModel(prog *language.Program) ([]*language.Scenario, []*language.Flow) {
	if prog == nil || prog.Model == nil {
		return nil, nil
	}

	var scenarios []*language.Scenario
	var flows []*language.Flow

	for _, item := range prog.Model.Items {
		if item.Scenario != nil {
			scenarios = append(scenarios, item.Scenario)
		}
		if item.Flow != nil {
			flows = append(flows, item.Flow)
		}
	}

	return scenarios, flows
}

// Helper functions for diagram generation

func sanitizeIDForMermaid(id string) string {
	return strings.Map(func(r rune) rune {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') {
			return r
		}
		return '_'
	}, id)
}

func escapeQuotesForMermaid(s string) string {
	return strings.ReplaceAll(s, "\"", "#quot;")
}

func formatLabelForMermaid(label, id, description, technology string) string {
	main := label
	if main == "" {
		main = id
	}

	res := main
	if technology != "" {
		res += fmt.Sprintf("\n(%s)", technology)
	}
	if description != "" {
		// Truncate description for Mermaid
		desc := description
		if len(desc) > 50 {
			desc = desc[:47] + "..."
		}
		res += fmt.Sprintf("\n%s", desc)
	}
	return res
}

// extractTermsFromText extracts potential glossary terms (simplified implementation)
func extractTermsFromText(text string, terms map[string]string) {
	// This is a simplified implementation
	// In a production system, you'd use NLP or a predefined glossary
	// For now, we'll look for common acronyms and technical terms

	// Common patterns: ALL_CAPS words, words in quotes, etc.
	words := strings.Fields(text)
	for _, word := range words {
		// Remove punctuation
		word = strings.Trim(word, ".,;:!?()[]{}\"'")
		if len(word) > 2 {
			// Check for acronyms (all caps, 2+ chars)
			if strings.ToUpper(word) == word && len(word) >= 2 {
				if _, exists := terms[word]; !exists {
					terms[word] = "Acronym or technical term (definition to be added)"
				}
			}
		}
	}
}
