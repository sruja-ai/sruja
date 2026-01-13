package views

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// ElementInfo holds information about an element for lookup purposes.
type ElementInfo struct {
	ID       string
	Kind     string
	Label    string
	ParentID string
}

// ElementLookup provides fast lookup of elements by ID and helper methods for navigation.
type ElementLookup struct {
	Elements map[string]*ElementInfo
}

// BuildElementLookup creates an element lookup structure from a program.
func BuildElementLookup(prog *language.Program) *ElementLookup {
	lookup := &ElementLookup{
		Elements: make(map[string]*ElementInfo),
	}

	if prog == nil || prog.Model == nil {
		return lookup
	}

	var extractElement func(elem *language.ElementDef, parentID string)
	extractElement = func(elem *language.ElementDef, parentID string) {
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
		label := ptrToString(elem.GetTitle())
		if label == "" {
			label = id
		}

		lookup.Elements[fullID] = &ElementInfo{
			ID:       fullID,
			Kind:     kind,
			Label:    label,
			ParentID: parentID,
		}

		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					extractElement(item.Element, fullID)
				}
			}
		}
	}

	for _, item := range prog.Model.Items {
		if item.ElementDef != nil {
			extractElement(item.ElementDef, "")
		}
	}

	return lookup
}

// GetRoot finds the root element (system or person) for a given FQN.
func (l *ElementLookup) GetRoot(fqn string) (string, bool) {
	if info, ok := l.Elements[fqn]; ok {
		kind := strings.ToLower(info.Kind)
		if kind == "system" || kind == "person" {
			return fqn, true
		}

		currentID := fqn
		for {
			if info.ParentID == "" {
				kind := strings.ToLower(info.Kind)
				if kind == "system" || kind == "person" {
					return currentID, true
				}
				return currentID, true
			}

			parentInfo, ok := l.Elements[info.ParentID]
			if !ok {
				return currentID, true
			}

			kind := strings.ToLower(parentInfo.Kind)
			if kind == "system" || kind == "person" {
				return info.ParentID, true
			}

			currentID = info.ParentID
			info = parentInfo
		}
	}

	parts := strings.Split(fqn, ".")
	if len(parts) > 0 {
		rootID := parts[0]
		if info, ok := l.Elements[rootID]; ok {
			kind := strings.ToLower(info.Kind)
			if kind == "system" || kind == "person" {
				return rootID, true
			}
		}
	}

	return "", false
}

// GetContainer finds the containing container for a given FQN.
func (l *ElementLookup) GetContainer(fqn string) string {
	if info, ok := l.Elements[fqn]; ok {
		kind := strings.ToLower(info.Kind)
		if kind == "container" || kind == "datastore" || kind == "queue" {
			return fqn
		}

		for currentInfo, ok := info, true; ok && currentInfo.ParentID != ""; currentInfo, ok = l.Elements[currentInfo.ParentID] {
			kind := strings.ToLower(currentInfo.Kind)
			if kind == "container" || kind == "datastore" || kind == "queue" {
				return currentInfo.ID
			}
		}
	}

	parts := strings.Split(fqn, ".")
	for i := len(parts) - 1; i > 0; i-- {
		containerID := strings.Join(parts[:i], ".")
		if info, ok := l.Elements[containerID]; ok {
			kind := strings.ToLower(info.Kind)
			if kind == "container" || kind == "datastore" || kind == "queue" {
				return containerID
			}
		}
	}

	return ""
}

// ResolveFQN resolves a short name to an FQN based on context.
func (l *ElementLookup) ResolveFQN(shortID, contextID string) string {
	if _, ok := l.Elements[shortID]; ok {
		return shortID
	}

	var bestMatch string
	contextScope := ""
	if contextID != "" {
		parts := strings.Split(contextID, ".")
		if len(parts) > 1 {
			contextScope = strings.Join(parts[:len(parts)-1], ".")
		} else {
			contextScope = parts[0]
		}
	}

	for id := range l.Elements {
		if strings.HasSuffix(id, "."+shortID) {
			if contextScope != "" && strings.HasPrefix(id, contextScope+".") {
				bestMatch = id
				break
			}
			if bestMatch == "" {
				bestMatch = id
			}
		}
	}
	if bestMatch != "" {
		return bestMatch
	}

	return shortID
}
