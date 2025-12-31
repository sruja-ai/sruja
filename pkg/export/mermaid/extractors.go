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
		if item.Relation != nil {
			relations = append(relations, item.Relation)
		}
	}
	return relations
}
