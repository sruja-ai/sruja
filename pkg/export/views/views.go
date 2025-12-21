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
func ApplyViewExpressions(prog *language.Program, view *language.View) (map[string]bool, error) {
	if view == nil {
		return nil, fmt.Errorf("view is nil")
	}

	// Estimate capacity based on view expressions
	estimatedElements := len(view.Expressions) * 10
	if estimatedElements < 32 {
		estimatedElements = 32
	}
	included := make(map[string]bool, estimatedElements)
	excluded := make(map[string]bool, estimatedElements/2)

	// Process expressions in order
	for _, expr := range view.Expressions {
		switch expr.Type {
		case "include":
			switch {
			case expr.Wildcard != nil && *expr.Wildcard == "*":
				// Include all elements in scope
				includeAllInScope(prog, view.Scope, included)
			case len(expr.Elements) > 0:
				// Include specific elements
				for _, elem := range expr.Elements {
					included[elem.String()] = true
				}
			case expr.Pattern != nil:
				// Pattern-based include (e.g., "->Element->")
				// For now, treat as wildcard - full pattern matching can be added later
				includeAllInScope(prog, view.Scope, included)
			}
		case "exclude":
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
func includeAllInScope(prog *language.Program, scope language.QualifiedIdent, included map[string]bool) {
	scopeStr := scope.String()
	systems := extractSystems(prog)

	// Find the system
	for _, sys := range systems {
		if sys.ID == scopeStr {
			// Include system
			included[sys.ID] = true

			// Helper to build qualified IDs efficiently
			buildQualifiedID := func(parts ...string) string {
				if len(parts) == 0 {
					return ""
				}
				if len(parts) == 1 {
					return parts[0]
				}
				totalLen := len(parts) - 1 // for dots
				for _, p := range parts {
					totalLen += len(p)
				}
				buf := make([]byte, 0, totalLen)
				buf = append(buf, parts[0]...)
				for i := 1; i < len(parts); i++ {
					buf = append(buf, '.')
					buf = append(buf, parts[i]...)
				}
				return string(buf)
			}

			// Include all containers
			for _, cont := range sys.Containers {
				included[buildQualifiedID(sys.ID, cont.ID)] = true
			}

			// Include all data stores
			for _, ds := range sys.DataStores {
				included[buildQualifiedID(sys.ID, ds.ID)] = true
			}

			// Include all queues
			for _, q := range sys.Queues {
				included[buildQualifiedID(sys.ID, q.ID)] = true
			}

			// Include all components
			for _, cont := range sys.Containers {
				for _, comp := range cont.Components {
					included[buildQualifiedID(sys.ID, cont.ID, comp.ID)] = true
				}
			}
			return
		}
	}
}

// ApplyStyles applies styles from the views block to elements.
// Returns a map of element IDs to style properties.
func ApplyStyles(prog *language.Program, viewBlock *language.ViewBlock) map[string]map[string]string {
	if viewBlock == nil || viewBlock.Styles == nil {
		return nil
	}

	// Estimate capacity: typically few styles per view block
	estimatedStyles := 16
	if viewBlock.Styles != nil && len(viewBlock.Styles.Styles) > 0 {
		estimatedStyles = len(viewBlock.Styles.Styles) * 5
	}
	styles := make(map[string]map[string]string, estimatedStyles)

	// Build tag-to-elements map
	tagMap := buildTagMap(prog)

	// Apply styles by tag
	for _, style := range viewBlock.Styles.Styles {
		tag := strings.Trim(style.Tag, "\"")
		elements, ok := tagMap[tag]
		if !ok {
			continue
		}

		// Build style properties map
		// Estimate capacity: typically 3-5 properties per style
		props := make(map[string]string, 8)
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
func buildTagMap(prog *language.Program) map[string]map[string]bool {
	// Estimate capacity: typically few tags per architecture
	estimatedTags := 16
	tagMap := make(map[string]map[string]bool, estimatedTags)
	const metaKeyTags = "tags"

	systems := extractSystems(prog)
	persons := extractPersons(prog)

	// Add default tags
	addTag(tagMap, "Element", systems, func(s *language.System) string { return s.ID })
	addTag(tagMap, "System", systems, func(s *language.System) string { return s.ID })
	addTag(tagMap, "Person", persons, func(p *language.Person) string { return p.ID })

	// Add tags from metadata
	for _, sys := range systems {
		for _, meta := range sys.Metadata {
			if meta.Key == metaKeyTags && len(meta.Array) > 0 {
				for _, tag := range meta.Array {
					addTagToElement(tagMap, tag, sys.ID)
				}
			}
		}

		// Helper to build qualified IDs efficiently
		buildQualifiedID := func(parts ...string) string {
			if len(parts) == 0 {
				return ""
			}
			if len(parts) == 1 {
				return parts[0]
			}
			totalLen := len(parts) - 1 // for dots
			for _, p := range parts {
				totalLen += len(p)
			}
			buf := make([]byte, 0, totalLen)
			buf = append(buf, parts[0]...)
			for i := 1; i < len(parts); i++ {
				buf = append(buf, '.')
				buf = append(buf, parts[i]...)
			}
			return string(buf)
		}

		for _, cont := range sys.Containers {
			contID := buildQualifiedID(sys.ID, cont.ID)
			for _, meta := range cont.Metadata {
				if meta.Key == metaKeyTags && len(meta.Array) > 0 {
					for _, tag := range meta.Array {
						addTagToElement(tagMap, tag, contID)
					}
				}
			}
		}

		for _, ds := range sys.DataStores {
			dsID := buildQualifiedID(sys.ID, ds.ID)
			for _, meta := range ds.Metadata {
				if meta.Key == metaKeyTags && len(meta.Array) > 0 {
					for _, tag := range meta.Array {
						addTagToElement(tagMap, tag, dsID)
					}
				}
			}
		}
	}

	return tagMap
}

func addTag[T any](tagMap map[string]map[string]bool, tag string, items []T, getID func(T) string) {
	for _, item := range items {
		addTagToElement(tagMap, tag, getID(item))
	}
}

func addTagToElement(tagMap map[string]map[string]bool, tag, elemID string) {
	if tagMap[tag] == nil {
		// Estimate capacity: typically 5-10 elements per tag
		tagMap[tag] = make(map[string]bool, 16)
	}
	tagMap[tag][elemID] = true
}

// FindViewByName finds a view by name in the views block.
// Note: This function works with the old ViewBlock structure.
// For LikeC4 views, use FindLikeC4ViewByName instead.
func FindViewByName(_ *language.Program, _ string) *language.View {
	// Check old ViewBlock format (if still supported)
	// For now, return nil as ViewBlock is not part of Program
	// This function may need to be updated to work with LikeC4ViewsBlock
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
