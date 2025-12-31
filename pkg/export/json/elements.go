package json

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// convertElementsFromModel converts Sruja Model elements to ElementDump
func (e *Exporter) convertElementsFromModel(dump *SrujaModelDump, model *language.Model) {
	if model == nil {
		return
	}

	var convertElement func(elem *language.ElementDef, parentFQN string)
	convertElement = func(elem *language.ElementDef, parentFQN string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		fqn := id
		if parentFQN != "" {
			fqn = parentFQN + "." + id
		}

		// Extract title, description, technology from element
		title := ""
		t := elem.GetTitle()
		if t != nil {
			title = *t
		}
		description := ""
		technology := ""
		var metadata []*language.MetaEntry

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
			}
		}

		kind := elem.GetKind()
		elementDump := ElementDump{
			ID:          fqn,
			Kind:        kind,
			Title:       title,
			Description: description,
			Technology:  technology,
			Metadata:    metaToMap(metadata),
			Parent:      parentFQN,
		}

		dump.Elements[fqn] = elementDump

		// Recursively process nested elements
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					convertElement(bodyItem.Element, fqn)
				}
			}
		}
	}

	// Process all top-level elements
	for _, item := range model.Items {
		if item.ElementDef != nil {
			convertElement(item.ElementDef, "")
		}
	}
}

// convertRelationsFromModel converts Sruja Model relations to RelationDump
func (e *Exporter) convertRelationsFromModel(dump *SrujaModelDump, model *language.Model) {
	if model == nil {
		return
	}

	relIndex := 0

	// Helper to resolve a QualifiedIdent to FQN based on context
	resolveFQN := func(qid language.QualifiedIdent, contextFQN string) string {
		if contextFQN == "" {
			return qid.String()
		}

		if len(qid.Parts) > 1 {
			firstPart := qid.Parts[0]
			if firstPart == contextFQN || strings.HasPrefix(contextFQN, firstPart+".") {
				return qid.String()
			}
		}

		// Fast path for joining context and qid
		var sb strings.Builder
		sb.Grow(len(contextFQN) + 1 + len(qid.Parts)*8) // Estimate
		sb.WriteString(contextFQN)
		sb.WriteByte('.')
		for i, part := range qid.Parts {
			if i > 0 {
				sb.WriteByte('.')
			}
			sb.WriteString(part)
		}
		return sb.String()
	}

	var collectRelations func(elem *language.ElementDef, contextFQN string)
	collectRelations = func(elem *language.ElementDef, contextFQN string) {
		if elem == nil {
			return
		}

		// Get the FQN for this element to use as context for nested relations
		var elemFQN string
		id := elem.GetID()
		if id != "" {
			if contextFQN != "" {
				elemFQN = contextFQN + "." + id
			} else {
				elemFQN = id
			}
		} else {
			elemFQN = contextFQN
		}

		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Relation != nil {
					rel := bodyItem.Relation
					fromFQN := resolveFQN(rel.From, elemFQN)
					toFQN := resolveFQN(rel.To, elemFQN)
					// Extract title: prefer Label, then Verb, then VerbRaw
					title := ptrToString(rel.Label)
					if title == "" {
						title = strVal(rel.Verb)
					}
					if title == "" && rel.VerbRaw != nil {
						title = rel.VerbRaw.Value
					}

					dump.Relations = append(dump.Relations, RelationDump{
						ID:          fmt.Sprintf("rel-%d", relIndex),
						Source:      NewFqnRef(fromFQN),
						Target:      NewFqnRef(toFQN),
						Title:       title,
						Description: strVal(rel.Verb), // Keep Verb in Description if needed, or swap based on semantics
					})
					relIndex++
				}
				if bodyItem.Element != nil {
					collectRelations(bodyItem.Element, elemFQN)
				}
			}
		}
	}

	// Pre-allocate relations slice based on model items as a starting capacity
	if len(model.Items) > 0 && len(dump.Relations) == 0 {
		dump.Relations = make([]RelationDump, 0, len(model.Items))
	}

	// Collect top-level relations and relations from elements
	for _, item := range model.Items {
		if item.Relation != nil {
			// Top-level relations are already fully qualified or relative to root
			fromFQN := item.Relation.From.String()
			toFQN := item.Relation.To.String()

			// Extract title: prefer Label, then Verb, then VerbRaw
			title := ptrToString(item.Relation.Label)
			if title == "" {
				title = strVal(item.Relation.Verb)
			}
			if title == "" && item.Relation.VerbRaw != nil {
				title = item.Relation.VerbRaw.Value
			}

			dump.Relations = append(dump.Relations, RelationDump{
				ID:          fmt.Sprintf("rel-%d", relIndex),
				Source:      NewFqnRef(fromFQN),
				Target:      NewFqnRef(toFQN),
				Title:       title,
				Description: strVal(item.Relation.Verb),
			})
			relIndex++
		}
		if item.ElementDef != nil {
			// Start with empty context for top-level elements
			collectRelations(item.ElementDef, "")
		}
	}
}

// Helper functions
func ptrToString(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

func metaToMap(meta []*language.MetaEntry) map[string]string {
	if len(meta) == 0 {
		return nil
	}
	m := make(map[string]string)
	for _, e := range meta {
		if e.Value != nil {
			m[e.Key] = *e.Value
		} else if len(e.Array) > 0 {
			m[e.Key] = strings.Join(e.Array, ",")
		}
	}
	return m
}

func extractTechnology(c *language.Container) *string {
	// Look for technology in items
	for _, item := range c.Items {
		if item.Technology != nil {
			return item.Technology
		}
	}
	return nil
}
