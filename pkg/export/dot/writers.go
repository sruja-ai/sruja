package dot

import (
	"fmt"
	"strings"
)

// writeGraphHeader writes the DOT digraph header with global attributes.
func (e *Exporter) writeGraphHeader(sb *strings.Builder) {
	sb.WriteString("digraph G {\n")
	sb.WriteString("  graph [\n")
	fmt.Fprintf(sb, "    rankdir=\"%s\",\n", e.Config.RankDir)
	fmt.Fprintf(sb, "    nodesep=%.2f,\n", pxToInch(e.Config.NodeSep))
	fmt.Fprintf(sb, "    ranksep=%.2f,\n", pxToInch(e.Config.RankSep))
	sb.WriteString("    layout=\"dot\",\n")
	sb.WriteString("    compound=true,\n")
	sb.WriteString("    splines=ortho,\n")
	sb.WriteString("    pad=0.2,\n")
	sb.WriteString("    fontname=\"Arial\",\n")
	sb.WriteString("    fontsize=12,\n")
	sb.WriteString("    dpi=72\n")
	sb.WriteString("  ];\n\n")
}

// writeGlobalNodeAttributes writes default node attributes.
func (e *Exporter) writeGlobalNodeAttributes(sb *strings.Builder) {
	sb.WriteString("  node [\n")
	sb.WriteString("    shape=rect,\n")
	sb.WriteString("    fixedsize=true,\n")
	sb.WriteString("    style=\"filled\",\n")
	sb.WriteString("    penwidth=0,\n")
	sb.WriteString("    fontname=\"Arial\"\n")
	sb.WriteString("  ];\n\n")
}

// writeGlobalEdgeAttributes writes default edge attributes.
func (e *Exporter) writeGlobalEdgeAttributes(sb *strings.Builder) {
	sb.WriteString("  edge [\n")
	sb.WriteString("    fontname=\"Arial\",\n")
	sb.WriteString("    fontsize=11,\n")
	sb.WriteString("    penwidth=2,\n")
	sb.WriteString("    arrowsize=0.75\n")
	sb.WriteString("  ];\n\n")
}

// writeNode writes a single node definition.
func (e *Exporter) writeNode(sb *strings.Builder, elem *Element, indent string) {
	width := elem.Width
	if width == 0 {
		width = e.Config.DefaultNodeWidth
	}
	height := elem.Height
	if height == 0 {
		height = e.Config.DefaultNodeHeight
	}

	label := elem.Title
	if label == "" {
		label = elem.ID
	}

	fmt.Fprintf(sb, "%s\"%s\" [\n", indent, escapeID(elem.ID))
	fmt.Fprintf(sb, "%s  label=\"%s\",\n", indent, escapeLabel(label))
	fmt.Fprintf(sb, "%s  width=%.2f,\n", indent, pxToInch(width))
	fmt.Fprintf(sb, "%s  height=%.2f\n", indent, pxToInch(height))
	fmt.Fprintf(sb, "%s];\n", indent)
}

// writeCluster writes a subgraph cluster for hierarchical grouping.
func (e *Exporter) writeCluster(sb *strings.Builder, parentID string, children []*Element, allElements []*Element) {
	// Find the parent element for the label
	var parentTitle string
	for _, elem := range allElements {
		if elem.ID == parentID {
			parentTitle = elem.Title
			break
		}
	}
	if parentTitle == "" {
		parentTitle = parentID
	}

	fmt.Fprintf(sb, "  subgraph \"cluster_%s\" {\n", escapeID(parentID))
	fmt.Fprintf(sb, "    label=\"%s\";\n", escapeLabel(parentTitle))
	sb.WriteString("    style=\"filled\";\n")
	sb.WriteString("    color=\"transparent\";\n")
	sb.WriteString("    margin=40;\n\n")

	for _, child := range children {
		e.writeNode(sb, child, "    ")
	}

	sb.WriteString("  }\n\n")
}

// writeRankConstraints writes rank=same constraints for proper alignment.
func (e *Exporter) writeRankConstraints(sb *strings.Builder, elements []*Element) {
	// Group elements by kind
	byKind := make(map[string][]*Element)
	for _, elem := range elements {
		byKind[elem.Kind] = append(byKind[elem.Kind], elem)
	}

	sb.WriteString("  // Rank constraints for alignment\n")

	// Add rank=same for each kind with multiple elements
	for kind, kindElements := range byKind {
		if len(kindElements) > 1 {
			ids := make([]string, len(kindElements))
			for i, elem := range kindElements {
				ids[i] = fmt.Sprintf("\"%s\"", escapeID(elem.ID))
			}
			fmt.Fprintf(sb, "  { rank=same; %s }  // All %ss aligned\n", strings.Join(ids, "; "), kind)
		}
	}

	// Add invisible edges for vertical ordering (persons above systems)
	persons := byKind["person"]
	systems := byKind["system"]
	if len(persons) > 0 && len(systems) > 0 {
		fmt.Fprintf(sb, "  \"%s\" -> \"%s\" [style=invis, weight=100];  // Persons above Systems\n",
			escapeID(persons[0].ID), escapeID(systems[0].ID))
	}

	// Group container-level elements together
	containers := byKind["container"]
	datastores := byKind["datastore"]
	queues := byKind["queue"]

	allContainerLevel := append(append(containers, datastores...), queues...)
	if len(allContainerLevel) > 1 {
		ids := make([]string, len(allContainerLevel))
		for i, elem := range allContainerLevel {
			ids[i] = fmt.Sprintf("\"%s\"", escapeID(elem.ID))
		}
		fmt.Fprintf(sb, "  { rank=same; %s }  // Container-level elements aligned\n", strings.Join(ids, "; "))
	}

	sb.WriteString("\n")
}

// writeEdge writes a single edge definition.
func (e *Exporter) writeEdge(sb *strings.Builder, rel *Relation) {
	attrs := []string{}

	// Add label if present
	if rel.Label != "" {
		attrs = append(attrs, fmt.Sprintf("label=\"%s\"", escapeLabel(rel.Label)))
		// Labeled edges are considered more important for layout
		if e.Config.UseEdgeWeights {
			attrs = append(attrs, "weight=10")
		}
	}

	attrStr := ""
	if len(attrs) > 0 {
		attrStr = fmt.Sprintf(" [%s]", strings.Join(attrs, ", "))
	}

	fmt.Fprintf(sb, "  \"%s\" -> \"%s\"%s;\n", escapeID(rel.From), escapeID(rel.To), attrStr)
}

// groupByParent groups elements by their parent ID.
// Returns root elements (no parent) and a map of parent ID to children.
func (e *Exporter) groupByParent(elements []*Element) ([]*Element, map[string][]*Element) {
	var rootElements []*Element
	clusters := make(map[string][]*Element)

	for _, elem := range elements {
		if elem.ParentID == "" {
			rootElements = append(rootElements, elem)
		} else {
			clusters[elem.ParentID] = append(clusters[elem.ParentID], elem)
		}
	}

	return rootElements, clusters
}
