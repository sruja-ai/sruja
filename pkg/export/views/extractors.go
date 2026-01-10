package views

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// ExtractAllElements extracts all elements from the program into a flat list.
func ExtractAllElements(prog *language.Program) []*Element {
	if prog == nil || prog.Model == nil {
		return nil
	}

	var elements []*Element

	var extractFromElementDef func(elem *language.ElementDef, parentID string)
	extractFromElementDef = func(elem *language.ElementDef, parentID string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		fullID := id
		if parentID != "" {
			fullID = parentID + "." + id
		}

		kind := strings.ToLower(elem.GetKind())
		switch kind {
		case "database", "db", "storage":
			kind = "datastore"
		case "mq":
			kind = "queue"
		case "actor":
			kind = "person"
		}

		title := ptrToString(elem.GetTitle())
		if title == "" {
			title = id
		}

		var technology string
		var description string
		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Technology != nil && technology == "" {
					technology = *item.Technology
				}
				if item.Description != nil && description == "" {
					description = *item.Description
				}
			}
		}

		newElem := &Element{
			ID:          fullID,
			Kind:        kind,
			Title:       title,
			Technology:  technology,
			Description: description,
			ParentID:    parentID,
		}

		w, h := MeasureNodeContent(newElem)
		newElem.Width = int(w)
		newElem.Height = int(h)

		elements = append(elements, newElem)

		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					extractFromElementDef(bodyItem.Element, fullID)
				}
			}
		}
	}

	for _, item := range prog.Model.Items {
		if item.ElementDef != nil {
			extractFromElementDef(item.ElementDef, "")
		}
	}

	return elements
}

// ExtractRelationsFromModel extracts all relations from the program.
func ExtractRelationsFromModel(prog *language.Program) []*Relation {
	if prog == nil || prog.Model == nil {
		return nil
	}

	var relations []*Relation
	var contextStack []string

	var extractFromItems func(items []language.ModelItem)
	var extractFromBodyItems func(items []*language.BodyItem)

	extractFromItems = func(items []language.ModelItem) {
		for _, item := range items {
			if item.Relation != nil {
				relations = append(relations, processRelationNode(item.Relation))
			}
			if item.ElementDef != nil {
				id := item.ElementDef.GetID()
				if id != "" {
					contextStack = append(contextStack, id)
					if body := item.ElementDef.GetBody(); body != nil {
						extractFromBodyItems(body.Items)
					}
					contextStack = contextStack[:len(contextStack)-1]
				}
			}
		}
	}

	extractFromBodyItems = func(items []*language.BodyItem) {
		for _, item := range items {
			if item.Relation != nil {
				relations = append(relations, processRelationNode(item.Relation))
			}
			if item.Element != nil {
				id := item.Element.GetID()
				if id != "" {
					contextStack = append(contextStack, id)
					if body := item.Element.GetBody(); body != nil {
						extractFromBodyItems(body.Items)
					}
					contextStack = contextStack[:len(contextStack)-1]
				}
			}
		}
	}

	extractFromItems(prog.Model.Items)
	return relations
}

func processRelationNode(rel *language.Relation) *Relation {
	if rel == nil {
		return nil
	}

	label := ptrToString(rel.Verb)
	if l := ptrToString(rel.Label); l != "" {
		if label != "" {
			label = label + " [" + l + "]"
		} else {
			label = l
		}
	}

	if len(rel.Tags) > 0 {
		tags := "[" + strings.Join(rel.Tags, ", ") + "]"
		if label != "" {
			label = label + " " + tags
		} else {
			label = tags
		}
	}

	return &Relation{
		From:  rel.From.String(),
		To:    rel.To.String(),
		Label: label,
	}
}
