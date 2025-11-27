// pkg/compiler/transformer.go
// Package compiler provides compilation from model to various diagram formats.
package compiler

import (
	"fmt"
	"time"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/model"
)

// Transformer transforms AST (Abstract Syntax Tree) to Model.
//
// The transformer converts the language-specific AST (which reflects DSL syntax)
// into the language-agnostic Model (which is JSON-serializable and can be compiled
// to various diagram formats).
//
// This transformation:
//   - Flattens hierarchical structure (systems contain containers, but model has flat element list)
//   - Preserves source locations (for error reporting)
//   - Maps AST types to Model types
//   - Handles all element types (systems, containers, components, etc.)
//
// Example usage:
//
//	transformer := compiler.NewTransformer()
//	model, err := transformer.Transform(program)
//	if err != nil {
//	    log.Fatal(err)
//	}
type Transformer struct {
}

// NewTransformer creates a new transformer.
func NewTransformer() *Transformer {
	return &Transformer{}
}

// Transform transforms an AST Program to a Model.
//
// This is the main transformation function that converts the parsed AST into
// the canonical model format. The transformation:
//
//  1. Creates a new Model with version and timestamp
//  2. Transforms all systems, containers, and components to flat Element list
//  3. Transforms all relations to flat Relation list
//  4. Transforms requirements and ADRs
//  5. Preserves source locations for error reporting
//
// Parameters:
//   - program: The parsed AST program
//
// Returns:
//   - *model.Model: The transformed model
//   - error: Transformation error if any
//
// Example:
//
//	parser, _ := language.NewParser()
//	program, _ := parser.Parse("example.sruja", dslText)
//	transformer := compiler.NewTransformer()
//	model, err := transformer.Transform(program)
func (t *Transformer) Transform(program *language.Program) (*model.Model, error) {
	// Validate input
	if program == nil || program.Architecture == nil {
		return nil, fmt.Errorf("program or architecture is nil")
	}

	arch := program.Architecture
	interp := func(s string) string {
		return s // Variable interpolation removed for now
	}

	// Create new model with metadata
	m := &model.Model{
		Version:     "1.0",      // Model version
		GeneratedAt: time.Now(), // Generation timestamp
		Architecture: &model.Architecture{
			Elements:     []model.Element{},
			Relations:    []model.Relation{},
			Requirements: []model.Requirement{},
			ADRs:         []model.ADR{},
			Journeys:     []model.Journey{},
		},
	}

	// Set architecture title
	m.Architecture.Name = arch.Name
	m.Architecture.Description = "" // Description not in new structure

	// Transform systems (from post-processed fields)
	for _, sys := range arch.Systems {
		loc := sys.Location()
		elem := model.Element{
			Type:        model.ElementTypeSystem,
			ID:          interp(sys.ID),
			Name:        interp(sys.Label),
			Description: t.stringPtrToString(sys.Description),
			Technology:  "",
			Tags:        []string{},
			Location: model.Location{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			},
		}
		m.Architecture.Elements = append(m.Architecture.Elements, elem)

		// Transform containers within this system
		for _, cont := range sys.Containers {
			contLoc := cont.Location()
			contTags := []string{}
			contTech := ""
			// Extract technology and tags from container items
			for _, item := range cont.Items {
				if item.Tags != nil {
					contTags = append(contTags, item.Tags...)
				}
				if item.Technology != nil {
					contTech = *item.Technology
				}
			}
			contElem := model.Element{
				Type:        model.ElementTypeContainer,
				ID:          interp(cont.ID),
				Name:        interp(cont.Label),
				Description: t.stringPtrToString(cont.Description),
				Technology:  interp(contTech),
				Tags:        contTags,
				Location: model.Location{
					File:   contLoc.File,
					Line:   contLoc.Line,
					Column: contLoc.Column,
				},
			}
			m.Architecture.Elements = append(m.Architecture.Elements, contElem)

			// Transform components within this container
			for _, comp := range cont.Components {
				compLoc := comp.Location()
				compTech := ""
				if comp.Technology != nil {
					compTech = *comp.Technology
				}
				compElem := model.Element{
					Type:        model.ElementTypeComponent,
					ID:          interp(comp.ID),
					Name:        interp(comp.Label),
					Description: t.stringPtrToString(comp.Description),
					Technology:  interp(compTech),
					Tags:        []string{},
					Location: model.Location{
						File:   compLoc.File,
						Line:   compLoc.Line,
						Column: compLoc.Column,
					},
				}
				m.Architecture.Elements = append(m.Architecture.Elements, compElem)
			}
		}

		// Transform data stores within this system
		for _, ds := range sys.DataStores {
			dsLoc := ds.Location()
			el := model.Element{
				Type:        model.ElementTypeDataStore,
				ID:          interp(ds.ID),
				Name:        interp(ds.Label),
				Description: t.stringPtrToString(ds.Description),
				Technology:  "",
				Tags:        []string{},
				Location:    model.Location{File: dsLoc.File, Line: dsLoc.Line, Column: dsLoc.Column},
			}
			m.Architecture.Elements = append(m.Architecture.Elements, el)
		}

		// Transform queues within this system
		for _, q := range sys.Queues {
			qLoc := q.Location()
			el := model.Element{
				Type:        model.ElementTypeQueue,
				ID:          interp(q.ID),
				Name:        interp(q.Label),
				Description: t.stringPtrToString(q.Description),
				Technology:  "",
				Tags:        []string{},
				Location:    model.Location{File: qLoc.File, Line: qLoc.Line, Column: qLoc.Column},
			}
			m.Architecture.Elements = append(m.Architecture.Elements, el)
		}
	}

	// Transform persons (from architecture level)
	for _, per := range arch.Persons {
		loc := per.Location()
		el := model.Element{
			Type:        model.ElementTypePerson,
			ID:          interp(per.ID),
			Name:        interp(per.Label),
			Description: "",
			Technology:  "",
			Tags:        []string{},
			Location:    model.Location{File: loc.File, Line: loc.Line, Column: loc.Column},
		}
		m.Architecture.Elements = append(m.Architecture.Elements, el)
	}

	// Transform relations (from architecture and system levels)
	for _, rel := range arch.Relations {
		relType := t.mapRelationTypePtr(rel.Verb)
		relLoc := rel.Location()
		r := model.Relation{
			From:        interp(rel.From),
			To:          interp(rel.To),
			Type:        relType,
			Description: t.stringPtrToString(rel.Label),
			Location: model.Location{
				File:   relLoc.File,
				Line:   relLoc.Line,
				Column: relLoc.Column,
			},
		}
		m.Architecture.Relations = append(m.Architecture.Relations, r)
	}

	// Transform requirements
	for _, req := range arch.Requirements {
		reqType := t.mapRequirementType(req.Type)
		reqLoc := req.Location()
		reqm := model.Requirement{
			ID:          req.ID,
			Type:        reqType,
			Description: interp(req.Description),
			Implements:  []string{}, // Implements field removed from AST
			Location: model.Location{
				File:   reqLoc.File,
				Line:   reqLoc.Line,
				Column: reqLoc.Column,
			},
		}
		m.Architecture.Requirements = append(m.Architecture.Requirements, reqm)
	}

	// Transform ADRs
	for _, adr := range arch.ADRs {
		adrLoc := adr.Location()
		adrm := model.ADR{
			ID:           adr.ID,
			Title:        interp(adr.Title),
			Context:      "",
			Decision:     "",
			Consequences: []string{},
			Status:       model.ADRStatusProposed, // Default status
			Location: model.Location{
				File:   adrLoc.File,
				Line:   adrLoc.Line,
				Column: adrLoc.Column,
			},
		}
		m.Architecture.ADRs = append(m.Architecture.ADRs, adrm)
	}

	// Transform journeys
	for _, journey := range arch.Journeys {
		journeyLoc := journey.Location()
		// Transform journey steps to model relations
		steps := []model.Relation{}
		for _, step := range journey.Steps {
			stepLoc := step.Location()
			// Handle different arrow directions
			from := interp(step.From)
			to := interp(step.To)
			label := t.stringPtrToString(step.Label)

			if step.Arrow == "->" {
				steps = append(steps, model.Relation{
					From:        from,
					To:          to,
					Type:        model.RelationTypeUses,
					Description: label,
					Location: model.Location{
						File:   stepLoc.File,
						Line:   stepLoc.Line,
						Column: stepLoc.Column,
					},
				})
			} else if step.Arrow == "<-" {
				steps = append(steps, model.Relation{
					From:        to,
					To:          from,
					Type:        model.RelationTypeUses,
					Description: label,
					Location: model.Location{
						File:   stepLoc.File,
						Line:   stepLoc.Line,
						Column: stepLoc.Column,
					},
				})
			} else if step.Arrow == "<->" {
				// Bidirectional: add both directions
				steps = append(steps, model.Relation{
					From:        from,
					To:          to,
					Type:        model.RelationTypeUses,
					Description: label,
					Location: model.Location{
						File:   stepLoc.File,
						Line:   stepLoc.Line,
						Column: stepLoc.Column,
					},
				})
				steps = append(steps, model.Relation{
					From:        to,
					To:          from,
					Type:        model.RelationTypeUses,
					Description: label,
					Location: model.Location{
						File:   stepLoc.File,
						Line:   stepLoc.Line,
						Column: stepLoc.Column,
					},
				})
			}
		}
		jm := model.Journey{
			ID:          journey.ID,
			Title:       interp(journey.Title),
			Description: "",
			Steps:       steps,
			Location: model.Location{
				File:   journeyLoc.File,
				Line:   journeyLoc.Line,
				Column: journeyLoc.Column,
			},
		}
		m.Architecture.Journeys = append(m.Architecture.Journeys, jm)
	}

	return m, nil
}

// stringPtrToString converts *string to string, returning empty string if nil.
func (t *Transformer) stringPtrToString(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

// mapRelationTypePtr maps AST relation verb (*string) to model RelationType.
func (t *Transformer) mapRelationTypePtr(verb *string) model.RelationType {
	if verb == nil {
		return model.RelationTypeUses
	}
	return t.mapRelationType(*verb)
}

// mapRelationType maps AST relation type string to model RelationType.
//
// The AST uses string types (e.g., "uses", "depends") which are mapped to
// strongly-typed RelationType constants in the model.
//
// If the AST type is unknown, defaults to RelationTypeUses.
func (t *Transformer) mapRelationType(astType string) model.RelationType {
	switch astType {
	case "depends":
		return model.RelationTypeDepends
	case "publishes":
		return model.RelationTypePublishes
	case "subscribes":
		return model.RelationTypeSubscribes
	case "reads":
		return model.RelationTypeReads
	case "writes":
		return model.RelationTypeWrites
	default:
		return model.RelationTypeUses
	}
}

// mapRequirementType maps AST requirement type string to model RequirementType.
//
// The AST uses string types (e.g., "functional", "constraint") which are mapped to
// strongly-typed RequirementType constants in the model.
//
// If the AST type is unknown, defaults to RequirementTypeFunctional.
func (t *Transformer) mapRequirementType(astType string) model.RequirementType {
	switch astType {
	case "functional":
		return model.RequirementTypeFunctional
	case "constraint":
		return model.RequirementTypeConstraint
	case "performance":
		return model.RequirementTypePerformance
	case "security":
		return model.RequirementTypeSecurity
	default:
		// Default to Functional for unknown types
		return model.RequirementTypeFunctional
	}
}

// mapADRStatus maps AST ADR status string to model ADRStatus.
//
// The AST uses string types (e.g., "proposed", "accepted") which are mapped to
// strongly-typed ADRStatus constants in the model.
//
// If the AST status is unknown, defaults to ADRStatusProposed.
func (t *Transformer) mapADRStatus(astStatus string) model.ADRStatus {
	switch astStatus {
	case "accepted":
		return model.ADRStatusAccepted
	case "rejected":
		return model.ADRStatusRejected
	case "deprecated":
		return model.ADRStatusDeprecated
	default:
		return model.ADRStatusProposed
	}
}
