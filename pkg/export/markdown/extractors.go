package markdown

import (
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
		if item.ElementDef != nil && (item.ElementDef.GetKind() == "system" || item.ElementDef.GetKind() == "System") {
			if sys := extractSystemFromElement(item.ElementDef); sys != nil {
				systems = append(systems, sys)
			}
		}
	}
	return systems
}

func extractSystemFromElement(elem *language.ElementDef) *language.System {
	if elem == nil || (elem.GetKind() != "system" && elem.GetKind() != "System") {
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

func extractContainerFromElement(elem *language.ElementDef) *language.Container {
	if elem == nil || (elem.GetKind() != "container" && elem.GetKind() != "Container") {
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

func extractComponentFromElement(elem *language.ElementDef) *language.Component {
	if elem == nil || (elem.GetKind() != "component" && elem.GetKind() != "Component") {
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

func extractDataStoreFromElement(elem *language.ElementDef) *language.DataStore {
	if elem == nil {
		return nil
	}
	kind := elem.GetKind()
	if kind != "database" && kind != "datastore" && kind != "Database" && kind != "DataStore" {
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

func extractQueueFromElement(elem *language.ElementDef) *language.Queue {
	if elem == nil || (elem.GetKind() != "queue" && elem.GetKind() != "Queue") {
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
		kind := ""
		if item.ElementDef != nil {
			kind = item.ElementDef.GetKind()
		}
		if kind == "person" || kind == "Person" {
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

// RequirementInfo represents minimal requirement info extracted from ElementDef
type RequirementInfo struct {
	ID    string
	Title string
	Type  string
}

func extractRequirementsFromModel(prog *language.Program) []RequirementInfo {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var requirements []RequirementInfo
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			if a.Kind == "requirement" || a.Kind == "Requirement" {
				req := RequirementInfo{
					ID:    a.Name,
					Title: getString(a.Title),
				}
				if a.SubKind != nil {
					req.Type = *a.SubKind
				}
				requirements = append(requirements, req)
			}
		}
	}
	return requirements
}

// ADRInfo represents minimal ADR info extracted from ElementDef
type ADRInfo struct {
	ID    string
	Title string
}

func extractADRsFromModel(prog *language.Program) []ADRInfo {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var adrs []ADRInfo
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			if a.Kind == "adr" || a.Kind == "Adr" || a.Kind == "ADR" {
				adrs = append(adrs, ADRInfo{
					ID:    a.Name,
					Title: getString(a.Title),
				})
			}
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
	var collectFromElement func(elem *language.ElementDef)
	collectFromElement = func(elem *language.ElementDef) {
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

// ScenarioStepInfo represents a step in a scenario or flow
type ScenarioStepInfo struct {
	From        string
	To          string
	Description string
	Tags        []string
}

// ScenarioInfo represents generic info extracted from ElementDef
type ScenarioInfo struct {
	ID          string
	Title       string
	Description string
	Steps       []ScenarioStepInfo
}

// FlowInfo represents generic info extracted from ElementDef
type FlowInfo struct {
	ID          string
	Title       string
	Description string
	Steps       []ScenarioStepInfo
}

// extractScenariosAndFlowsFromModel extracts scenarios and flows from the model
func extractScenariosAndFlowsFromModel(prog *language.Program) ([]ScenarioInfo, []FlowInfo) {
	if prog == nil || prog.Model == nil {
		return nil, nil
	}

	var scenarios []ScenarioInfo
	var flows []FlowInfo

	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			var info ScenarioInfo
			info.ID = a.Name
			info.Title = getString(a.Title)

			// Extract description from body if available (usually first item description or from tags?)
			// ElementDef uses Body for properties.
			body := item.ElementDef.GetBody()
			if body != nil {
				for _, bItem := range body.Items {
					if bItem.Description != nil {
						info.Description = *bItem.Description
					}
					// Check for Steps
					if bItem.Step != nil {
						s := bItem.Step
						step := ScenarioStepInfo{
							From:        strings.Join(s.FromParts, "."),
							To:          strings.Join(s.ToParts, "."),
							Description: getString(s.Description),
							Tags:        s.Tags,
						}
						info.Steps = append(info.Steps, step)
					}
					// Also support generic relations as steps if "step" keyword is missing but Relation is present
					if bItem.Relation != nil {
						r := bItem.Relation
						step := ScenarioStepInfo{
							From:        r.From.String(),
							To:          r.To.String(),
							Description: getString(r.Label), // Label usually acts as desc
							Tags:        r.Tags,
						}
						info.Steps = append(info.Steps, step)
					}
				}
			}

			switch a.Kind {
			case "scenario", "Scenario", "story", "Story":
				scenarios = append(scenarios, info)
			case "flow", "Flow":
				// FlowInfo structure matches ScenarioInfo
				flows = append(flows, FlowInfo(info))
			}
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
