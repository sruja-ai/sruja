// pkg/dx/explainer.go
package dx

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// Explainer provides natural-language explanations of architecture elements.
//
// It generates ChatGPT-style explanations that help developers understand
// their architecture models quickly and intuitively.
type Explainer struct {
	program *language.Program
}

// NewExplainer creates a new explainer for a program.
func NewExplainer(program *language.Program) *Explainer {
	return &Explainer{program: program}
}

// ExplainElement provides a comprehensive explanation of an element.
//
// It includes:
// - Purpose and description
// - Incoming relations
// - Outgoing relations
// - Metadata (SLO, owner, team)
// - Related ADRs
// - Journey participation
// - Dependencies
func (e *Explainer) ExplainElement(elementID string) (*ElementExplanation, error) {
	if e.program == nil || e.program.Architecture == nil {
		return nil, fmt.Errorf("no architecture found")
	}

	explanation := &ElementExplanation{
		ID: elementID,
	}

	// Find the element
	elem := e.findElement(elementID)
	if elem == nil {
		return nil, fmt.Errorf("element '%s' not found", elementID)
	}

	explanation.Element = elem
	explanation.Description = e.buildDescription(elem)
	explanation.Relations = e.findRelations(elementID)
	explanation.Metadata = e.extractMetadata(elem)
	explanation.ADRs = e.findRelatedADRs(elementID)
	explanation.Scenarios = e.findRelatedScenarios(elementID)
	explanation.Dependencies = e.findDependencies(elementID)

	return explanation, nil
}

// ElementExplanation contains a comprehensive explanation of an element.
type ElementExplanation struct {
	ID           string
	Element      interface{} // System, Container, Component, Person, etc.
	Description  string
	Relations    RelationsInfo
	Metadata     map[string]string
	ADRs         []*language.ADR
	Scenarios    []*ScenarioInfo
	Dependencies []string
}

// RelationsInfo describes incoming and outgoing relations.
type RelationsInfo struct {
	Incoming []*RelationInfo
	Outgoing []*RelationInfo
}

// RelationInfo describes a single relation.
type RelationInfo struct {
	From      string
	To        string
	Label     string
	Type      string
	Direction string
}

// ScenarioInfo describes scenario participation.
type ScenarioInfo struct {
	ID    string
	Label string
	Role  string // "actor", "system", "step"
}

// findElement finds an element by ID.
func (e *Explainer) findElement(id string) interface{} {
	arch := e.program.Architecture

	// Search systems
	for _, sys := range arch.Systems {
		if sys.ID == id {
			return sys
		}
		// Search containers in system
		for _, cont := range sys.Containers {
			if cont.ID == id {
				return cont
			}
			// Search components in container
			for _, comp := range cont.Components {
				if comp.ID == id {
					return comp
				}
			}
		}
		// Search components directly in system
		for _, comp := range sys.Components {
			if comp.ID == id {
				return comp
			}
		}
		// Search datastores
		for _, ds := range sys.DataStores {
			if ds.ID == id {
				return ds
			}
		}
		// Search queues
		for _, q := range sys.Queues {
			if q.ID == id {
				return q
			}
		}
	}

	// Search persons
	for _, person := range arch.Persons {
		if person.ID == id {
			return person
		}
	}

	return nil
}

// buildDescription creates a natural-language description of an element.
func (e *Explainer) buildDescription(elem interface{}) string {
	var sb strings.Builder

	switch v := elem.(type) {
	case *language.System:
		sb.WriteString(fmt.Sprintf("**System: %s**\n\n", v.Label))
		if v.Description != nil {
			sb.WriteString(fmt.Sprintf("%s\n\n", *v.Description))
		}
		sb.WriteString(fmt.Sprintf("This is a system that provides %s functionality. ", strings.ToLower(v.Label)))
		if len(v.Containers) > 0 {
			sb.WriteString(fmt.Sprintf("It consists of %d container(s). ", len(v.Containers)))
		}
		if len(v.DataStores) > 0 {
			sb.WriteString(fmt.Sprintf("It uses %d data store(s). ", len(v.DataStores)))
		}

	case *language.Container:
		sb.WriteString(fmt.Sprintf("**Container: %s**\n\n", v.Label))
		if v.Description != nil {
			sb.WriteString(fmt.Sprintf("%s\n\n", *v.Description))
		}
		sb.WriteString("This is a container application within a system. ")
		if len(v.Components) > 0 {
			sb.WriteString(fmt.Sprintf("It contains %d component(s). ", len(v.Components)))
		}

	case *language.Component:
		sb.WriteString(fmt.Sprintf("**Component: %s**\n\n", v.Label))
		if v.Description != nil {
			sb.WriteString(fmt.Sprintf("%s\n\n", *v.Description))
		}
		sb.WriteString("This is a component that provides specific functionality. ")
		if v.Technology != nil {
			sb.WriteString(fmt.Sprintf("It uses %s technology. ", *v.Technology))
		}

	case *language.Person:
		sb.WriteString(fmt.Sprintf("**Person: %s**\n\n", v.Label))
		sb.WriteString("This represents a person who interacts with the architecture. ")

	default:
		sb.WriteString("This is an architecture element. ")
	}

	return sb.String()
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
	fromID := rel.From
	toID := rel.To

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
	arch := e.program.Architecture

	for _, sys := range arch.Systems {
		for _, rel := range sys.Relations {
			processRelation(rel, elementID, &info)
		}

		// Check containers
		for _, cont := range sys.Containers {
			for _, rel := range cont.Relations {
				processRelation(rel, elementID, &info)
			}
		}

		// Check components
		for _, comp := range sys.Components {
			for _, rel := range comp.Relations {
				processRelation(rel, elementID, &info)
			}
		}
	}

	// Check architecture-level relations
	for _, rel := range arch.Relations {
		processRelation(rel, elementID, &info)
	}

	return info
}

// extractMetadata extracts metadata from an element.
func (e *Explainer) extractMetadata(elem interface{}) map[string]string {
	metadata := make(map[string]string)

	switch v := elem.(type) {
	case *language.System:
		for _, meta := range v.Metadata {
			metadata[meta.Key] = meta.Value
		}
	case *language.Container:
		for _, meta := range v.Metadata {
			metadata[meta.Key] = meta.Value
		}
	case *language.Component:
		for _, meta := range v.Metadata {
			metadata[meta.Key] = meta.Value
		}
	}

	return metadata
}

// findRelatedADRs finds ADRs that mention the element.
func (e *Explainer) findRelatedADRs(elementID string) []*language.ADR {
	var related []*language.ADR
	arch := e.program.Architecture

	for _, adr := range arch.ADRs {
		// Simple check: if ADR title mentions the element ID
		if adr.Title != nil && strings.Contains(*adr.Title, elementID) {
			related = append(related, adr)
		}
	}

	return related
}

// findRelatedScenarios finds scenarios that involve the element.
func (e *Explainer) findRelatedScenarios(elementID string) []*ScenarioInfo {
	var related []*ScenarioInfo
	arch := e.program.Architecture

	for _, scenario := range arch.Scenarios {
		for _, step := range scenario.Steps {
			if step.From == elementID || step.To == elementID {
				related = append(related, &ScenarioInfo{
					ID:    scenario.Title, // Use title as ID since scenario ID is optional
					Label: scenario.Title,
					Role:  "participant",
				})
				break
			}
		}
	}

	return related
}

// findDependencies finds all dependencies of an element.
func (e *Explainer) findDependencies(elementID string) []string {
	deps := make(map[string]bool)
	relations := e.findRelations(elementID)

	// Add outgoing relation targets as dependencies
	for _, rel := range relations.Outgoing {
		deps[rel.To] = true
	}

	var result []string
	for dep := range deps {
		result = append(result, dep)
	}

	return result
}

// Format formats an element explanation as a human-readable string.
func (exp *ElementExplanation) Format() string {
	var sb strings.Builder

	// Description
	sb.WriteString(exp.Description)
	sb.WriteString("\n")

	// Metadata
	if len(exp.Metadata) > 0 {
		sb.WriteString("\n## Metadata\n\n")
		for key, value := range exp.Metadata {
			sb.WriteString(fmt.Sprintf("- **%s**: %s\n", key, value))
		}
		sb.WriteString("\n")
	}

	// Relations
	if len(exp.Relations.Incoming) > 0 || len(exp.Relations.Outgoing) > 0 {
		sb.WriteString("## Relations\n\n")
		if len(exp.Relations.Incoming) > 0 {
			sb.WriteString("### Incoming\n\n")
			for _, rel := range exp.Relations.Incoming {
				sb.WriteString(fmt.Sprintf("- **%s** → %s", rel.From, exp.ID))
				if rel.Label != "" {
					sb.WriteString(fmt.Sprintf(" (%s)", rel.Label))
				}
				sb.WriteString("\n")
			}
			sb.WriteString("\n")
		}
		if len(exp.Relations.Outgoing) > 0 {
			sb.WriteString("### Outgoing\n\n")
			for _, rel := range exp.Relations.Outgoing {
				sb.WriteString(fmt.Sprintf("- %s → **%s**", exp.ID, rel.To))
				if rel.Label != "" {
					sb.WriteString(fmt.Sprintf(" (%s)", rel.Label))
				}
				sb.WriteString("\n")
			}
			sb.WriteString("\n")
		}
	}

	// Dependencies
	if len(exp.Dependencies) > 0 {
		sb.WriteString("## Dependencies\n\n")
		for _, dep := range exp.Dependencies {
			sb.WriteString(fmt.Sprintf("- %s\n", dep))
		}
		sb.WriteString("\n")
	}

	// ADRs
	if len(exp.ADRs) > 0 {
		sb.WriteString("## Related ADRs\n\n")
		for _, adr := range exp.ADRs {
			title := ""
			if adr.Title != nil {
				title = *adr.Title
			}
			sb.WriteString(fmt.Sprintf("- **%s**: %s\n", adr.ID, title))
		}
		sb.WriteString("\n")
	}

	// Scenarios
	if len(exp.Scenarios) > 0 {
		sb.WriteString("## Related Scenarios\n\n")
		for _, scenario := range exp.Scenarios {
			sb.WriteString(fmt.Sprintf("- **%s**: %s\n", scenario.ID, scenario.Label))
		}
		sb.WriteString("\n")
	}

	return sb.String()
}
