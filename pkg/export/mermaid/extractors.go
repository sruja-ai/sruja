package mermaid

import (
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
	kind := ""
	if elem != nil {
		kind = elem.GetKind()
	}
	if elem == nil || (kind != "database" && kind != "Database" && kind != "datastore" && kind != "DataStore") {
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
	if elem == nil || (elem.GetKind() != "queue" && elem.GetKind() != "Queue" && elem.GetKind() != "MessageQueue") {
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
		if item.ElementDef != nil && (item.ElementDef.GetKind() == "person" || item.ElementDef.GetKind() == "Person") {
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

func extractTopLevelContainers(prog *language.Program) []*language.Container {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var containers []*language.Container
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && (item.ElementDef.GetKind() == "container" || item.ElementDef.GetKind() == "Container") {
			if cont := extractContainerFromElement(item.ElementDef); cont != nil {
				containers = append(containers, cont)
			}
		}
	}
	return containers
}

func extractRelationsFromModel(prog *language.Program) []*language.Relation {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var relations []*language.Relation
	for _, item := range prog.Model.Items {
		relations = append(relations, extractRelationsFromItem(item)...)
	}
	return relations
}

func extractRelationsFromItem(item language.ModelItem) []*language.Relation {
	var relations []*language.Relation
	if item.Relation != nil {
		relations = append(relations, item.Relation)
	}
	// Check inside ElementDef body if present
	if item.ElementDef != nil && item.ElementDef.Assignment != nil && item.ElementDef.Assignment.Body != nil {
		for _, bodyItem := range item.ElementDef.Assignment.Body.Items {
			// Convert BodyItem to ModelItem-like structure or extract directly
			// BodyItem has Element *ElementDef, Relation *Relation, etc.
			// Reusing recursive logic needs mapping or duplication.
			// Simple approach: check Relation and recurse ElementDef
			if bodyItem.Relation != nil {
				relations = append(relations, bodyItem.Relation)
			}
			if bodyItem.Element != nil {
				// Construct pseudo ModelItem for recursion, or better, recurse on ElementDef
				nestedRels := extractRelationsFromElement(bodyItem.Element)
				relations = append(relations, nestedRels...)
			}
		}
	}
	return relations
}

func extractRelationsFromElement(elem *language.ElementDef) []*language.Relation {
	var relations []*language.Relation
	if elem.Assignment != nil && elem.Assignment.Body != nil {
		for _, bodyItem := range elem.Assignment.Body.Items {
			if bodyItem.Relation != nil {
				relations = append(relations, bodyItem.Relation)
			}
			if bodyItem.Element != nil {
				relations = append(relations, extractRelationsFromElement(bodyItem.Element)...)
			}
		}
	}
	return relations
}
