// pkg/export/views/views.go
// Package views provides utilities for working with view definitions from DSL.
package views

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// ApplyViewExpressions filters elements based on view expressions.
// Returns a set of element IDs that should be included in the view.
func ApplyViewExpressions(arch *language.Architecture, view *language.View) (map[string]bool, error) {
	if view == nil {
		return nil, fmt.Errorf("view is nil")
	}

	included := make(map[string]bool)
	excluded := make(map[string]bool)

	// Process expressions in order
	for _, expr := range view.Expressions {
		if expr.Type == "include" {
			if expr.Wildcard != nil && *expr.Wildcard == "*" {
				// Include all elements in scope
				includeAllInScope(arch, view.Scope, included)
			} else if len(expr.Elements) > 0 {
				// Include specific elements
				for _, elem := range expr.Elements {
					included[elem.String()] = true
				}
			} else if expr.Pattern != nil {
				// Pattern-based include (e.g., "->Element->")
				// For now, treat as wildcard - full pattern matching can be added later
				includeAllInScope(arch, view.Scope, included)
			}
		} else if expr.Type == "exclude" {
			if len(expr.Elements) > 0 {
				for _, elem := range expr.Elements {
					excluded[elem.String()] = true
				}
			}
		}
	}

	// Apply exclusions
	for elem := range excluded {
		delete(included, elem)
	}

	return included, nil
}

// includeAllInScope includes all elements within the view scope.
func includeAllInScope(arch *language.Architecture, scope language.QualifiedIdent, included map[string]bool) {
	scopeStr := scope.String()

	// Find the system
	for _, sys := range arch.Systems {
		if sys.ID == scopeStr {
			// Include system
			included[sys.ID] = true

			// Include all containers
			for _, cont := range sys.Containers {
				included[sys.ID+"."+cont.ID] = true
			}

			// Include all data stores
			for _, ds := range sys.DataStores {
				included[sys.ID+"."+ds.ID] = true
			}

			// Include all queues
			for _, q := range sys.Queues {
				included[sys.ID+"."+q.ID] = true
			}

			// Include all components
			for _, cont := range sys.Containers {
				for _, comp := range cont.Components {
					included[sys.ID+"."+cont.ID+"."+comp.ID] = true
				}
			}
			return
		}
	}
}

// ApplyStyles applies styles from the views block to elements.
// Returns a map of element IDs to style properties.
func ApplyStyles(arch *language.Architecture, viewBlock *language.ViewBlock) map[string]map[string]string {
	if viewBlock == nil || viewBlock.Styles == nil {
		return nil
	}

	styles := make(map[string]map[string]string)

	// Build tag-to-elements map
	tagMap := buildTagMap(arch)

	// Apply styles by tag
	for _, style := range viewBlock.Styles.Styles {
		tag := strings.Trim(style.Tag, "\"")
		elements, ok := tagMap[tag]
		if !ok {
			continue
		}

		// Build style properties map
		props := make(map[string]string)
		for _, prop := range style.Properties {
			if prop.Value != nil {
				props[prop.Key] = strings.Trim(*prop.Value, "\"")
			}
		}

		// Apply to all elements with this tag
		for elemID := range elements {
			if styles[elemID] == nil {
				styles[elemID] = make(map[string]string)
			}
			for k, v := range props {
				styles[elemID][k] = v
			}
		}
	}

	return styles
}

// buildTagMap builds a map of tags to element IDs.
func buildTagMap(arch *language.Architecture) map[string]map[string]bool {
	tagMap := make(map[string]map[string]bool)

	// Add default tags
	addTag(tagMap, "Element", arch.Systems, func(s *language.System) string { return s.ID })
	addTag(tagMap, "System", arch.Systems, func(s *language.System) string { return s.ID })
	addTag(tagMap, "Person", arch.Persons, func(p *language.Person) string { return p.ID })

	// Add tags from metadata
	for _, sys := range arch.Systems {
		for _, meta := range sys.Metadata {
			if meta.Key == "tags" && len(meta.Array) > 0 {
				for _, tag := range meta.Array {
					addTagToElement(tagMap, tag, sys.ID)
				}
			}
		}

		for _, cont := range sys.Containers {
			contID := sys.ID + "." + cont.ID
			for _, meta := range cont.Metadata {
				if meta.Key == "tags" && len(meta.Array) > 0 {
					for _, tag := range meta.Array {
						addTagToElement(tagMap, tag, contID)
					}
				}
			}
		}

		for _, ds := range sys.DataStores {
			dsID := sys.ID + "." + ds.ID
			for _, meta := range ds.Metadata {
				if meta.Key == "tags" && len(meta.Array) > 0 {
					for _, tag := range meta.Array {
						addTagToElement(tagMap, tag, dsID)
					}
				}
			}
		}
	}

	return tagMap
}

func addTag[T any](tagMap map[string]map[string]bool, tag string, items []T, getId func(T) string) {
	for _, item := range items {
		addTagToElement(tagMap, tag, getId(item))
	}
}

func addTagToElement(tagMap map[string]map[string]bool, tag, elemID string) {
	if tagMap[tag] == nil {
		tagMap[tag] = make(map[string]bool)
	}
	tagMap[tag][elemID] = true
}

// FindViewByName finds a view by name in the views block.
func FindViewByName(arch *language.Architecture, viewName string) *language.View {
	if arch.Views == nil {
		return nil
	}

	for _, view := range arch.Views.Views {
		if strings.Trim(view.Name, "\"") == viewName {
			return view
		}
	}

	return nil
}

// GetAutolayoutDirection returns the autolayout direction from a view, or default.
func GetAutolayoutDirection(view *language.View, defaultDir string) string {
	if view != nil && view.Autolayout != nil {
		dir := strings.ToUpper(*view.Autolayout)
		// Map lr/tb to LR/TB
		switch dir {
		case "LR":
			return "LR"
		case "TB":
			return "TB"
		case "AUTO":
			return defaultDir
		default:
			return defaultDir
		}
	}
	return defaultDir
}
