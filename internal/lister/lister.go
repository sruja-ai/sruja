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

// ListSystems extracts all systems from an architecture.
func ListSystems(arch *language.Architecture) []SystemInfo {
	if arch == nil {
		return nil
	}

	result := make([]SystemInfo, 0, len(arch.Systems))
	for _, sys := range arch.Systems {
		info := SystemInfo{
			ID:    sys.ID,
			Label: sys.Label,
		}
		if sys.Description != nil {
			info.Description = *sys.Description
		}
		result = append(result, info)
	}
	return result
}

// ListContainers extracts all containers from an architecture.
func ListContainers(arch *language.Architecture) []ContainerInfo {
	if arch == nil {
		return nil
	}

	var result []ContainerInfo
	for _, sys := range arch.Systems {
		for _, cont := range sys.Containers {
			info := ContainerInfo{
				ID:       cont.ID,
				Label:    cont.Label,
				SystemID: sys.ID,
			}
			if cont.Description != nil {
				info.Description = *cont.Description
			}
			result = append(result, info)
		}
	}
	return result
}

// ListComponents extracts all components from an architecture.
func ListComponents(arch *language.Architecture) []ComponentInfo {
	if arch == nil {
		return nil
	}

	var result []ComponentInfo
	for _, sys := range arch.Systems {
		// System-level components
		for _, comp := range sys.Components {
			info := ComponentInfo{
				ID:       comp.ID,
				Label:    comp.Label,
				SystemID: sys.ID,
			}
			if comp.Description != nil {
				info.Description = *comp.Description
			}
			result = append(result, info)
		}
		// Container-level components
		for _, cont := range sys.Containers {
			for _, comp := range cont.Components {
				info := ComponentInfo{
					ID:          comp.ID,
					Label:       comp.Label,
					SystemID:    sys.ID,
					ContainerID: cont.ID,
				}
				if comp.Description != nil {
					info.Description = *comp.Description
				}
				result = append(result, info)
			}
		}
	}
	return result
}

// ListPersons extracts all persons from an architecture.
func ListPersons(arch *language.Architecture) []PersonInfo {
	if arch == nil {
		return nil
	}

	result := make([]PersonInfo, 0, len(arch.Persons))
	for _, person := range arch.Persons {
		result = append(result, PersonInfo{
			ID:    person.ID,
			Label: person.Label,
		})
	}
	for _, sys := range arch.Systems {
		for _, person := range sys.Persons {
			result = append(result, PersonInfo{
				ID:    person.ID,
				Label: person.Label,
			})
		}
	}
	return result
}

// ListDataStores extracts all datastores from an architecture.
func ListDataStores(arch *language.Architecture) []DataStoreInfo {
	if arch == nil {
		return nil
	}

	var result []DataStoreInfo
	for _, sys := range arch.Systems {
		for _, ds := range sys.DataStores {
			result = append(result, DataStoreInfo{
				ID:       ds.ID,
				Label:    ds.Label,
				SystemID: sys.ID,
			})
		}
		for _, cont := range sys.Containers {
			for _, ds := range cont.DataStores {
				result = append(result, DataStoreInfo{
					ID:       ds.ID,
					Label:    ds.Label,
					SystemID: sys.ID,
				})
			}
		}
	}
	return result
}

// ListQueues extracts all queues from an architecture.
func ListQueues(arch *language.Architecture) []QueueInfo {
	if arch == nil {
		return nil
	}

	var result []QueueInfo
	for _, sys := range arch.Systems {
		for _, q := range sys.Queues {
			result = append(result, QueueInfo{
				ID:       q.ID,
				Label:    q.Label,
				SystemID: sys.ID,
			})
		}
		for _, cont := range sys.Containers {
			for _, q := range cont.Queues {
				result = append(result, QueueInfo{
					ID:       q.ID,
					Label:    q.Label,
					SystemID: sys.ID,
				})
			}
		}
	}
	return result
}

// ListScenarios extracts all scenarios from an architecture.
func ListScenarios(arch *language.Architecture) []ScenarioInfo {
	if arch == nil {
		return nil
	}

	result := make([]ScenarioInfo, 0, len(arch.Scenarios))
	for _, scenario := range arch.Scenarios {
		result = append(result, ScenarioInfo{
			ID:    scenario.ID,
			Title: scenario.Title,
		})
	}
	return result
}

// ListADRs extracts all ADRs from an architecture.
func ListADRs(arch *language.Architecture) []ADRInfo {
	if arch == nil {
		return nil
	}

	result := make([]ADRInfo, 0, len(arch.ADRs))
	for _, adr := range arch.ADRs {
		title := ""
		if adr.Title != nil {
			title = *adr.Title
		}
		result = append(result, ADRInfo{
			ID:    adr.ID,
			Title: title,
		})
	}
	return result
}
