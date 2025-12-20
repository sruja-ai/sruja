// Package lister provides pure functions for listing architecture elements.
package lister

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// SystemInfo represents a system for listing.
type SystemInfo struct {
	ID          string
	Label       string
	Description string
}

// ContainerInfo represents a container for listing.
type ContainerInfo struct {
	ID          string
	Label       string
	SystemID    string
	Description string
}

// ComponentInfo represents a component for listing.
type ComponentInfo struct {
	ID          string
	Label       string
	SystemID    string
	ContainerID string
	Description string
}

// PersonInfo represents a person for listing.
type PersonInfo struct {
	ID    string
	Label string
}

// DataStoreInfo represents a datastore for listing.
type DataStoreInfo struct {
	ID       string
	Label    string
	SystemID string
}

// QueueInfo represents a queue for listing.
type QueueInfo struct {
	ID       string
	Label    string
	SystemID string
}

// ScenarioInfo represents a scenario for listing.
type ScenarioInfo struct {
	ID    string
	Title string
}

// ADRInfo represents an ADR for listing.
type ADRInfo struct {
	ID    string
	Title string
}

// ListSystems extracts all systems from a LikeC4 model.
func ListSystems(program *language.Program) []SystemInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []SystemInfo
	var collectSystems func(elem *language.LikeC4ElementDef, parentPrefix string)
	collectSystems = func(elem *language.LikeC4ElementDef, parentPrefix string) {
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

		// Check if this is a system
		if elem.GetKind() == "system" {
			label := id
			if title := elem.GetTitle(); title != nil {
				label = *title
			}
			info := SystemInfo{
				ID:    qualifiedID,
				Label: label,
			}
			if body := elem.GetBody(); body != nil {
				for _, item := range body.Items {
					if item.Description != nil {
						info.Description = *item.Description
						break
					}
				}
			}
			result = append(result, info)
		}

		// Recursively collect nested elements
		if body := elem.GetBody(); body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					collectSystems(item.Element, qualifiedID)
				}
			}
		}
	}

	// Process all items in the model
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			collectSystems(item.ElementDef, "")
		}
	}

	return result
}

// ListContainers extracts all containers from a LikeC4 model.
func ListContainers(program *language.Program) []ContainerInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []ContainerInfo
	var collectContainers func(elem *language.LikeC4ElementDef, parentPrefix, systemID string)
	collectContainers = func(elem *language.LikeC4ElementDef, parentPrefix, systemID string) {
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
		if elem.GetKind() == "system" {
			systemID = qualifiedID
		}

		// Check if this is a container
		kind := elem.GetKind()
		if kind == "container" || kind == "webapp" || kind == "mobile" || kind == "api" || kind == "database" || kind == "queue" {
			label := id
			if title := elem.GetTitle(); title != nil {
				label = *title
			}
			info := ContainerInfo{
				ID:       qualifiedID,
				Label:    label,
				SystemID: systemID,
			}
			if body := elem.GetBody(); body != nil {
				for _, item := range body.Items {
					if item.Description != nil {
						info.Description = *item.Description
						break
					}
				}
			}
			result = append(result, info)
		}

		// Recursively collect nested elements
		if body := elem.GetBody(); body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					collectContainers(item.Element, qualifiedID, systemID)
				}
			}
		}
	}

	// Process all items in the model
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			collectContainers(item.ElementDef, "", "")
		}
	}

	return result
}

// ListComponents extracts all components from a LikeC4 model.
func ListComponents(program *language.Program) []ComponentInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []ComponentInfo
	var collectComponents func(elem *language.LikeC4ElementDef, parentPrefix, systemID, containerID string)
	collectComponents = func(elem *language.LikeC4ElementDef, parentPrefix, systemID, containerID string) {
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
		if kind == "system" {
			systemID = qualifiedID
			containerID = "" // Reset container ID when entering a new system
		}

		// Update containerID if this is a container
		if kind == "container" || kind == "webapp" || kind == "mobile" || kind == "api" {
			containerID = qualifiedID
		}

		// Check if this is a component
		if kind == "component" {
			label := id
			if title := elem.GetTitle(); title != nil {
				label = *title
			}
			info := ComponentInfo{
				ID:          qualifiedID,
				Label:       label,
				SystemID:    systemID,
				ContainerID: containerID,
			}
			if body := elem.GetBody(); body != nil {
				for _, item := range body.Items {
					if item.Description != nil {
						info.Description = *item.Description
						break
					}
				}
			}
			result = append(result, info)
		}

		// Recursively collect nested elements
		if body := elem.GetBody(); body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					collectComponents(item.Element, qualifiedID, systemID, containerID)
				}
			}
		}
	}

	// Process all items in the model
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			collectComponents(item.ElementDef, "", "", "")
		}
	}

	return result
}

// ListPersons extracts all persons from a LikeC4 model.
func ListPersons(program *language.Program) []PersonInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []PersonInfo
	var collectPersons func(elem *language.LikeC4ElementDef, parentPrefix string)
	collectPersons = func(elem *language.LikeC4ElementDef, parentPrefix string) {
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
		if kind == "person" || kind == "actor" || kind == "user" {
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

// ListDataStores extracts all datastores from a LikeC4 model.
func ListDataStores(program *language.Program) []DataStoreInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []DataStoreInfo
	var collectDataStores func(elem *language.LikeC4ElementDef, parentPrefix, systemID string)
	collectDataStores = func(elem *language.LikeC4ElementDef, parentPrefix, systemID string) {
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
		if kind == "system" {
			systemID = qualifiedID
		}

		// Check if this is a datastore/database
		if kind == "database" || kind == "datastore" {
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

// ListQueues extracts all queues from a LikeC4 model.
func ListQueues(program *language.Program) []QueueInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []QueueInfo
	var collectQueues func(elem *language.LikeC4ElementDef, parentPrefix, systemID string)
	collectQueues = func(elem *language.LikeC4ElementDef, parentPrefix, systemID string) {
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
		if kind == "system" {
			systemID = qualifiedID
		}

		// Check if this is a queue
		if kind == "queue" {
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

// ListScenarios extracts all scenarios from a LikeC4 model.
func ListScenarios(program *language.Program) []ScenarioInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []ScenarioInfo
	// Scenarios are at the top level of the model, not nested in elements
	for _, item := range program.Model.Items {
		if item.Scenario != nil {
			title := ""
			if item.Scenario.Title != nil {
				title = *item.Scenario.Title
			}
			result = append(result, ScenarioInfo{
				ID:    item.Scenario.ID,
				Title: title,
			})
		}
	}
	return result
}

// ListADRs extracts all ADRs from a LikeC4 model.
func ListADRs(program *language.Program) []ADRInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	var result []ADRInfo
	// ADRs are at the top level of the model, not nested in elements
	for _, item := range program.Model.Items {
		if item.ADR != nil {
			title := ""
			if item.ADR.Title != nil {
				title = *item.ADR.Title
			}
			result = append(result, ADRInfo{
				ID:    item.ADR.ID,
				Title: title,
			})
		}
	}
	return result
}
