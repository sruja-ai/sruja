package dot

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// elementInfo holds information about an element for lookup purposes.
type elementInfo struct {
	ID       string
	Kind     string
	Label    string
	ParentID string
}

// elementLookup provides fast lookup of elements by ID and helper methods for navigation.
type elementLookup struct {
	elements map[string]*elementInfo
}

// buildElementLookup creates an element lookup structure from a program.
func buildElementLookup(prog *language.Program) *elementLookup {
	lookup := &elementLookup{
		elements: make(map[string]*elementInfo),
	}

	if prog == nil || prog.Model == nil {
		return lookup
	}

	// Recursively extract all elements
	var extractElement func(elem *language.ElementDef, parentID string)
	extractElement = func(elem *language.ElementDef, parentID string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		// Build full ID
		fullID := id
		if parentID != "" {
			fullID = parentID + "." + id
		}

		kind := elem.GetKind()
		label := getString(elem.GetTitle())
		if label == "" {
			label = id
		}

		lookup.elements[fullID] = &elementInfo{
			ID:       fullID,
			Kind:     kind,
			Label:    label,
			ParentID: parentID,
		}

		// Recurse into children
		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					extractElement(item.Element, fullID)
				}
			}
		}
	}

	// Extract from model items
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil {
			extractElement(item.ElementDef, "")
		}
	}

	return lookup
}

// getRoot finds the root element (system or person) for a given FQN.
// Returns the root ID and true if found, empty string and false otherwise.
func (l *elementLookup) getRoot(fqn string) (string, bool) {
	if info, ok := l.elements[fqn]; ok {
		// Check if this is already a root (system or person)
		kind := strings.ToLower(info.Kind)
		if kind == "system" || kind == "person" {
			return fqn, true
		}

		// Walk up the parent chain
		currentID := fqn
		for {
			if info.ParentID == "" {
				// Reached top level, check if it's a system or person
				kind := strings.ToLower(info.Kind)
				if kind == "system" || kind == "person" {
					return currentID, true
				}
				// If not, return the top-level element anyway
				return currentID, true
			}

			parentInfo, ok := l.elements[info.ParentID]
			if !ok {
				// Parent not found, return current
				return currentID, true
			}

			kind := strings.ToLower(parentInfo.Kind)
			if kind == "system" || kind == "person" {
				return info.ParentID, true
			}

			// Continue walking up
			currentID = info.ParentID
			info = parentInfo
		}
	}

	// Try to find by prefix matching
	parts := strings.Split(fqn, ".")
	if len(parts) > 0 {
		// Try first part as root
		rootID := parts[0]
		if info, ok := l.elements[rootID]; ok {
			kind := strings.ToLower(info.Kind)
			if kind == "system" || kind == "person" {
				return rootID, true
			}
		}
	}

	return "", false
}

// getContainer finds the containing container for a given FQN.
// Returns the container ID if found, empty string otherwise.
func (l *elementLookup) getContainer(fqn string) string {
	if info, ok := l.elements[fqn]; ok {
		// Check if this is already a container
		kind := strings.ToLower(info.Kind)
		if kind == "container" {
			return fqn
		}

		// Walk up the parent chain to find container
		currentInfo := info
		for {
			if currentInfo.ParentID == "" {
				break
			}

			parentInfo, ok := l.elements[currentInfo.ParentID]
			if !ok {
				break
			}

			kind := strings.ToLower(parentInfo.Kind)
			if kind == "container" {
				return currentInfo.ParentID
			}

			currentInfo = parentInfo
		}
	}

	// Try to find by prefix matching
	parts := strings.Split(fqn, ".")
	for i := len(parts) - 1; i > 0; i-- {
		containerID := strings.Join(parts[:i], ".")
		if info, ok := l.elements[containerID]; ok {
			kind := strings.ToLower(info.Kind)
			if kind == "container" {
				return containerID
			}
		}
	}

	return ""
}
