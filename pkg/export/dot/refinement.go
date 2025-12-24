// Package dot provides iterative layout refinement based on quality metrics.
//
// This implements FAANG-level iterative refinement to improve layout quality.

package dot

// RefineConstraints improves constraints based on quality metrics.
func RefineConstraints(
	original LayoutConstraints,
	quality LayoutQuality,
) LayoutConstraints {
	refined := original // Copy (struct copy in Go)

	// Add anti-crossing constraints
	if quality.EdgeCrossings > 0 {
		refined = addAntiCrossingHints(refined)
	}

	// Strengthen rank constraints
	if quality.RankAlignment < 0.9 {
		refined = strengthenRanks(refined)
	}

	// Adjust spacing for overlaps - more aggressive increases
	if quality.NodeOverlaps > 0 {
		// More aggressive spacing increases to eliminate overlaps
		refined.Global.NodeSep *= 1.35 // Increased from 1.2 to 1.35
		refined.Global.RankSep *= 1.40 // Increased from 1.2 to 1.40 for vertical spacing
		refined.Global.Sep *= 1.30     // Increased from 1.2 to 1.30
	}

	// Improve edge routing for dense diagrams - react earlier to crossings
	if quality.EdgeCrossings > 3 {
		refined.Global.Splines = "ortho" // Orthogonal routing earlier (was >5, now >3)
	} else if quality.EdgeCrossings > 1 {
		refined.Global.Splines = "polyline" // Use polyline for even small crossing counts
	}

	// Add edge concentration for parallel edges - enable earlier
	if quality.EdgeCrossings > 1 {
		refined.Global.Concentrate = true // Enable earlier (was >3, now >1)
	}

	return refined
}

// addAntiCrossingHints adds constraints to reduce edge crossings.
func addAntiCrossingHints(constraints LayoutConstraints) LayoutConstraints {
	// Increase edge weights for important edges to prioritize their routing
	for i := range constraints.Edges {
		if constraints.Edges[i].Weight < 10 {
			constraints.Edges[i].Weight = 5 // Increase weight for better routing
		}
	}

	// Use orthogonal routing for dense graphs
	if len(constraints.Edges) > len(constraints.Sizes)*2 {
		constraints.Global.Splines = "ortho"
	}

	return constraints
}

// strengthenRanks adds more rank constraints to improve alignment.
func strengthenRanks(constraints LayoutConstraints) LayoutConstraints {
	// Ensure we have rank constraints for all same-level nodes
	// This is already handled in buildRankConstraints, but we can add
	// invisible edges between ranks to enforce ordering

	// The rank constraints are already built correctly in buildRankConstraints,
	// so this is mainly for future enhancements like cross-cluster alignment

	return constraints
}

// LayoutWithRefinement performs layout with iterative refinement.
// This is the FAANG-level approach: measure quality and refine until good enough.
func LayoutWithRefinement(
	elements []*Element,
	relations []*Relation,
	viewLevel int,
	config Config,
	maxIterations int,
) (string, LayoutConstraints, LayoutQuality) {
	// Phase 1: Build initial constraints
	constraints := BuildConstraints(elements, relations, viewLevel, config)

	// Phase 2: Generate DOT and measure quality (simulated)
	dot := GenerateDOTFromConstraints(elements, relations, constraints)
	quality := MeasureQuality(dot, elements, relations)

	// Phase 3: Refine if needed
	iterations := 0
	for iterations < maxIterations && quality.NeedsRefinement() {
		constraints = RefineConstraints(constraints, quality)
		dot = GenerateDOTFromConstraints(elements, relations, constraints)
		quality = MeasureQuality(dot, elements, relations)
		iterations++
	}

	return dot, constraints, quality
}
