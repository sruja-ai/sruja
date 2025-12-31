package lister

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// ListPersons extracts all persons from a Sruja model.
func ListPersons(program *language.Program) []PersonInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []PersonInfo
	var collectPersons func(elem *language.ElementDef, parentPrefix string)
	collectPersons = func(elem *language.ElementDef, parentPrefix string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		qualifiedID := id
		if parentPrefix != "" {
			qualifiedID = parentPrefix + "." + id
		}

		// Check if this is a person
		kind := elem.GetKind()

		if kind == "person" || kind == "Person" || kind == "actor" || kind == "Actor" || kind == "user" || kind == "User" {
			label := id
			if title := elem.GetTitle(); title != nil {
				label = *title
			}
			result = append(result, PersonInfo{
				ID:    qualifiedID,
				Label: label,
			})
		}

		// Recursively collect nested elements
		if body := elem.GetBody(); body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					collectPersons(item.Element, qualifiedID)
				}
			}
		}
	}

	// Process all items in the model
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			collectPersons(item.ElementDef, "")
		}
	}

	return result
}

// ListDataStores extracts all datastores from a Sruja model.
func ListDataStores(program *language.Program) []DataStoreInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []DataStoreInfo
	var collectDataStores func(elem *language.ElementDef, parentPrefix, systemID string)
	collectDataStores = func(elem *language.ElementDef, parentPrefix, systemID string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		qualifiedID := id
		if parentPrefix != "" {
			qualifiedID = parentPrefix + "." + id
		}

		// Update systemID if this is a system
		kind := elem.GetKind()
		if kind == "system" || kind == "System" {
			systemID = qualifiedID
		}

		// Check if this is a datastore/database
		if kind == "database" || kind == "Database" || kind == "datastore" || kind == "Datastore" {
			label := id
			if title := elem.GetTitle(); title != nil {
				label = *title
			}
			result = append(result, DataStoreInfo{
				ID:       qualifiedID,
				Label:    label,
				SystemID: systemID,
			})
		}

		// Recursively collect nested elements
		if body := elem.GetBody(); body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					collectDataStores(item.Element, qualifiedID, systemID)
				}
			}
		}
	}

	// Process all items in the model
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			collectDataStores(item.ElementDef, "", "")
		}
	}

	return result
}

// ListQueues extracts all queues from a Sruja model.
func ListQueues(program *language.Program) []QueueInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []QueueInfo
	var collectQueues func(elem *language.ElementDef, parentPrefix, systemID string)
	collectQueues = func(elem *language.ElementDef, parentPrefix, systemID string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		qualifiedID := id
		if parentPrefix != "" {
			qualifiedID = parentPrefix + "." + id
		}

		// Update systemID if this is a system
		kind := elem.GetKind()
		if kind == "system" || kind == "System" {
			systemID = qualifiedID
		}

		// Check if this is a queue
		if kind == "queue" || kind == "Queue" {
			label := id
			if title := elem.GetTitle(); title != nil {
				label = *title
			}
			result = append(result, QueueInfo{
				ID:       qualifiedID,
				Label:    label,
				SystemID: systemID,
			})
		}

		// Recursively collect nested elements
		if body := elem.GetBody(); body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					collectQueues(item.Element, qualifiedID, systemID)
				}
			}
		}
	}

	// Process all items in the model
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			collectQueues(item.ElementDef, "", "")
		}
	}

	return result
}

// ListScenarios extracts all scenarios from a Sruja model.
func ListScenarios(program *language.Program) []ScenarioInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []ScenarioInfo
	// Scenarios are at the top level of the model via ElementDef
	for _, item := range program.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			if a.Kind == "scenario" || a.Kind == "Scenario" || a.Kind == "story" || a.Kind == "Story" {
				title := ""
				if a.Title != nil {
					title = *a.Title
				}
				result = append(result, ScenarioInfo{
					ID:    a.Name,
					Title: title,
				})
			}
		}
	}
	return result
}

// ListADRs extracts all ADRs from a Sruja model.
func ListADRs(program *language.Program) []ADRInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []ADRInfo
	// ADRs are at the top level of the model via ElementDef
	for _, item := range program.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			if a.Kind == "adr" || a.Kind == "Adr" || a.Kind == "ADR" {
				title := ""
				if a.Title != nil {
					title = *a.Title
				}
				result = append(result, ADRInfo{
					ID:    a.Name,
					Title: title,
				})
			}
		}
	}
	return result
}
