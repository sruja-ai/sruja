// Package markdown provides markdown generation.
//
//nolint:gocritic // Use WriteString for consistency
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

func writeMermaidStyles(sb *strings.Builder) {
	sb.WriteString("    classDef personStyle fill:#ffcccc,stroke:#333,stroke-width:2px,color:#000\n")
	sb.WriteString("    classDef systemStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000\n")
	sb.WriteString("    classDef containerStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000\n")
	sb.WriteString("    classDef databaseStyle fill:#ccffcc,stroke:#333,stroke-width:2px,color:#000\n")
	sb.WriteString("    classDef queueStyle fill:#ffe5cc,stroke:#333,stroke-width:2px,color:#000\n")
	sb.WriteString("    classDef externalStyle fill:#eeeeee,stroke:#666,stroke-width:2px,color:#000,stroke-dasharray: 3 3\n")
	sb.WriteString("    classDef componentStyle fill:#e6f7ff,stroke:#333,stroke-width:2px,color:#000\n")
	sb.WriteString("\n")
}

func renderContainerNodes(sb *strings.Builder, sys *language.System) {
	for _, cont := range sys.Containers {
		nodeID := sanitizeNodeID(cont.ID)
		label := cont.Label
		if label == "" {
			label = cont.ID
		}
		label = strings.ReplaceAll(label, "\"", "\\\"")
		sb.WriteString(fmt.Sprintf("        %s[\"%s\"]\n", nodeID, label))
		sb.WriteString(fmt.Sprintf("        class %s containerStyle\n", nodeID))
	}

	for _, ds := range sys.DataStores {
		nodeID := sanitizeNodeID(ds.ID)
		label := ds.Label
		if label == "" {
			label = ds.ID
		}
		label = strings.ReplaceAll(label, "\"", "\\\"")
		sb.WriteString(fmt.Sprintf("        %s[(\"%s\")]\n", nodeID, label))
		sb.WriteString(fmt.Sprintf("        class %s databaseStyle\n", nodeID))
	}

	for _, q := range sys.Queues {
		nodeID := sanitizeNodeID(q.ID)
		label := q.Label
		if label == "" {
			label = q.ID
		}
		label = strings.ReplaceAll(label, "\"", "\\\"")
		sb.WriteString(fmt.Sprintf("        %s[\"%s\"]\n", nodeID, label))
		sb.WriteString(fmt.Sprintf("        class %s queueStyle\n", nodeID))
	}
}

func resolveSystemElement(sys *language.System, name string, compToContainer map[string]string) (string, bool) {
	parts := strings.Split(name, ".")
	candidate := parts[len(parts)-1]

	// Check direct system children
	for _, c := range sys.Containers {
		if c.ID == candidate {
			return candidate, true
		}
	}
	for _, ds := range sys.DataStores {
		if ds.ID == candidate {
			return candidate, true
		}
	}
	for _, q := range sys.Queues {
		if q.ID == candidate {
			return candidate, true
		}
	}

	// Check components map
	if parent, ok := compToContainer[candidate]; ok {
		return parent, true
	}
	return "", false
}

func relationLabel(rel *language.Relation) string {
	label := ""
	if rel.Verb != nil {
		label = *rel.Verb
	} else if rel.Label != nil {
		label = *rel.Label
	}
	return strings.ReplaceAll(label, "\"", "\\\"")
}

func renderEdge(sb *strings.Builder, from, to, label string) {
	fromID := sanitizeNodeID(from)
	toID := sanitizeNodeID(to)
	if label != "" {
		sb.WriteString(fmt.Sprintf("    %s -->|%s| %s\n", fromID, label, toID))
	} else {
		sb.WriteString(fmt.Sprintf("    %s --> %s\n", fromID, toID))
	}
}

func renderComponentNodes(sb *strings.Builder, container *language.Container) map[string]bool {
	compIDs := make(map[string]bool)
	for _, comp := range container.Components {
		nodeID := sanitizeNodeID(comp.ID)
		label := comp.Label
		if label == "" {
			label = comp.Label
			if label == "" {
				label = comp.ID
			}
		}
		label = strings.ReplaceAll(label, "\"", "\\\"")
		sb.WriteString(fmt.Sprintf("    %s[\"%s\"]\n", nodeID, label))
		sb.WriteString(fmt.Sprintf("    class %s componentStyle\n", nodeID))
		compIDs[comp.ID] = true
	}
	return compIDs
}

// Resolver resolves an element name (potentially qualified) to an ID valid in the current view
type Resolver func(name string) (string, bool)

//nolint:gocyclo // Complex logic required for resolving external relations
func renderExternalRelations(
	sb *strings.Builder,
	arch *language.Architecture,
	resolveInternal Resolver,
	containerToSystem map[string]string,
) {
	externalNodes := make(map[string]bool)

	addExternal := func(top string, label string, isPerson bool) string {
		id := sanitizeNodeID(top)
		if externalNodes[top] {
			return id
		}
		safe := strings.ReplaceAll(label, "\"", "\\\"")
		if isPerson {
			sb.WriteString(fmt.Sprintf("    %s[\"%s\"]\n", id, safe))
			sb.WriteString(fmt.Sprintf("    class %s personStyle\n", id))
		} else {
			sb.WriteString(fmt.Sprintf("    %s[\"%s\"]\n", id, safe))
			sb.WriteString(fmt.Sprintf("    class %s externalStyle\n", id))
		}
		externalNodes[top] = true
		return id
	}

	for _, rel := range arch.Relations {
		fromInternal, okFrom := resolveInternal(rel.From.String())
		toInternal, okTo := resolveInternal(rel.To.String())
		lbl := relationLabel(rel)

		// Internal -> External
		if okFrom && !okTo {
			top, topOk := resolveOverviewNode(arch, containerToSystem, rel.To.String())
			if !topOk {
				continue
			}

			extLabel := top
			isPerson := isPersonOrSystem(arch, top) && !isSystem(arch, top)
			if isPerson {
				for _, p := range arch.Persons {
					if p.ID == top {
						extLabel = p.Label
						break
					}
				}
			} else {
				for _, s := range arch.Systems {
					if s.ID == top {
						extLabel = s.Label
						break
					}
				}
			}

			extID := addExternal(top, extLabel, isPerson)
			fromID := sanitizeNodeID(fromInternal)
			if lbl != "" {
				sb.WriteString(fmt.Sprintf("    %s -->|%s| %s\n", fromID, lbl, extID))
			} else {
				sb.WriteString(fmt.Sprintf("    %s --> %s\n", fromID, extID))
			}
		}

		// External -> Internal
		if okTo && !okFrom {
			top, topOk := resolveOverviewNode(arch, containerToSystem, rel.From.String())
			if !topOk {
				continue
			}

			extLabel := top
			isPerson := isPersonOrSystem(arch, top) && !isSystem(arch, top)
			if isPerson {
				for _, p := range arch.Persons {
					if p.ID == top {
						extLabel = p.Label
						break
					}
				}
			} else {
				for _, s := range arch.Systems {
					if s.ID == top {
						extLabel = s.Label
						break
					}
				}
			}

			extID := addExternal(top, extLabel, isPerson)
			toID := sanitizeNodeID(toInternal)
			if lbl != "" {
				sb.WriteString(fmt.Sprintf("    %s -->|%s| %s\n", extID, lbl, toID))
			} else {
				sb.WriteString(fmt.Sprintf("    %s --> %s\n", extID, toID))
			}
		}
	}
}

// Duplicate helper for circular dependency avoidance if needed,
// but since we are in same package, we can rely on mermaid.go's resolveOverviewNode and isPersonOrSystem
// Wait, resolveOverviewNode is in mermaid.go which is same package.
