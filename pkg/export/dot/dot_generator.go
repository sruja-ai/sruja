// Package dot provides constraint-based DOT generation.
//
// This file implements DOT generation from explicit constraints,
// following the FAANG-level constraint-based architecture.

package dot

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
)

// GenerateDOTFromConstraints generates DOT string from constraints.
func GenerateDOTFromConstraints(
	elements []*Element,
	_ []*Relation,
	constraints LayoutConstraints,
) string {
	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	// Write graph header
	writeGraphHeaderFromConstraints(sb, constraints, len(elements))

	// Write global node attributes
	writeGlobalNodeAttributesFromConstraints(sb)

	// Write global edge attributes
	writeGlobalEdgeAttributesFromConstraints(sb)

	// Group elements by parent for cluster generation
	rootElements, clusters := groupByParent(elements)

	// Flatten L3 views to prevent Graphviz layout crashes ("missing node" in cluster)
	// L3 views already focus on a single container, so internal clustering is unnecessary
	// and often harmful to the layout engine.
	if constraints.ViewLevel == 3 {
		for _, children := range clusters {
			rootElements = append(rootElements, children...)
		}
		// Clear clusters
		clusters = make(map[string][]*Element)
	}

	// Build parentMap for cross-cluster edge detection and depth calculation
	parentMap := make(map[string]string)
	for _, elem := range elements {
		parentMap[elem.ID] = elem.ParentID
	}

	// Write root-level nodes
	for _, elem := range rootElements {
		writeNodeFromConstraints(sb, elem, constraints, "  ")
	}

	// Write clusters (subgraphs) with depth-based styling
	for parentID, children := range clusters {
		depth := calculateClusterDepth(parentID, parentMap)
		writeClusterFromConstraints(sb, parentID, children, elements, constraints, depth)
	}

	// Write rank constraints
	writeRankConstraintsFromData(sb, constraints.Ranks)

	// Write edges from constraints
	writeEdgesFromConstraints(sb, constraints.Edges, parentMap)

	sb.WriteString("}\n")

	return sb.String()
}

// writeGraphHeaderFromConstraints writes graph header from constraints.
func writeGraphHeaderFromConstraints(sb *strings.Builder, constraints LayoutConstraints, _ int) {
	sb.WriteString("digraph G {\n")
	sb.WriteString("  graph [\n")
	fmt.Fprintf(sb, "    rankdir=\"%s\",\n", constraints.Global.RankDir)
	fmt.Fprintf(sb, "    nodesep=%.2f,\n", constraints.Global.NodeSep)
	fmt.Fprintf(sb, "    ranksep=%.2f,\n", constraints.Global.RankSep)
	sb.WriteString("    layout=\"dot\",\n")
	sb.WriteString("    compound=true,\n")
	fmt.Fprintf(sb, "    splines=%s,\n", constraints.Global.Splines)
	sb.WriteString("    TBbalance=min,\n")
	sb.WriteString("    outputorder=nodesfirst,\n")
	sb.WriteString("    newrank=true,\n")
	fmt.Fprintf(sb, "    pad=%.1f,\n", GraphPad)
	fmt.Fprintf(sb, "    overlap=%s,\n", constraints.Global.Overlap)
	if constraints.Global.Concentrate {
		sb.WriteString("    concentrate=true,\n")
	}
	if constraints.Global.Sep > 0 {
		fmt.Fprintf(sb, "    sep=%.2f,\n", constraints.Global.Sep)
	}
	fmt.Fprintf(sb, "    fontname=\"%s\",\n", FontName)
	fmt.Fprintf(sb, "    fontsize=%d,\n", FontSizeGlobal)
	sb.WriteString("    dpi=72\n")
	sb.WriteString("  ];\n\n")
}

// writeNodeFromConstraints writes a node from constraints.
func writeNodeFromConstraints(sb *strings.Builder, elem *Element, constraints LayoutConstraints, indent string) {
	// Find size constraint for this node
	var width, height float64
	var fixedX, fixedY float64
	for _, size := range constraints.Sizes {
		if size.NodeID == elem.ID {
			width = size.PreferredWidth
			height = size.PreferredHeight
			fixedX = size.FixedX
			fixedY = size.FixedY
			break
		}
	}

	// Fallback to element's measured size
	if width == 0 {
		width = float64(elem.Width)
	}
	if height == 0 {
		height = float64(elem.Height)
	}

	fmt.Fprintf(sb, "%s\"%s\" [\n", indent, escapeID(elem.ID))

	// Use HTML-like label for rich formatting and dynamic sizing
	htmlLabel := buildNodeHTML(elem.Title, elem.Kind, elem.Technology, elem.Description)
	fmt.Fprintf(sb, "%s  label=<%s>,\n", indent, htmlLabel)

	// Remove fixedsize and explicit dimensions to allow content-driven sizing
	// Only apply fixedsize if manually positioned
	if fixedX > 0 || fixedY > 0 {
		fmt.Fprintf(sb, "%s  pos=\"%.0f,%.0f\",\n", indent, fixedX, fixedY)
		fmt.Fprintf(sb, "%s  fixedsize=true", indent)
		// Rest of fixed size logic would go here if we were supporting resizing in manual mode,
		// but for now we let it be dynamic or use min dimensions if needed.
	} else {
		// For auto-layout, allow dynamic sizing but respect minimums if set in sizes
		// Graphviz will expand node to fit label, but 'width'/'height' act as minimums.
		if width > 0 {
			fmt.Fprintf(sb, "%s  width=%.2f,\n", indent, pxToInchFloat(width))
		}
		if height > 0 {
			fmt.Fprintf(sb, "%s  height=%.2f,\n", indent, pxToInchFloat(height))
		}
	}

	// reset margin to 0 because HTML table handles padding
	fmt.Fprintf(sb, "%s  margin=0", indent)

	// Add group attribute for sibling clustering
	if elem.ParentID != "" {
		fmt.Fprintf(sb, ",\n%s  group=\"%s\"", indent, escapeID(elem.ParentID))
	}

	sb.WriteString("\n")
	fmt.Fprintf(sb, "%s];\n", indent)
}

// writeRankConstraintsFromData writes rank constraints from constraint data.
func writeRankConstraintsFromData(sb *strings.Builder, ranks []RankConstraint) {
	if len(ranks) == 0 {
		return
	}

	sb.WriteString("  // Rank constraints for alignment\n")

	for _, rank := range ranks {
		if len(rank.NodeIDs) == 0 {
			continue
		}

		ids := make([]string, len(rank.NodeIDs))
		for i, id := range rank.NodeIDs {
			ids[i] = fmt.Sprintf("\"%s\"", escapeID(id))
		}

		comment := ""
		switch rank.Type {
		case "min":
			comment = " // Top rank"
		case "max":
			comment = " // Bottom rank"
		case "same":
			comment = " // Same rank alignment"
		}

		fmt.Fprintf(sb, "  { rank=%s; %s }%s\n", rank.Type, strings.Join(ids, "; "), comment)
	}

	// Add invisible edges for rank ordering (persons above systems, etc.)
	// This ensures proper vertical ordering with strong constraints
	// Use high-weight invisible edges to enforce rank separation
	if len(ranks) >= 2 {
		// Connect first node of first rank to first node of second rank
		firstRank := ranks[0]
		secondRank := ranks[1]
		if len(firstRank.NodeIDs) > 0 && len(secondRank.NodeIDs) > 0 {
			// Use very high weight to ensure rank ordering is respected
			fmt.Fprintf(sb, "  \"%s\" -> \"%s\" [style=invis, weight=1000, minlen=2];  // Rank ordering\n",
				escapeID(firstRank.NodeIDs[0]), escapeID(secondRank.NodeIDs[0]))
		}

		// For high-quality layout, also connect all nodes in first rank to all in second
		// This creates a stronger constraint for proper vertical ordering
		if len(firstRank.NodeIDs) > 1 && len(secondRank.NodeIDs) > 1 {
			// Connect last node of first rank to last node of second rank
			// This helps maintain alignment across the entire rank
			lastFirst := firstRank.NodeIDs[len(firstRank.NodeIDs)-1]
			lastSecond := secondRank.NodeIDs[len(secondRank.NodeIDs)-1]
			fmt.Fprintf(sb, "  \"%s\" -> \"%s\" [style=invis, weight=1000, minlen=2];  // Rank alignment\n",
				escapeID(lastFirst), escapeID(lastSecond))
		}
	}

	sb.WriteString("\n")
}

// writeEdgesFromConstraints writes edges from constraint data.
// Enhanced with edge bundling for complex diagrams.
func writeEdgesFromConstraints(sb *strings.Builder, edges []EdgeConstraint, parentMap map[string]string) {
	// Group edges by source-target pair for bundling detection
	edgeGroups := groupEdgesForBundling(edges)

	for _, edge := range edges {
		attrs := []string{}

		// Check if this edge should be bundled
		bundleKey := edge.From + "->" + edge.To
		group := edgeGroups[bundleKey]
		isBundled := len(group) >= EdgeBundlingThreshold

		// Add label if present
		if edge.Label.Text != "" {
			attrs = append(attrs, fmt.Sprintf("label=\"%s\"", escapeLabel(edge.Label.Text)))

			// Add label positioning attributes for FAANG-quality appearance
			attrs = append(attrs, fmt.Sprintf("fontsize=%d", FontSizeEdge))
			attrs = append(attrs, fmt.Sprintf("fontcolor=\"%s\"", ColorSlate700))
			// attrs = append(attrs, "decorate=true")         // Connect label to edge visually - DISABLED, creates confusion
			attrs = append(attrs, "labelfloat=false") // Force space for label to prevent overlap with edge

			// Add label positioning attributes
			// NOTE: labeldistance is primarily for head/taillabel, but can affect main label positioning
			if edge.Label.Distance != 0 {
				attrs = append(attrs, fmt.Sprintf("labeldistance=%.2f", edge.Label.Distance))
			}
			if edge.Label.Angle != 0 {
				attrs = append(attrs, fmt.Sprintf("labelangle=%.2f", edge.Label.Angle))
			}
			// labelpos is usually not needed for center labels, but keeping if set
			if edge.Label.Position != 0.5 {
				attrs = append(attrs, fmt.Sprintf("labelpos=%.2f", edge.Label.Position))
			}
		}

		// Add weight
		if edge.Weight > 0 {
			attrs = append(attrs, fmt.Sprintf("weight=%d", edge.Weight))
		}

		// Add minimum length
		if edge.MinLen > 0 {
			attrs = append(attrs, fmt.Sprintf("minlen=%d", edge.MinLen))
		}

		// Add ports if specified
		fromPort := ""
		toPort := ""
		if edge.Ports.Tail != "" {
			fromPort = ":" + edge.Ports.Tail
		}
		if edge.Ports.Head != "" {
			toPort = ":" + edge.Ports.Head
		}

		// Add lhead/ltail for cross-cluster edges
		// These attributes clip edges to cluster boundaries when compound=true
		fromParent := parentMap[edge.From]
		toParent := parentMap[edge.To]

		// Only add lhead/ltail when edge crosses cluster boundaries
		// (i.e., when parents are different and both are non-empty)
		if fromParent != "" && toParent != "" && fromParent != toParent {
			attrs = append(attrs, fmt.Sprintf("ltail=\"cluster_%s\"", escapeID(fromParent)))
			attrs = append(attrs, fmt.Sprintf("lhead=\"cluster_%s\"", escapeID(toParent)))
		}

		// Add bundling attributes for parallel edges
		if isBundled {
			// Calculate position in bundle (for staggering labels)
			bundleIndex := getBundleIndex(edge, group)
			bundleSize := len(group)

			if bundleIndex > 0 {
				// Stagger parallel edges slightly to reduce overlap
				attrs = append(attrs, fmt.Sprintf("minlen=%d", edge.MinLen+bundleIndex))
			}

			// Use samehead for bundled edges (same starting arrow)
			if bundleSize > 1 {
				attrs = append(attrs, fmt.Sprintf("samehead=\"bundle_%s\"", bundleKey))
			}

			// Use sametail for bundled edges (same ending arrow)
			if bundleSize > 1 {
				attrs = append(attrs, fmt.Sprintf("sametail=\"bundle_%s\"", bundleKey))
			}

			// Lighter color for bundled edges to reduce visual clutter
			attrs = append(attrs, "color=\"#8a9bab\"")
		}

		// Add constraint attribute
		if !edge.AffectsLayout {
			attrs = append(attrs, "constraint=false")
		}

		attrStr := ""
		if len(attrs) > 0 {
			attrStr = fmt.Sprintf(" [%s]", strings.Join(attrs, ", "))
		}

		fmt.Fprintf(sb, "  \"%s\"%s -> \"%s\"%s%s;\n",
			escapeID(edge.From), fromPort,
			escapeID(edge.To), toPort,
			attrStr)
	}
}

// groupEdgesForBundling groups edges by their source-target pair.
func groupEdgesForBundling(edges []EdgeConstraint) map[string][]EdgeConstraint {
	groups := make(map[string][]EdgeConstraint)
	for _, edge := range edges {
		key := edge.From + "->" + edge.To
		groups[key] = append(groups[key], edge)
	}
	return groups
}

// getBundleIndex returns the index of an edge within its bundle.
func getBundleIndex(edge EdgeConstraint, group []EdgeConstraint) int {
	for i, e := range group {
		if e.From == edge.From && e.To == edge.To && e.Label.Text == edge.Label.Text {
			return i
		}
	}
	return 0
}

// groupByParent groups elements by their parent ID (helper function).
func groupByParent(elements []*Element) ([]*Element, map[string][]*Element) {
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

// writeGlobalNodeAttributesFromConstraints writes default node attributes.
func writeGlobalNodeAttributesFromConstraints(sb *strings.Builder) {
	sb.WriteString("  node [\n")
	sb.WriteString("    shape=rect,\n")
	// fixedsize=true is removed to allow dynamic sizing based on HTML labels
	sb.WriteString("    style=\"filled\",\n")
	fmt.Fprintf(sb, "    penwidth=%d,\n", PenWidthNode)
	fmt.Fprintf(sb, "    fontname=\"%s\"\n", FontName)
	sb.WriteString("  ];\n\n")
}

// writeGlobalEdgeAttributesFromConstraints writes default edge attributes.
func writeGlobalEdgeAttributesFromConstraints(sb *strings.Builder) {
	sb.WriteString("  edge [\n")
	fmt.Fprintf(sb, "    fontname=\"%s\",\n", FontName)
	fmt.Fprintf(sb, "    fontsize=%d,\n", FontSizeEdge)
	fmt.Fprintf(sb, "    penwidth=%d,\n", PenWidthEdge)
	fmt.Fprintf(sb, "    arrowsize=%.2f,\n", ArrowSize)
	fmt.Fprintf(sb, "    color=\"%s\",\n", ColorSlate500)
	fmt.Fprintf(sb, "    fontcolor=\"%s\"\n", ColorSlate700)
	sb.WriteString("  ];\n\n")
}

// calculateClusterDepth calculates the nesting depth of a cluster.
// Depth 0 = root level (no parent), depth 1 = nested one level, etc.
func calculateClusterDepth(parentID string, parentMap map[string]string) int {
	depth := 0
	current := parentID
	for current != "" {
		parent, exists := parentMap[current]
		if !exists || parent == "" {
			break
		}
		depth++
		current = parent
	}
	return depth
}

// writeClusterFromConstraints writes a subgraph cluster for hierarchical grouping.
// Enhanced with depth-based visual hierarchy and improved containment.
func writeClusterFromConstraints(sb *strings.Builder, parentID string, children []*Element, allElements []*Element, constraints LayoutConstraints, depth int) {
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

	// Depth-based styling for visual hierarchy
	// Different colors and styles based on nesting depth
	var fillColor, strokeColor string
	var penwidth int

	switch depth {
	case 0:
		// Top-level system containers
		fillColor = "#e8f4f8"
		strokeColor = "#b0c4de"
		penwidth = 2
	case 1:
		// First level nested (services within systems)
		fillColor = "#f0f8ff"
		strokeColor = "#add8e6"
		penwidth = 1
	case 2:
		// Second level nested (components within containers)
		fillColor = "#fafaff"
		strokeColor = "#d3d3d3"
		penwidth = 1
	default:
		// Deeply nested
		fillColor = "#fcfcfc"
		strokeColor = "#e0e0e0"
		penwidth = 1
	}

	sb.WriteString("    style=\"filled,rounded\";\n")
	fmt.Fprintf(sb, "    color=\"%s\";\n", strokeColor)
	fmt.Fprintf(sb, "    bgcolor=\"%s\";\n", fillColor)
	fmt.Fprintf(sb, "    penwidth=%d;\n", penwidth)

	// Increase margin for deeper nesting to improve visual separation
	// For complex diagrams, add extra margin to prevent child nodes from touching cluster boundaries
	margin := MarginCluster + depth*3
	// Add extra margin for clusters with many children
	if len(children) >= 5 {
		margin += 15 // Extra margin for clusters with many children
	}
	// Even more margin for very large clusters
	if len(children) >= 10 {
		margin += 20
	}
	fmt.Fprintf(sb, "    margin=\"%d,%d\";\n", margin, margin/2)

	sb.WriteString("    labelloc=\"t\";\n")  // Top
	sb.WriteString("    labeljust=\"l\";\n") // Left
	fmt.Fprintf(sb, "    fontsize=%d;\n", FontSizeCluster)
	fmt.Fprintf(sb, "    fontcolor=\"%s\";\n", ColorSlate800)

	// Add compound node type hint for styling
	fmt.Fprintf(sb, "    compoundnode=true;\n")

	sb.WriteString("\n")

	for _, child := range children {
		writeNodeFromConstraints(sb, child, constraints, "    ")
	}

	sb.WriteString("  }\n\n")
}
