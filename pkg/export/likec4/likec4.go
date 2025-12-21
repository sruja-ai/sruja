package likec4

import (
	"encoding/json"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// Exporter handles LikeC4 model generation.
type Exporter struct{}

// NewExporter creates a new LikeC4 exporter.
func NewExporter() *Exporter {
	return &Exporter{}
}

// Export converts a LikeC4 Program to LikeC4 JSON format.
func (e *Exporter) Export(program *language.Program) ([]byte, error) {
	if program == nil {
		return json.Marshal(&Model{})
	}

	model := e.ToModel(program)
	return json.MarshalIndent(model, "", "  ")
}

// ExportCompact exports without indentation for smaller output.
func (e *Exporter) ExportCompact(program *language.Program) ([]byte, error) {
	if program == nil {
		return json.Marshal(&Model{})
	}

	model := e.ToModel(program)
	return json.Marshal(model)
}

// ToModel converts a LikeC4 Program to a LikeC4 Model struct.
func (e *Exporter) ToModel(program *language.Program) *Model {
	if program == nil || program.Model == nil {
		return &Model{}
	}

	modelBlock := program.Model

	model := &Model{
		Elements:  make([]Element, 0),
		Relations: make([]Relation, 0),
		Views:     make([]View, 0, 1),
	}

	// Convert elements from Model block
	var convertElement func(elem *language.LikeC4ElementDef, parentID string)
	convertElement = func(elem *language.LikeC4ElementDef, parentID string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		fqn := id
		if parentID != "" {
			fqn = buildFQN(parentID, id)
		}

		// Extract title, description, technology, metadata
		title := ""
		titlePtr := elem.GetTitle()
		if titlePtr != nil {
			title = *titlePtr
		}
		description := ""
		technology := ""
		var metadata []*language.MetaEntry
		var sloBlock *language.SLOBlock
		var scaleBlock *language.ScaleBlock

		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Description != nil {
					description = *item.Description
				}
				if item.Technology != nil {
					technology = *item.Technology
				}
				if item.Metadata != nil {
					metadata = item.Metadata.Entries
				}
				if item.SLO != nil {
					sloBlock = item.SLO
					// Post-process SLO to extract typed fields
					sloBlock.PostProcess()
				}
				if item.Scale != nil {
					scaleBlock = item.Scale
					scaleBlock.PostProcess()
				}
			}
		}

		kind := elem.GetKind()
		meta := convertMetadata(metadata)
		if sloBlock != nil {
			if meta == nil {
				meta = make(map[string]interface{})
			}
			sloData := make(map[string]interface{})
			if sloBlock.Availability != nil && sloBlock.Availability.Target != nil {
				sloData["availability"] = *sloBlock.Availability.Target
			}
			if sloBlock.Latency != nil && sloBlock.Latency.P99 != nil {
				sloData["latency_p99"] = *sloBlock.Latency.P99
			}
			if len(sloData) > 0 {
				meta["slo"] = sloData
			}
		}
		if scaleBlock != nil {
			if meta == nil {
				meta = make(map[string]interface{})
			}
			scaleData := make(map[string]interface{})
			if scaleBlock.Min != nil {
				scaleData["min"] = *scaleBlock.Min
			}
			if scaleBlock.Max != nil {
				scaleData["max"] = *scaleBlock.Max
			}
			if scaleBlock.Metric != nil {
				scaleData["metric"] = *scaleBlock.Metric
			}
			if len(scaleData) > 0 {
				meta["scale"] = scaleData
			}
		}

		element := Element{
			ID:          fqn,
			Kind:        kind,
			Title:       getLabel(title, id),
			Description: description,
			Technology:  technology,
			Parent:      parentID,
			Metadata:    meta,
		}

		model.Elements = append(model.Elements, element)

		// Recursively process nested elements
		body = elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					convertElement(bodyItem.Element, fqn)
				}
			}
		}
	}

	// Process all top-level elements
	for _, item := range modelBlock.Items {
		if item.ElementDef != nil {
			convertElement(item.ElementDef, "")
		}
	}

	// Collect relations
	var collectRelations func(elem *language.LikeC4ElementDef)
	collectRelations = func(elem *language.LikeC4ElementDef) {
		if elem == nil {
			return
		}
		body := elem.GetBody()
		if body == nil {
			return
		}

		for _, bodyItem := range body.Items {
			if bodyItem.Relation != nil {
				model.Relations = append(model.Relations, e.convertRelation(bodyItem.Relation))
			}
			if bodyItem.Element != nil {
				collectRelations(bodyItem.Element)
			}
		}
	}

	for _, item := range modelBlock.Items {
		if item.Relation != nil {
			model.Relations = append(model.Relations, e.convertRelation(item.Relation))
		}
		if item.ElementDef != nil {
			collectRelations(item.ElementDef)
		}
	}

	// Create default view
	if len(model.Elements) > 0 {
		model.Views = append(model.Views, View{
			ID:      "default",
			Title:   "Model",
			Include: []string{"*"},
		})
	}

	return model
}

func (e *Exporter) convertRelation(rel *language.Relation) Relation {
	title := getString(rel.Label)
	if title == "" {
		title = getString(rel.Verb)
	}

	return Relation{
		Source: rel.From.String(),
		Target: rel.To.String(),
		Title:  title,
		Tags:   rel.Tags,
	}
}

// Helper functions

func getString(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

func getLabel(label, id string) string {
	if label != "" {
		return label
	}
	return id
}

func buildFQN(parentID, id string) string {
	if parentID == "" {
		return id
	}
	if strings.Contains(id, parentID) {
		return id
	}
	return parentID + "." + id
}

func getTechnology(props map[string]string) string {
	if props == nil {
		return ""
	}
	if tech, ok := props["technology"]; ok {
		return tech
	}
	return ""
}

func convertMetadata(metadata []*language.MetaEntry) map[string]interface{} {
	if len(metadata) == 0 {
		return nil
	}

	result := make(map[string]interface{})
	for _, m := range metadata {
		if m.Value != nil {
			result[m.Key] = *m.Value
		} else if len(m.Array) > 0 {
			result[m.Key] = m.Array
		}
	}
	return result
}
