package mermaid

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/export/views"
)

// Generate creates a Mermaid diagram from elements and relations.
func (e *Exporter) Generate(elements []*views.Element, relations []*views.Relation) string {
	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	e.writeHeader(sb)
	e.writeStyles(sb)

	// Group elements by parent for subgraph generation
	rootElements, clusters := e.groupByParent(elements)

	// Map elements by ID for quick access
	elementMap := make(map[string]*views.Element)
	for _, elem := range elements {
		elementMap[elem.ID] = elem
	}

	// Write root-level nodes
	for _, elem := range rootElements {
		if _, isCluster := clusters[elem.ID]; isCluster {
			// This is a cluster (parent system or container)
			e.writeSubgraph(sb, elem, clusters, elementMap, Indent4)
		} else {
			// This is a leaf node
			e.writeAnyElement(sb, elem, Indent4)
		}
	}

	// Write Relations
	for _, rel := range relations {
		e.writeRelation(sb, rel)
	}

	return sb.String()
}

func (e *Exporter) writeSubgraph(sb *strings.Builder, parent *views.Element, clusters map[string][]*views.Element, elementMap map[string]*views.Element, indent string) {
	id := sanitizeID(parent.ID)
	label := escapeQuotes(parent.Title)
	if label == "" {
		label = parent.ID
	}

	fmt.Fprintf(sb, "%ssubgraph %s[\"%s\"]\n", indent, id, label)
	fmt.Fprintf(sb, "%sdirection TB\n", indent+Indent4)

	children := clusters[parent.ID]
	for _, child := range children {
		if _, isCluster := clusters[child.ID]; isCluster {
			e.writeSubgraph(sb, child, clusters, elementMap, indent+Indent4)
		} else {
			e.writeAnyElement(sb, child, indent+Indent4)
		}
	}

	fmt.Fprintf(sb, "%send\n", indent)
}

func (e *Exporter) writeAnyElement(sb *strings.Builder, elem *views.Element, indent string) {
	switch strings.ToLower(elem.Kind) {
	case "person":
		e.writePerson(sb, elem)
	case "system":
		// Leaf system (no containers)
		id := sanitizeID(elem.ID)
		label := escapeQuotes(formatLabel(elem.Title, elem.ID, elem.Description, elem.Technology))
		fmt.Fprintf(sb, "%s%s[\"%s\"]\n", indent, id, label)
		fmt.Fprintf(sb, "%sclass %s %s\n", indent, id, ClassSystem)
	case "container":
		e.writeContainer(sb, elem, indent)
	case "datastore":
		e.writeDataStore(sb, elem, indent)
	case "queue":
		e.writeQueue(sb, elem, indent)
	case "component":
		e.writeComponent(sb, elem, indent)
	default:
		// Fallback
		id := sanitizeID(elem.ID)
		label := escapeQuotes(formatLabel(elem.Title, elem.ID, elem.Description, elem.Technology))
		fmt.Fprintf(sb, "%s%s[\"%s\"]\n", indent, id, label)
	}
}

func (e *Exporter) groupByParent(elements []*views.Element) ([]*views.Element, map[string][]*views.Element) {
	var rootElements []*views.Element
	clusters := make(map[string][]*views.Element)

	// Map to check if an element exists in the list
	elementSet := make(map[string]bool)
	for _, elem := range elements {
		elementSet[elem.ID] = true
	}

	for _, elem := range elements {
		// A root element is either one with no parent, or one whose parent is not in our list
		if elem.ParentID == "" || !elementSet[elem.ParentID] {
			rootElements = append(rootElements, elem)
		} else {
			clusters[elem.ParentID] = append(clusters[elem.ParentID], elem)
		}
	}

	return rootElements, clusters
}
