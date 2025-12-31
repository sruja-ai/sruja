package dot

import (
	"fmt"
	"strings"
)

// writeGraphHeader writes the DOT digraph header with global attributes.
// nodeCount is used for adaptive spacing - more nodes need more spacing.
//
//nolint:unused // Old implementation, replaced by constraint-based approach in dot_generator.go
func (e *Exporter) writeGraphHeader(sb *strings.Builder, nodeCount int) {
	// Adaptive spacing based on node count
	// Base: 120px nodesep, 130px ranksep
	// Scale up for denser diagrams
	nodeSep := e.Config.NodeSep
	rankSep := e.Config.RankSep

	if nodeCount > 3 {
		switch {
		case nodeCount <= 6:
			// For 4-6 nodes, add 20% spacing
			nodeSep = int(float64(nodeSep) * 1.2)
			rankSep = int(float64(rankSep) * 1.2)
		case nodeCount <= 10:
			// For 7-10 nodes, add 40% spacing
			nodeSep = int(float64(nodeSep) * 1.4)
			rankSep = int(float64(rankSep) * 1.4)
		default:
			// For 11+ nodes, add 60% spacing
			nodeSep = int(float64(nodeSep) * 1.6)
			rankSep = int(float64(rankSep) * 1.6)
		}
	}

	sb.WriteString("digraph G {\n")
	sb.WriteString("  graph [\n")
	fmt.Fprintf(sb, "    rankdir=\"%s\",\n", e.Config.RankDir)
	fmt.Fprintf(sb, "    nodesep=%.2f,\n", pxToInch(nodeSep))
	fmt.Fprintf(sb, "    ranksep=%.2f,\n", pxToInch(rankSep))
	sb.WriteString("    layout=\"dot\",\n")
	sb.WriteString("    compound=true,\n")
	sb.WriteString("    splines=spline,\n")         // Curved splines for natural look
	sb.WriteString("    TBbalance=min,\n")          // Better vertical balance
	sb.WriteString("    outputorder=nodesfirst,\n") // Draw nodes before edges
	sb.WriteString("    newrank=true,\n")           // Better cluster handling
	sb.WriteString("    pad=0.4,\n")
	sb.WriteString("    fontname=\"Arial\",\n")
	sb.WriteString("    fontsize=12,\n")
	sb.WriteString("    dpi=72\n")
	sb.WriteString("  ];\n\n")
}

// writeGlobalNodeAttributes writes default node attributes.
//
//nolint:unused // Old implementation, replaced by constraint-based approach in dot_generator.go
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
//
//nolint:unused // Old implementation, replaced by constraint-based approach in dot_generator.go
func (e *Exporter) writeGlobalEdgeAttributes(sb *strings.Builder) {
	sb.WriteString("  edge [\n")
	sb.WriteString("    fontname=\"Arial\",\n")
	sb.WriteString("    fontsize=11,\n")
	sb.WriteString("    penwidth=2,\n")
	sb.WriteString("    arrowsize=0.75\n")
	sb.WriteString("  ];\n\n")
}

// writeNode writes a single node definition.
//
//nolint:unused // Old implementation, replaced by constraint-based approach in dot_generator.go
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
	fmt.Fprintf(sb, "%s  height=%.2f", indent, pxToInch(height))

	// Add group attribute for sibling clustering
	if elem.ParentID != "" {
		fmt.Fprintf(sb, ",\n%s  group=\"%s\"", indent, escapeID(elem.ParentID))
	}

	sb.WriteString("\n")
	fmt.Fprintf(sb, "%s];\n", indent)
}

// writeCluster writes a subgraph cluster for hierarchical grouping.
//
//nolint:unused // Old implementation, replaced by constraint-based approach in dot_generator.go
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
//
//nolint:unused // Old implementation, replaced by constraint-based approach in dot_generator.go
func (e *Exporter) writeRankConstraints(sb *strings.Builder, elements []*Element) {
	// Group elements by kind
	byKind := make(map[string][]*Element)
	for _, elem := range elements {
		byKind[elem.Kind] = append(byKind[elem.Kind], elem)
	}

	sb.WriteString("  // Rank constraints for alignment\n")

	// L1 Context View: Persons at top, then systems
	persons := byKind["person"]
	systems := byKind["system"]

	// Put all persons on the same rank (top)
	if len(persons) > 0 {
		ids := make([]string, len(persons))
		for i, elem := range persons {
			ids[i] = fmt.Sprintf("\"%s\"", escapeID(elem.ID))
		}
		fmt.Fprintf(sb, "  { rank=min; %s }  // Persons at top\n", strings.Join(ids, "; "))
	}

	// Put all systems on the same rank (below persons)
	if len(systems) > 0 {
		ids := make([]string, len(systems))
		for i, elem := range systems {
			ids[i] = fmt.Sprintf("\"%s\"", escapeID(elem.ID))
		}
		fmt.Fprintf(sb, "  { rank=same; %s }  // Systems aligned\n", strings.Join(ids, "; "))
	}

	// Add invisible edges for vertical ordering (persons above systems)
	if len(persons) > 0 && len(systems) > 0 {
		// Connect first person to first system with invisible high-weight edge
		fmt.Fprintf(sb, "  \"%s\" -> \"%s\" [style=invis, weight=1000];  // Persons above Systems\n",
			escapeID(persons[0].ID), escapeID(systems[0].ID))
	}

	// L2/L3: Group container-level elements together
	containers := byKind["container"]
	datastores := byKind["datastore"]
	queues := byKind["queue"]

	// All containers on same rank
	if len(containers) > 0 {
		ids := make([]string, len(containers))
		for i, elem := range containers {
			ids[i] = fmt.Sprintf("\"%s\"", escapeID(elem.ID))
		}
		fmt.Fprintf(sb, "  { rank=same; %s }  // Containers aligned\n", strings.Join(ids, "; "))
	}

	// All datastores on same rank (typically below containers)
	if len(datastores) > 0 {
		ids := make([]string, len(datastores))
		for i, elem := range datastores {
			ids[i] = fmt.Sprintf("\"%s\"", escapeID(elem.ID))
		}
		fmt.Fprintf(sb, "  { rank=same; %s }  // Datastores aligned\n", strings.Join(ids, "; "))
	}

	// All queues on same rank (typically with datastores)
	if len(queues) > 0 {
		ids := make([]string, len(queues))
		for i, elem := range queues {
			ids[i] = fmt.Sprintf("\"%s\"", escapeID(elem.ID))
		}
		fmt.Fprintf(sb, "  { rank=same; %s }  // Queues aligned\n", strings.Join(ids, "; "))
	}

	// Components
	components := byKind["component"]
	if len(components) > 0 {
		ids := make([]string, len(components))
		for i, elem := range components {
			ids[i] = fmt.Sprintf("\"%s\"", escapeID(elem.ID))
		}
		fmt.Fprintf(sb, "  { rank=same; %s }  // Components aligned\n", strings.Join(ids, "; "))
	}

	sb.WriteString("\n")
}

// writeEdge writes a single edge definition.
//
//nolint:unused // Old implementation, replaced by constraint-based approach in dot_generator.go
func (e *Exporter) writeEdge(sb *strings.Builder, rel *Relation) {
	attrs := []string{}

	// Add label if present
	if rel.Label != "" {
		attrs = append(attrs, fmt.Sprintf("label=\"%s\"", escapeLabel(rel.Label)))
	}

	// Edge weights and layout constraints
	if e.Config.UseEdgeWeights {
		// Labeled edges are considered more important for layout
		if rel.Label != "" {
			attrs = append(attrs, "weight=10")
		} else {
			attrs = append(attrs, "weight=1")
		}
		// Minimum edge length to prevent nodes from being too close
		attrs = append(attrs, "minlen=1")
	}

	attrStr := ""
	if len(attrs) > 0 {
		attrStr = fmt.Sprintf(" [%s]", strings.Join(attrs, ", "))
	}

	fmt.Fprintf(sb, "  \"%s\" -> \"%s\"%s;\n", escapeID(rel.From), escapeID(rel.To), attrStr)
}

// groupByParent groups elements by their parent ID.
// Returns root elements (no parent) and a map of parent ID to children.
//
//nolint:unused // Old implementation, replaced by constraint-based approach in dot_generator.go
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
