// Package dot provides constraint-based layout system for Graphviz DOT generation.
//
// This file implements the FAANG-level constraint-based architecture where
// constraints are explicit data structures, not implicit in string generation.

package dot

// RankConstraint defines rank alignment constraints for nodes.
type RankConstraint struct {
	// Type is "min", "max", or "same"
	Type string
	// NodeIDs are the node IDs that should be on this rank
	NodeIDs []string
}

// SizeConstraint defines size constraints for a node.
type SizeConstraint struct {
	NodeID string
	// Min/Max bounds
	MinWidth, MaxWidth, MinHeight, MaxHeight float64
	// Preferred size (used if within bounds)
	PreferredWidth, PreferredHeight float64
	// Fixed position (for manual layout positions)
	FixedX, FixedY float64
}

// EdgePorts defines edge attachment points.
type EdgePorts struct {
	// Tail port: "n", "s", "e", "w", "ne", "nw", "se", "sw", or ""
	Tail string
	// Head port: same as Tail
	Head string
}

// EdgeLabel defines edge label positioning.
type EdgeLabel struct {
	Text     string
	Distance float64 // Distance from edge (in inches)
	Angle    float64 // Label angle (0=parallel, 90=perpendicular)
	Position float64 // Position along edge (0.0=head, 1.0=tail, 0.5=middle)
}

// EdgeConstraint defines constraints for an edge.
type EdgeConstraint struct {
	From, To string
	Weight   int // Edge weight (higher = more important for layout)
	MinLen   int // Minimum edge length
	Ports    EdgePorts
	Label    EdgeLabel
	// Constraint affects layout (false = edge doesn't affect node positioning)
	AffectsLayout bool
}

// GlobalConstraints defines global graph layout constraints.
type GlobalConstraints struct {
	NodeSep float64 // Horizontal spacing between nodes (in inches)
	RankSep float64 // Vertical spacing between ranks (in inches)
	Splines string  // "spline", "ortho", "polyline"
	RankDir string  // "TB" (top-bottom), "LR" (left-right)
	// Additional Graphviz attributes
	Overlap     string  // "false", "scale", "prism", etc.
	Concentrate bool    // Bundle parallel edges
	Sep         float64 // Minimum separation (in inches)
}

// LayoutConstraints contains all constraints for a layout.
type LayoutConstraints struct {
	Ranks     []RankConstraint
	Sizes     []SizeConstraint
	Edges     []EdgeConstraint
	Global    GlobalConstraints
	ViewLevel int // C4 view level (1=Context, 2=Container, 3=Component)
}

// BuildConstraints builds layout constraints from elements and relations.
func BuildConstraints(elements []*Element, relations []*Relation, viewLevel int, config Config) LayoutConstraints {

	constraints := LayoutConstraints{
		Global: GlobalConstraints{
			NodeSep:     pxToInchFloat(float64(config.NodeSep)),
			RankSep:     pxToInchFloat(float64(config.RankSep)),
			Splines:     "spline",
			RankDir:     config.RankDir,
			Overlap:     "false",
			Concentrate: len(relations) > 8, // Bundle parallel edges slightly later
			Sep:         0.1,                // Reduced from initialSep logic to standard small separation
		},
		ViewLevel: viewLevel,
	}

	// Adaptive spacing based on node count (logarithmic for FAANG-quality)
	// Increased base scaling factor to prevent overlaps in denser diagrams
	// Use more uniform scaling to improve spacing consistency
	if len(elements) > 2 {
		// More uniform logarithmic scaling to prevent overlaps while maintaining consistency
		// Formula: 1.0 + 0.25 * log10(nodeCount) for better adaptation
		scaleFactor := DynamicScalingBase + DynamicScalingFactor*float64(len(elements))/DynamicScalingDivisor
		if scaleFactor > DynamicScalingCap {
			scaleFactor = DynamicScalingCap // Slightly higher cap for dense diagrams
		}
		// Apply uniform scaling to both dimensions to maintain consistency
		constraints.Global.NodeSep *= scaleFactor
		constraints.Global.RankSep *= scaleFactor * 1.1 // Slightly more vertical spacing for better rank separation
	}

	// Additional aggressive spacing boost for very complex diagrams (20+ nodes)
	// This helps prevent overlaps and improves readability in large diagrams
	// Use more uniform multipliers to improve spacing consistency
	if len(elements) >= ComplexGraphThreshold {
		// More uniform spacing boost for very complex diagrams
		// Slightly reduced multipliers to maintain better consistency
		constraints.Global.NodeSep *= 1.25 // Reduced from 1.30 for better consistency
		constraints.Global.RankSep *= 1.30 // Reduced from 1.35, more uniform ratio
	}

	// Level-specific spacing improvements
	// L1 (Context View): Persons and systems - different node sizes cause overlaps
	// Use more uniform scaling to improve spacing consistency
	if viewLevel == 1 || viewLevel == 0 {
		constraints.Global.NodeSep *= L1NodeSepScale // Extra 15% spacing for L1
		constraints.Global.RankSep *= L1RankSepScale // Extra 20% vertical spacing for L1
		// Additional boost for L1 diagrams with many nodes to prevent overlaps
		// More uniform multipliers for better consistency
		if len(elements) >= 8 {
			constraints.Global.NodeSep *= 1.15 // Reduced from 1.20 for better consistency
			constraints.Global.RankSep *= 1.20 // Reduced from 1.25, more uniform ratio
		}
	}

	// L2 (Container View): Containers within systems
	// L2 benefits more from edge routing than spacing
	// Use polyline splines and edge routing optimizations instead of excessive spacing
	if viewLevel == 2 {
		// L2 needs polyline for better edge routing (reduces crossings)
		if len(elements) >= 6 {
			constraints.Global.NodeSep *= 1.10 // Reduced from 1.20 - less spacing, better routing
			constraints.Global.RankSep *= 1.12 // Reduced from 1.25
		}
		// Very complex L2 diagrams - focus on routing, not spacing
		if len(elements) >= 12 {
			constraints.Global.NodeSep *= 1.12 // Additional boost (reduced from 1.20)
			constraints.Global.RankSep *= 1.15 // Additional vertical spacing (reduced from 1.25)
		}
		// Use polyline for complex L2 to reduce edge crossings
		if len(elements) >= 8 {
			constraints.Global.Splines = "polyline" // Force polyline for better L2 routing
		}
	}

	// L3 (Component View): Components within containers
	// Round 1 strategy: aggressive spacing for better crossing reduction
	// L3 (Component View): Components within containers
	if viewLevel == 3 {
		// Standard spacing for L3
		if len(elements) >= 5 {
			constraints.Global.NodeSep *= L3NodeSepScale
			constraints.Global.RankSep *= L3RankSepScale
		}
		// Complex L3 - moderate spacing increase
		if len(elements) >= 15 {
			constraints.Global.NodeSep *= 1.10
			constraints.Global.RankSep *= 1.15
		}
	}

	// Spline configuration based on view level and complexity
	// L2 benefits from polyline for better edge routing
	// L3 uses spline with higher minlen for better aesthetics
	// Spline configuration
	// Standardize on spline (curves) for better aesthetics.
	// Only use polyline (angular) for L2/L3 if it's extremely dense to avoid crossing through nodes.
	constraints.Global.Splines = "spline"

	if viewLevel == 2 {
		nodeCountForDensity := float64(len(elements))
		if nodeCountForDensity < 1 {
			nodeCountForDensity = 1
		}
		edgeDensity := float64(len(relations)) / nodeCountForDensity
		// Only use polyline for extremely dense graphs where curves might be confusing
		if edgeDensity > 2.0 && len(elements) >= 20 {
			constraints.Global.Splines = "polyline"
		}
	}

	// Build rank constraints
	constraints.Ranks = buildRankConstraints(elements, viewLevel)

	// Build size constraints (with hub detection)
	constraints.Sizes = buildSizeConstraints(elements, relations, config)

	// Build parent map for edge weighting
	parentMap := make(map[string]string)
	for _, elem := range elements {
		parentMap[elem.ID] = elem.ParentID
	}

	// Build edge constraints with crossing reduction hints
	constraints.Edges = buildEdgeConstraints(relations, config, len(elements), parentMap, viewLevel)

	// Add invisible edges between sibling clusters to keep them together
	constraints.Edges = addSiblingClusterConstraints(constraints.Edges, elements, parentMap)

	return constraints
}

// buildRankConstraints builds rank constraints based on view level.
// Strict alignment of same-level nodes for professional appearance.
func buildRankConstraints(elements []*Element, viewLevel int) []RankConstraint {
	var ranks []RankConstraint

	// Group elements by kind
	byKind := make(map[string][]*Element)
	for _, elem := range elements {
		byKind[elem.Kind] = append(byKind[elem.Kind], elem)
	}

	// L1 (Context View): Persons/actors at top, systems below
	// Ensures all persons align perfectly, all systems align perfectly
	// Improved alignment for better spacing consistency
	if viewLevel == 1 || viewLevel == 0 {
		// Collect all person-like elements (person, actor, external)
		var persons []*Element
		persons = append(persons, byKind["person"]...)
		persons = append(persons, byKind["actor"]...)
		persons = append(persons, byKind["external"]...)

		if len(persons) > 0 {
			nodeIDs := make([]string, len(persons))
			for i, p := range persons {
				nodeIDs[i] = p.ID
			}
			// Use rank=min to ensure persons are at the top
			ranks = append(ranks, RankConstraint{
				Type:    "min",
				NodeIDs: nodeIDs,
			})
		}

		// Removed: rank=same for systems.
		// Allowing Graphviz to determine vertical hierarchy between systems (e.g. System -> External System)
		// yields better results for Context diagrams. Forcing all systems to same rank can create
		// worse layouts when systems have relationships that suggest different ranks.
	}

	// L2 (Container View): Containers, datastores, queues
	if viewLevel == 2 {
		// Do NOT force all containers to rank=same.
		// Complex systems (e.g. AI agents, pipelines) need vertical depth.
		// Let Graphviz determine rank based on edges (Top-Down flow).

		// Align datastores and queues at the bottom for cleaner layout
		// This is a common C4 pattern
		var infrastructure []string

		for _, d := range byKind["datastore"] {
			infrastructure = append(infrastructure, d.ID)
		}
		for _, q := range byKind["queue"] {
			infrastructure = append(infrastructure, q.ID)
		}

		if len(infrastructure) > 0 {
			ranks = append(ranks, RankConstraint{
				Type:    "same", // Align them together
				NodeIDs: infrastructure,
			})
			// Note: We don't use "max" because sometimes you want them side-by-side with lowest component
		}

		// For complex L2 diagrams (12+ containers), add additional rank grouping
		// to reduce edge crossings by keeping closely related containers on the same rank
		if len(elements) >= 12 && len(elements) <= 25 {
			// For moderately complex L2, group containers by their immediate relationships
			// This is done via edge weights which influence rank assignment
		}
	}

	// L3 (Component View): Components
	if viewLevel == 3 {
		components := byKind["component"]
		if len(components) > 0 {
			// For simple L3 diagrams (under 8 components), align all on same rank
			// For complex L3, let Graphviz determine hierarchy based on edges
			if len(components) <= 8 {
				nodeIDs := make([]string, len(components))
				for i, c := range components {
					nodeIDs[i] = c.ID
				}
				ranks = append(ranks, RankConstraint{
					Type:    "same",
					NodeIDs: nodeIDs,
				})
			}
			// For complex L3, only add rank constraints for components that are
			// strongly connected (bidirectional edges suggest same-level communication)
		}
	}

	return ranks
}

// buildSizeConstraints builds size constraints from elements.
// FAANG pattern: Larger size for "hub" nodes with many connections.
// Also handles manual position overrides from layout blocks.
func buildSizeConstraints(elements []*Element, relations []*Relation, config Config) []SizeConstraint {
	sizes := make([]SizeConstraint, 0, len(elements))

	// Calculate node degrees for hub detection
	degree := make(map[string]int)
	for _, rel := range relations {
		degree[rel.From]++
		degree[rel.To]++
	}

	for _, elem := range elements {
		// Start with element's default/parsed size
		width := float64(elem.Width)
		height := float64(elem.Height)

		// Check for frontend-provided size override (from text measurement)
		if size, ok := config.NodeSizes[elem.ID]; ok {
			width = size.Width
			height = size.Height
		}

		// Check for manual position override
		fixedX, fixedY := float64(0), float64(0)
		if pos, ok := config.ElementPositions[elem.ID]; ok {
			fixedX = pos.X
			fixedY = pos.Y
		}

		// Set min/max bounds
		minWidth, maxWidth := MinWidthComponent, MaxNodeWidth
		minHeight, maxHeight := MinHeightComponent, MaxNodeHeight

		// Adjust based on kind
		switch elem.Kind {
		case "person":
			minWidth, minHeight = MinWidthPerson, MinHeightPerson
		case "system":
			minWidth, minHeight = MinWidthSystem, MinHeightSystem
		case "container":
			minWidth, minHeight = MinWidthContainer, MinHeightContainer
		case "component":
			minWidth, minHeight = MinWidthComponent, MinHeightComponent
		case "datastore", "queue":
			minWidth, minHeight = MinWidthInfrastructure, MinHeightInfrastructure
		}

		// Hub detection: Increase size for high-degree nodes to allow routing space
		// Only apply hub scaling if we didn't get an explicit override, OR if override is smaller than min
		if _, hasOverride := config.NodeSizes[elem.ID]; !hasOverride {
			nodeDegree := degree[elem.ID]
			// Less aggressive hub scaling to prevent overlap
			if nodeDegree > HubDegreeThreshold {
				minWidth *= HubScaleWidth
				minHeight *= HubScaleHeight
			}
		}

		// If we have an override, we trust it, but we still ensure it meets minimum requirements
		// for the shape type if it's very small, though usually text measurement handles this.
		// However, we should be careful not to double-pad if the frontend already padded.
		// Assumption: Frontend sends exact size + padding.

		if _, hasOverride := config.NodeSizes[elem.ID]; hasOverride {
			// Use the override as the preferred size directly
			// We still allow it to grow if needed, but respect the measured size
			sizes = append(sizes, SizeConstraint{
				NodeID:          elem.ID,
				MinWidth:        width,    // Treat override as min to prevent shrinking
				MaxWidth:        maxWidth, // Allow expansion if layout engine really needs it (unlikely)
				MinHeight:       height,
				MaxHeight:       maxHeight,
				PreferredWidth:  width,
				PreferredHeight: height,
				FixedX:          fixedX,
				FixedY:          fixedY,
			})
			continue
		}

		// Fallback to heuristic sizing if no override

		// Add buffer padding to width/height to prevent overlaps
		// This provides extra space around nodes for better spacing
		bufferWidth := minWidth * BufferPaddingPercent   // 5% buffer
		bufferHeight := minHeight * BufferPaddingPercent // 5% buffer
		if width < minWidth {
			width = minWidth + bufferWidth
		} else {
			width += bufferWidth // Add buffer even if width is already >= minWidth
		}
		if height < minHeight {
			height = minHeight + bufferHeight
		} else {
			height += bufferHeight // Add buffer even if height is already >= minHeight
		}

		// Clamp to bounds after adding buffer
		if width > maxWidth {
			width = maxWidth
		}
		if height > maxHeight {
			height = maxHeight
		}

		sizes = append(sizes, SizeConstraint{
			NodeID:          elem.ID,
			MinWidth:        minWidth,
			MaxWidth:        maxWidth,
			MinHeight:       minHeight,
			MaxHeight:       maxHeight,
			PreferredWidth:  width,
			PreferredHeight: height,
			FixedX:          fixedX,
			FixedY:          fixedY,
		})
	}

	return sizes
}

// buildEdgeConstraints builds edge constraints from relations.
// FAANG pattern: Use higher weights and minlen to reduce crossings.
func buildEdgeConstraints(relations []*Relation, config Config, nodeCount int, parentMap map[string]string, viewLevel int) []EdgeConstraint {
	edges := make([]EdgeConstraint, 0, len(relations))

	// Track outgoing edges per node for weight distribution
	outDegree := make(map[string]int)
	for _, rel := range relations {
		outDegree[rel.From]++
	}

	for _, rel := range relations {
		edge := EdgeConstraint{
			From:          rel.From,
			To:            rel.To,
			AffectsLayout: true,
			MinLen:        1,
		}

		// Smarter edge weight based on label AND edge importance
		// FAANG pattern: More sophisticated weight distribution for crossing reduction
		if config.UseEdgeWeights {
			var baseWeight int

			// Labeled edges are more important - give them higher weight
			if rel.Label != "" {
				baseWeight = WeightLabeledEdge // Higher priority for labeled edges (was 20)
			} else {
				baseWeight = WeightUnlabeledEdge // Slightly higher base weight for unlabeled edges (was 3)
			}

			// Reduce weight for nodes with many outgoing edges (prevents star pattern issues)
			// But be less aggressive to maintain routing quality
			if outDegree[rel.From] > HighDegreeThreshold {
				baseWeight = baseWeight * HighDegreeReductionNumerator / HighDegreeReductionDenominator // Reduce by 1/4 (was 2/3 for >3)
			}

			// Increase weight for internal edges (same parent) to keep clusters tight
			if p1, ok1 := parentMap[rel.From]; ok1 && p1 != "" {
				if p2, ok2 := parentMap[rel.To]; ok2 && p2 != "" {
					if p1 == p2 {
						baseWeight += WeightInternalBoost // Boost internal edges
					}
				}
			}

			// Increase minlen for complex diagrams to reduce crossings
			// L2 uses polyline for better routing, L3 uses higher minlen
			// Increase minlen slightly for complex diagrams, but cap at 2-3
			if nodeCount >= ComplexGraphThreshold {
				edge.MinLen = 2 // Moderate separation (was 4)
			} else if nodeCount >= DenseGraphThreshold {
				edge.MinLen = 1 // Standard (was 3)
			} else {
				edge.MinLen = 1
			}

			// Level-specific edge routing optimizations
			if viewLevel == 2 && nodeCount >= 12 {
				edge.MinLen = 2
			}
			if viewLevel == 3 && nodeCount >= 15 {
				edge.MinLen = 2
			}

			// For very long distance edges in complex graphs, maybe bump to 3, but rarely.
			// Extreme case handling: very dense diagrams
			nodeCountForDensity := nodeCount
			if nodeCountForDensity < 1 {
				nodeCountForDensity = 1
			}
			edgeDensity := float64(len(relations)) / float64(nodeCountForDensity)
			if edgeDensity > 2.0 && nodeCount >= 20 {
				edge.MinLen = 2
			}

			edge.Weight = baseWeight
		} else {
			edge.Weight = 1
		}

		// Edge label positioning
		if rel.Label != "" {
			// Increase label distance for complex diagrams to reduce overlaps
			labelDistance := 1.5 // Default distance in inches
			if nodeCount > DenseGraphThreshold {
				labelDistance = 2.0 // More distance for dense diagrams
			}
			if nodeCount >= ComplexGraphThreshold {
				labelDistance = 2.5 // Even more distance for very complex diagrams
			}

			edge.Label = EdgeLabel{
				Text:     rel.Label,
				Distance: labelDistance,
				Angle:    0,   // Parallel to edge
				Position: 0.5, // Middle of edge
			}

			// For long labels, adjust positioning
			if len(rel.Label) > 40 {
				edge.Label.Angle = 0 // Keep parallel, avoids twisting reading
			}
		}

		edges = append(edges, edge)
	}

	return edges
}

// addSiblingClusterConstraints adds invisible edges between sibling clusters to keep them together.
// Inspired by PlantUML's "together" keyword, this groups sibling clusters visually.
func addSiblingClusterConstraints(edges []EdgeConstraint, elements []*Element, parentMap map[string]string) []EdgeConstraint {
	// Identify elements that will become clusters (elements that have children)
	hasChildren := make(map[string]bool)
	for _, elem := range elements {
		if elem.ParentID != "" {
			hasChildren[elem.ParentID] = true
		}
	}

	// Group sibling clusters by their parent
	siblingsByParent := make(map[string][]string)
	for _, elem := range elements {
		if hasChildren[elem.ID] {
			// This element will become a cluster (it has children)
			parent := elem.ParentID
			siblingsByParent[parent] = append(siblingsByParent[parent], elem.ID)
		}
	}

	// Add invisible edges between first and last sibling to keep clusters together
	for parent, siblings := range siblingsByParent {
		if len(siblings) > 1 && parent != "" {
			// Only add constraints for siblings that share a parent (not root-level elements)
			first := siblings[0]
			last := siblings[len(siblings)-1]
			edges = append(edges, EdgeConstraint{
				From:          first,
				To:            last,
				Weight:        1000, // High priority to keep siblings together
				MinLen:        1,
				AffectsLayout: true,
			})
		}
	}

	return edges
}

// pxToInchFloat converts pixels to inches (for float64).
func pxToInchFloat(px float64) float64 {
	return px / 72.0
}
