// Package dot provides layout quality metrics for iterative refinement.
//
// ⚠️ DEVELOPER TOOL ONLY - For offline development use
// This code is NOT used in production or real-time user workflows.
// It's meant for developers to measure and improve layout quality during development.
//
// This implements FAANG-level quality measurement to drive layout optimization.

package dot

// LayoutQuality measures the quality of a layout.
type LayoutQuality struct {
	// EdgeCrossings is the number of edge crossings (lower is better)
	EdgeCrossings int
	// NodeOverlaps is the number of node overlaps (should be 0)
	NodeOverlaps int
	// LabelOverlaps is the number of label overlaps (should be 0)
	LabelOverlaps int
	// AvgEdgeLength is the average edge length
	AvgEdgeLength float64
	// EdgeLengthVariance is the variance in edge lengths (lower is better)
	EdgeLengthVariance float64
	// RankAlignment measures how well nodes align within ranks (0.0-1.0, higher is better)
	RankAlignment float64
	// ClusterBalance measures how balanced clusters are (0.0-1.0, higher is better)
	ClusterBalance float64
	// Score is the overall quality score (0.0-1.0, higher is better)
	Score float64
}

// NeedsRefinement returns true if the layout quality needs improvement.
func (q LayoutQuality) NeedsRefinement() bool {
	return q.Score < 0.7 || q.EdgeCrossings > 5 || q.NodeOverlaps > 0
}

// CalculateScore computes the overall quality score from metrics.
func (q *LayoutQuality) CalculateScore() {
	score := 1.0

	// Penalize edge crossings (each crossing reduces score by 0.05, max 0.5 reduction)
	if q.EdgeCrossings > 0 {
		penalty := float64(q.EdgeCrossings) * 0.05
		if penalty > 0.5 {
			penalty = 0.5
		}
		score -= penalty
	}

	// Penalize node overlaps heavily (each overlap reduces score by 0.2, max 0.6 reduction)
	if q.NodeOverlaps > 0 {
		penalty := float64(q.NodeOverlaps) * 0.2
		if penalty > 0.6 {
			penalty = 0.6
		}
		score -= penalty
	}

	// Penalize label overlaps (each overlap reduces score by 0.1, max 0.3 reduction)
	if q.LabelOverlaps > 0 {
		penalty := float64(q.LabelOverlaps) * 0.1
		if penalty > 0.3 {
			penalty = 0.3
		}
		score -= penalty
	}

	// Penalize poor rank alignment (reduces score by up to 0.2)
	if q.RankAlignment < 0.9 {
		score -= (0.9 - q.RankAlignment) * 0.2
	}

	// Penalize poor cluster balance (reduces score by up to 0.1)
	if q.ClusterBalance < 0.8 {
		score -= (0.8 - q.ClusterBalance) * 0.1
	}

	// Ensure score is in valid range
	if score < 0.0 {
		score = 0.0
	}
	if score > 1.0 {
		score = 1.0
	}

	q.Score = score
}

// MeasureQuality analyzes a Graphviz layout result and computes quality metrics.
// Note: This is a placeholder implementation. In a full implementation, we would
// parse the Graphviz JSON output to compute actual metrics.
func MeasureQuality(_ string, elements []*Element, relations []*Relation) LayoutQuality {
	quality := LayoutQuality{
		// For now, we estimate based on graph structure
		// A full implementation would parse Graphviz JSON output
		EdgeCrossings:  estimateEdgeCrossings(elements, relations),
		NodeOverlaps:   estimateNodeOverlaps(elements),
		LabelOverlaps:  0,    // Would need Graphviz output to measure
		RankAlignment:  0.95, // Assume good alignment if rank constraints are used
		ClusterBalance: 0.9,  // Assume reasonable balance
	}

	// Calculate overall score
	quality.CalculateScore()

	return quality
}

// estimateEdgeCrossings estimates edge crossings based on graph structure.
// This is a simplified heuristic - a full implementation would use Graphviz output.
func estimateEdgeCrossings(elements []*Element, relations []*Relation) int {
	if len(elements) < 2 {
		return 0
	}

	// Calculate graph density: E / (V * (V-1))
	// For directed graphs, max edges is V * (V-1)
	numNodes := float64(len(elements))
	maxEdges := numNodes * (numNodes - 1)
	if maxEdges == 0 {
		return 0
	}

	density := float64(len(relations)) / maxEdges

	// Map density to expected crossings
	// Heuristic: Higher density = significantly more crossings
	// 0-10% density: very few crossings
	// >30% density: many crossings
	if density < 0.1 {
		return int(float64(len(relations)) * 0.05) // Estimate 5% of edges might cross
	} else if density < 0.3 {
		return int(float64(len(relations)) * 0.15) // Estimate 15% of edges might cross
	} else {
		return int(float64(len(relations)) * 0.30) // Estimate 30% of edges might cross
	}
}

// estimateNodeOverlaps estimates node overlaps.
// This is a simplified heuristic - a full implementation would use Graphviz output.
func estimateNodeOverlaps(elements []*Element) int {
	// With proper constraint-based layout, purely algorithmic overlaps should be rare.
	// However, if we have very many nodes, the probability increases.
	if len(elements) > 50 {
		return int(float64(len(elements)) * 0.01) // Estimate 1% overlap for very large graphs
	}
	return 0
}
