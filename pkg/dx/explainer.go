// pkg/dx/explainer.go
package dx

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
	"golang.org/x/text/cases"
	lang "golang.org/x/text/language"
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
	if e.program == nil || e.program.Model == nil {
		return nil, fmt.Errorf("no model found")
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

// findElement finds an element by ID in LikeC4 Model.
func (e *Explainer) findElement(id string) interface{} {
	if e.program == nil || e.program.Model == nil {
		return nil
	}

	// Search for element in LikeC4 Model
	var findElement func(elem *language.LikeC4ElementDef, currentFQN string) *language.LikeC4ElementDef
	findElement = func(elem *language.LikeC4ElementDef, currentFQN string) *language.LikeC4ElementDef {
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

// buildDescription creates a natural-language description of an element.
func (e *Explainer) buildDescription(elem interface{}) string {
	var sb strings.Builder

	// Handle LikeC4ElementDef
	if likeC4Elem, ok := elem.(*language.LikeC4ElementDef); ok {
		id := likeC4Elem.GetID()
		kind := likeC4Elem.GetKind()
		title := id
		t := likeC4Elem.GetTitle()
		if t != nil {
			title = *t
		}

		sb.WriteString(fmt.Sprintf("**%s: %s**\n\n", cases.Title(lang.Und).String(kind), title))

		// Extract description from body
		description := ""
		technology := ""
		nestedCounts := make(map[string]int)
		body := likeC4Elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Description != nil {
					description = *item.Description
				}
				if item.Technology != nil {
					technology = *item.Technology
				}
				if item.Element != nil {
					nestedCounts[item.Element.GetKind()]++
				}
			}
		}

		if description != "" {
			sb.WriteString(fmt.Sprintf("%s\n\n", description))
		}

		// Specific descriptions based on kind
		switch kind {
		case "system":
			sb.WriteString("This is a system that provides functionality. ")
			if count, ok := nestedCounts["container"]; ok && count > 0 {
				sb.WriteString(fmt.Sprintf("It consists of %d container(s). ", count))
			}
			if count, ok := nestedCounts["database"]; ok && count > 0 {
				sb.WriteString(fmt.Sprintf("It uses %d data store(s). ", count))
			}
		case "container":
			sb.WriteString("This is a container application within a system. ")
			if count, ok := nestedCounts["component"]; ok && count > 0 {
				sb.WriteString(fmt.Sprintf("It contains %d component(s). ", count))
			}
		case "component":
			sb.WriteString("This is a component that provides specific functionality. ")
		case "person":
			sb.WriteString("This represents a person who interacts with the architecture. ")
		default:
			sb.WriteString("This is an architecture element. ")
		}

		if technology != "" {
			sb.WriteString(fmt.Sprintf("It uses %s technology. ", technology))
		}

		return sb.String()
	}

	// Legacy support for old Architecture types (if any remain)
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

	// Collect all relations from LikeC4 Model
	// Use internal helper - we need to make collectLikeC4Elements exported or use it differently
	// For now, collect relations manually
	relations := []*language.Relation{}
	var collectRelations func(elem *language.LikeC4ElementDef)
	collectRelations = func(elem *language.LikeC4ElementDef) {
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

	// Handle LikeC4ElementDef
	if likeC4Elem, ok := elem.(*language.LikeC4ElementDef); ok {
		body := likeC4Elem.GetBody()
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
func (e *Explainer) findRelatedADRs(elementID string) []*language.ADR {
	var related []*language.ADR
	if e.program == nil || e.program.Model == nil {
		return related
	}

	// Search for ADRs in Model items
	for _, item := range e.program.Model.Items {
		if item.ADR != nil {
			// Simple check: if ADR title mentions the element ID
			if item.ADR.Title != nil && strings.Contains(*item.ADR.Title, elementID) {
				related = append(related, item.ADR)
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

	// Search for scenarios in Model items
	for _, item := range e.program.Model.Items {
		if item.Scenario != nil {
			for _, step := range item.Scenario.Steps {
				// step.From and step.To are QualifiedIdent - use String() for comparison
				if step.From.String() == elementID || step.To.String() == elementID {
					title := ""
					if item.Scenario.Title != nil {
						title = *item.Scenario.Title
					}
					related = append(related, &ScenarioInfo{
						ID:    item.Scenario.ID,
						Label: title,
						Role:  "participant",
					})
					break
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

// Format formats an element explanation as a human-readable string.
func (exp *ElementExplanation) Format() string {
	var sb strings.Builder

	// Description
	sb.WriteString(exp.Description)
	sb.WriteString("\n")

	exp.formatMetadata(&sb)
	exp.formatRelations(&sb)
	exp.formatDependencies(&sb)
	exp.formatADRs(&sb)
	exp.formatScenarios(&sb)

	return sb.String()
}

func (exp *ElementExplanation) formatMetadata(sb *strings.Builder) {
	if len(exp.Metadata) == 0 {
		return
	}
	sb.WriteString("\n## Metadata\n\n")
	for key, value := range exp.Metadata {
		fmt.Fprintf(sb, "- **%s**: %s\n", key, value)
	}
	sb.WriteString("\n")
}

func (exp *ElementExplanation) formatRelations(sb *strings.Builder) {
	if len(exp.Relations.Incoming) == 0 && len(exp.Relations.Outgoing) == 0 {
		return
	}
	sb.WriteString("## Relations\n\n")
	if len(exp.Relations.Incoming) > 0 {
		sb.WriteString("### Incoming\n\n")
		for _, rel := range exp.Relations.Incoming {
			fmt.Fprintf(sb, "- **%s** → %s", rel.From, exp.ID)
			if rel.Label != "" {
				fmt.Fprintf(sb, " (%s)", rel.Label)
			}
			sb.WriteString("\n")
		}
		sb.WriteString("\n")
	}
	if len(exp.Relations.Outgoing) > 0 {
		sb.WriteString("### Outgoing\n\n")
		for _, rel := range exp.Relations.Outgoing {
			fmt.Fprintf(sb, "- %s → **%s**", exp.ID, rel.To)
			if rel.Label != "" {
				fmt.Fprintf(sb, " (%s)", rel.Label)
			}
			sb.WriteString("\n")
		}
		sb.WriteString("\n")
	}
}

func (exp *ElementExplanation) formatDependencies(sb *strings.Builder) {
	if len(exp.Dependencies) == 0 {
		return
	}
	sb.WriteString("## Dependencies\n\n")
	for _, dep := range exp.Dependencies {
		fmt.Fprintf(sb, "- %s\n", dep)
	}
	sb.WriteString("\n")
}

func (exp *ElementExplanation) formatADRs(sb *strings.Builder) {
	if len(exp.ADRs) == 0 {
		return
	}
	sb.WriteString("## Related ADRs\n\n")
	for _, adr := range exp.ADRs {
		title := ""
		if adr.Title != nil {
			title = *adr.Title
		}
		fmt.Fprintf(sb, "- **%s**: %s\n", adr.ID, title)
	}
	sb.WriteString("\n")
}

func (exp *ElementExplanation) formatScenarios(sb *strings.Builder) {
	if len(exp.Scenarios) == 0 {
		return
	}
	sb.WriteString("## Related Scenarios\n\n")
	for _, scenario := range exp.Scenarios {
		fmt.Fprintf(sb, "- **%s**: %s\n", scenario.ID, scenario.Label)
	}
	sb.WriteString("\n")
}
