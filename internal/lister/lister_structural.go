package lister

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

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
