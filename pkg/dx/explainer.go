// pkg/dx/explainer.go
package dx

import (
	"fmt"

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
