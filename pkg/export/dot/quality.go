// Package dot provides layout quality metrics for iterative refinement.
//
// This package measures actual layout quality from SVG output,
// providing accurate metrics for layout optimization.

package dot

import (
	"encoding/xml"
	"math"
	"strconv"
	"strings"
)

// LayoutQuality measures the quality of a layout.
type LayoutQuality struct {
	// EdgeCrossings is the number of edge crossings (lower is better)
	EdgeCrossings int
	// NodeOverlaps is the number of node overlaps (should be 0)
	NodeOverlaps int
	// LabelOverlaps is the number of label overlaps (should be 0)
	LabelOverlaps int
	// ParentChildContainment is the number of children outside parent bounds
	ParentChildContainment int
	// AvgEdgeLength is the average edge length
	AvgEdgeLength float64
	// EdgeLengthVariance is the variance in edge lengths (lower is better)
	EdgeLengthVariance float64
	// RankAlignment measures how well nodes align within ranks (0.0-1.0, higher is better)
	RankAlignment float64
	// SpacingConsistency measures spacing uniformity (0.0-1.0, higher is better)
	SpacingConsistency float64
	// ClusterBalance measures how balanced clusters are (0.0-1.0, higher is better)
	ClusterBalance float64
	// Score is the overall quality score (0.0-1.0, higher is better)
	Score float64
}

// SVGNode represents a node element from SVG output.
type SVGNode struct {
	ID      string  `xml:"id,attr"`
	X       float64 `xml:"x,attr"`
	Y       float64 `xml:"y,attr"`
	Width   float64 `xml:"width,attr"`
	Height  float64 `xml:"height,attr"`
	Class   string  `xml:"class,attr"`
	RX      float64 `xml:"rx,attr"` // Rounded corners
	RY      float64 `xml:"ry,attr"`
	Fill    string  `xml:"fill,attr"`
	Stroke  string  `xml:"stroke,attr"`
	StrokeW float64 `xml:"stroke-width,attr"`
}

// SVGEdge represents an edge element from SVG output.
type SVGEdge struct {
	ID        string   `xml:"id,attr"`
	Points    []Point  `xml:"points,attr"` // For polylines
	Path      string   `xml:"d,attr"`      // For paths (Bezier curves)
	MarkerEnd string   `xml:"marker-end,attr"`
	Class     string   `xml:"class,attr"`
	Label     *SVGText `xml:"title,attr,omitempty"` // Label as title element
}

// SVGText represents text elements.
type SVGText struct {
	X       float64 `xml:"x,attr"`
	Y       float64 `xml:"y,attr"`
	Content string  `xml:",chardata"`
}

// Point represents a 2D coordinate.
type Point struct {
	X, Y float64
}

// SVGRoot represents the root SVG element.
type SVGRoot struct {
	XMLName xml.Name   `xml:"svg"`
	ViewBox string     `xml:"viewBox,attr"`
	Width   float64    `xml:"width,attr"`
	Height  float64    `xml:"height,attr"`
	Nodes   []SVGNode  `xml:"rect"`
	Edges   []SVGEdge  `xml:"path"`
	Groups  []SVGGroup `xml:"g"`
}

// SVGGroup represents a group element.
type SVGGroup struct {
	ID     string     `xml:"id,attr"`
	Class  string     `xml:"class,attr"`
	Nodes  []SVGNode  `xml:"rect"`
	Edges  []SVGEdge  `xml:"path"`
	Groups []SVGGroup `xml:"g"`
}

// NeedsRefinement returns true if the layout quality needs improvement.
func (q LayoutQuality) NeedsRefinement() bool {
	return q.Score < 0.85 || q.EdgeCrossings > 0 || q.NodeOverlaps > 0 || q.ParentChildContainment > 0
}

// CalculateScore computes the overall quality score from metrics.
func (q *LayoutQuality) CalculateScore() {
	score := 1.0

	// Penalize edge crossings heavily (each crossing reduces score by 0.08, max 0.6 reduction)
	if q.EdgeCrossings > 0 {
		penalty := float64(q.EdgeCrossings) * 0.08
		if penalty > 0.6 {
			penalty = 0.6
		}
		score -= penalty
	}

	// Penalize node overlaps heavily (each overlap reduces score by 0.3, max 0.8 reduction)
	if q.NodeOverlaps > 0 {
		penalty := float64(q.NodeOverlaps) * 0.3
		if penalty > 0.8 {
			penalty = 0.8
		}
		score -= penalty
	}

	// Penalize label overlaps (each overlap reduces score by 0.15, max 0.4 reduction)
	if q.LabelOverlaps > 0 {
		penalty := float64(q.LabelOverlaps) * 0.15
		if penalty > 0.4 {
			penalty = 0.4
		}
		score -= penalty
	}

	// Penalize parent-child containment violations (each violation reduces score by 0.25, max 0.7 reduction)
	if q.ParentChildContainment > 0 {
		penalty := float64(q.ParentChildContainment) * 0.25
		if penalty > 0.7 {
			penalty = 0.7
		}
		score -= penalty
	}

	// Penalize poor rank alignment (reduces score by up to 0.25)
	if q.RankAlignment < 0.95 {
		score -= (0.95 - q.RankAlignment) * 0.25
	}

	// Penalize poor spacing consistency (reduces score by up to 0.2)
	if q.SpacingConsistency < 0.85 {
		score -= (0.85 - q.SpacingConsistency) * 0.2
	}

	// Penalize poor cluster balance (reduces score by up to 0.1)
	if q.ClusterBalance < 0.85 {
		score -= (0.85 - q.ClusterBalance) * 0.1
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

// MeasureQualityFromSVG parses SVG output and computes quality metrics.
func MeasureQualityFromSVG(svgContent string) LayoutQuality {
	quality := LayoutQuality{}

	// Parse SVG
	var svg SVGRoot
	if err := xml.Unmarshal([]byte(svgContent), &svg); err != nil {
		// Fall back to heuristic measurement if parsing fails
		return quality
	}

	// Extract all nodes
	allNodes := extractAllNodes(svg)

	// Extract all edges
	allEdges := extractAllEdges(svg)

	// Count edge crossings
	quality.EdgeCrossings = countEdgeCrossingsFromSVG(allNodes, allEdges)

	// Count node overlaps
	quality.NodeOverlaps = countNodeOverlapsFromSVG(allNodes)

	// Count label overlaps
	quality.LabelOverlaps = countLabelOverlapsFromSVG(allNodes, allEdges)

	// Calculate edge length statistics
	edgeLengths := calculateEdgeLengthsFromSVG(allEdges)
	if len(edgeLengths) > 0 {
		quality.AvgEdgeLength = average(edgeLengths)
		quality.EdgeLengthVariance = variance(edgeLengths, quality.AvgEdgeLength)
	}

	// Calculate rank alignment
	quality.RankAlignment = calculateRankAlignmentFromSVG(allNodes)

	// Calculate spacing consistency
	quality.SpacingConsistency = calculateSpacingConsistencyFromSVG(allNodes)

	// Calculate cluster balance
	quality.ClusterBalance = calculateClusterBalanceFromSVG(svg)

	// Calculate overall score
	quality.CalculateScore()

	return quality
}

// extractAllNodes recursively extracts all nodes from SVG.
func extractAllNodes(svg SVGRoot) []SVGNode {
	var nodes []SVGNode
	nodes = append(nodes, svg.Nodes...)
	for _, group := range svg.Groups {
		nodes = append(nodes, group.Nodes...)
		nodes = append(nodes, extractNodesFromGroup(group)...)
	}
	return nodes
}

// extractNodesFromGroup extracts nodes from a group.
func extractNodesFromGroup(group SVGGroup) []SVGNode {
	var nodes []SVGNode
	nodes = append(nodes, group.Nodes...)
	for _, g := range group.Groups {
		nodes = append(nodes, g.Nodes...)
		nodes = append(nodes, extractNodesFromGroup(g)...)
	}
	return nodes
}

// extractAllEdges recursively extracts all edges from SVG.
func extractAllEdges(svg SVGRoot) []SVGEdge {
	var edges []SVGEdge
	edges = append(edges, svg.Edges...)
	for _, group := range svg.Groups {
		edges = append(edges, group.Edges...)
		edges = append(edges, extractEdgesFromGroup(group)...)
	}
	return edges
}

// extractEdgesFromGroup extracts edges from a group.
func extractEdgesFromGroup(group SVGGroup) []SVGEdge {
	var edges []SVGEdge
	edges = append(edges, group.Edges...)
	for _, g := range group.Groups {
		edges = append(edges, g.Edges...)
		edges = append(edges, extractEdgesFromGroup(g)...)
	}
	return edges
}

// countEdgeCrossingsFromSVG counts actual edge crossings from SVG.
func countEdgeCrossingsFromSVG(nodes []SVGNode, edges []SVGEdge) int {
	if len(edges) < 2 {
		return 0
	}

	// Build node position map for edge endpoint lookup
	nodeMap := make(map[string]SVGNode)
	for _, node := range nodes {
		nodeMap[node.ID] = node
	}

	// Get edge paths as point lists
	var edgePaths [][]Point
	for _, edge := range edges {
		points := parseEdgePoints(edge, nodeMap)
		if len(points) >= 2 {
			edgePaths = append(edgePaths, points)
		}
	}

	// Count crossings
	crossings := 0
	for i := 0; i < len(edgePaths); i++ {
		for j := i + 1; j < len(edgePaths); j++ {
			if edgesShareNode(edgePaths[i], edgePaths[j]) {
				continue
			}
			if polylinesCross(edgePaths[i], edgePaths[j]) {
				crossings++
			}
		}
	}

	return crossings
}

// parseEdgePoints parses edge points from SVG edge.
func parseEdgePoints(edge SVGEdge, nodeMap map[string]SVGNode) []Point {
	var points []Point

	// Parse path data if available
	if edge.Path != "" {
		points = parsePathData(edge.Path)
	}

	// If no path but has points attribute
	if len(points) == 0 && len(edge.Points) > 0 {
		for _, p := range edge.Points {
			points = append(points, Point{X: p.X, Y: p.Y})
		}
	}

	return points
}

// parsePathData parses SVG path data (simplified).
func parsePathData(path string) []Point {
	var points []Point
	parts := strings.Fields(path)

	for _, part := range parts {
		// Handle different path commands
		if part == "M" || part == "L" || part == "C" || part == "Q" || part == "S" || part == "T" {
			continue
		}

		// Parse coordinate pair
		coords := strings.Split(part, ",")
		if len(coords) == 2 {
			x, _ := strconv.ParseFloat(coords[0], 64)
			y, _ := strconv.ParseFloat(coords[1], 64)
			points = append(points, Point{X: x, Y: y})
		}
	}

	return points
}

// edgesShareNode checks if two edge paths share a common endpoint.
func edgesShareNode(path1, path2 []Point) bool {
	if len(path1) == 0 || len(path2) == 0 {
		return false
	}

	// Check start points
	if pointsEqual(path1[0], path2[0]) || pointsEqual(path1[0], path2[len(path2)-1]) {
		return true
	}

	// Check end points
	if pointsEqual(path1[len(path1)-1], path2[0]) || pointsEqual(path1[len(path1)-1], path2[len(path2)-1]) {
		return true
	}

	return false
}

// pointsEqual checks if two points are approximately equal.
func pointsEqual(p1, p2 Point) bool {
	const epsilon = 1.0
	return abs(p1.X-p2.X) < epsilon && abs(p1.Y-p2.Y) < epsilon
}

// polylinesCross checks if two polylines cross.
func polylinesCross(poly1, poly2 []Point) bool {
	for i := 0; i < len(poly1)-1; i++ {
		for j := 0; j < len(poly2)-1; j++ {
			if lineSegmentsIntersect(poly1[i], poly1[i+1], poly2[j], poly2[j+1]) {
				return true
			}
		}
	}
	return false
}

// lineSegmentsIntersect checks if two line segments intersect.
func lineSegmentsIntersect(p1, p2, p3, p4 Point) bool {
	d1 := direction(p3, p4, p1)
	d2 := direction(p3, p4, p2)
	d3 := direction(p1, p2, p3)
	d4 := direction(p1, p2, p4)

	if ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) &&
		((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0)) {
		return true
	}

	return false
}

// direction calculates the direction of three points.
func direction(p1, p2, p3 Point) float64 {
	return (p2.X-p1.X)*(p3.Y-p1.Y) - (p2.Y-p1.Y)*(p3.X-p1.X)
}

// abs returns the absolute value.
func abs(x float64) float64 {
	if x < 0 {
		return -x
	}
	return x
}

// countNodeOverlapsFromSVG counts actual node overlaps from SVG.
func countNodeOverlapsFromSVG(nodes []SVGNode) int {
	overlaps := 0
	for i := 0; i < len(nodes); i++ {
		for j := i + 1; j < len(nodes); j++ {
			if boxesOverlap(nodes[i], nodes[j]) {
				overlaps++
			}
		}
	}
	return overlaps
}

// boxesOverlap checks if two node bounding boxes overlap.
func boxesOverlap(n1, n2 SVGNode) bool {
	r1 := getBounds(n1)
	r2 := getBounds(n2)

	return !(r1.Right < r2.Left || r2.Right < r1.Left || r1.Bottom < r2.Top || r2.Bottom < r1.Top)
}

// getBounds returns the bounding box of a node.
func getBounds(node SVGNode) Bounds {
	padding := 2.0 // Small padding for visual bounds
	return Bounds{
		Left:   node.X - padding,
		Right:  node.X + node.Width + padding,
		Top:    node.Y - padding,
		Bottom: node.Y + node.Height + padding,
	}
}

// Bounds represents a rectangular bounding box.
type Bounds struct {
	Left, Right, Top, Bottom float64
}

// countLabelOverlapsFromSVG counts label overlaps from SVG.
func countLabelOverlapsFromSVG(nodes []SVGNode, edges []SVGEdge) int {
	// Simplified: check if any text elements overlap with nodes
	overlaps := 0

	// This is a placeholder - actual implementation would need to
	// extract text positions from the SVG

	return overlaps
}

// calculateEdgeLengthsFromSVG calculates edge lengths from SVG.
func calculateEdgeLengthsFromSVG(edges []SVGEdge) []float64 {
	var lengths []float64
	for _, edge := range edges {
		points := parseEdgePoints(edge, make(map[string]SVGNode))
		if len(points) >= 2 {
			length := polylineLength(points)
			lengths = append(lengths, length)
		}
	}
	return lengths
}

// polylineLength calculates the length of a polyline.
func polylineLength(points []Point) float64 {
	length := 0.0
	for i := 0; i < len(points)-1; i++ {
		dx := points[i+1].X - points[i].X
		dy := points[i+1].Y - points[i].Y
		length += math.Sqrt(dx*dx + dy*dy)
	}
	return length
}

// average calculates the average of a slice of floats.
func average(values []float64) float64 {
	if len(values) == 0 {
		return 0
	}
	sum := 0.0
	for _, v := range values {
		sum += v
	}
	return sum / float64(len(values))
}

// variance calculates the variance of a slice of floats.
func variance(values []float64, mean float64) float64 {
	if len(values) == 0 {
		return 0
	}
	sum := 0.0
	for _, v := range values {
		sum += (v - mean) * (v - mean)
	}
	return sum / float64(len(values))
}

// calculateRankAlignmentFromSVG calculates rank alignment from SVG.
func calculateRankAlignmentFromSVG(nodes []SVGNode) float64 {
	if len(nodes) < 2 {
		return 1.0
	}

	// Group nodes by approximate Y position (rank)
	const tolerance = 15.0
	rankGroups := make(map[float64][]SVGNode)
	for _, node := range nodes {
		rankY := math.Round(node.Y/tolerance) * tolerance
		rankGroups[rankY] = append(rankGroups[rankY], node)
	}

	// Calculate alignment score for each rank
	totalScore := 0.0
	rankCount := 0

	for _, rankNodes := range rankGroups {
		if len(rankNodes) < 2 {
			totalScore += 1.0
			rankCount++
			continue
		}

		// Check vertical alignment (Y-spread)
		// We want nodes in the same rank to be aligned vertically (similar Y center)
		minY := rankNodes[0].Y + rankNodes[0].Height/2
		maxY := rankNodes[0].Y + rankNodes[0].Height/2

		for _, n := range rankNodes {
			centerY := n.Y + n.Height/2
			if centerY < minY {
				minY = centerY
			}
			if centerY > maxY {
				maxY = centerY
			}
		}

		ySpread := maxY - minY
		avgHeight := averageNodeHeight(rankNodes)

		// Score based on vertical spread relative to average node height
		// We DO NOT penalize horizontal spread (width of the row/rank),
		// as wide ranks are natural and valid in diagrams.
		var rankScore float64
		if ySpread < avgHeight*0.1 {
			rankScore = 1.0
		} else if ySpread < avgHeight*0.2 {
			rankScore = 0.9
		} else if ySpread < avgHeight*0.4 {
			rankScore = 0.75
		} else {
			rankScore = 0.5
		}

		totalScore += rankScore
		rankCount++
	}

	if rankCount == 0 {
		return 1.0
	}
	return totalScore / float64(rankCount)
}

// averageNodeWidth calculates the average width of nodes.
func averageNodeWidth(nodes []SVGNode) float64 {
	if len(nodes) == 0 {
		return 0
	}
	sum := 0.0
	for _, n := range nodes {
		sum += n.Width
	}
	return sum / float64(len(nodes))
}

// averageNodeHeight calculates the average height of nodes.
func averageNodeHeight(nodes []SVGNode) float64 {
	if len(nodes) == 0 {
		return 0
	}
	sum := 0.0
	for _, n := range nodes {
		sum += n.Height
	}
	return sum / float64(len(nodes))
}

// calculateSpacingConsistencyFromSVG calculates spacing consistency from SVG.
func calculateSpacingConsistencyFromSVG(nodes []SVGNode) float64 {
	if len(nodes) < 3 {
		return 1.0
	}

	// Calculate horizontal spacing between adjacent nodes
	var spacings []float64
	for i := 0; i < len(nodes)-1; i++ {
		for j := i + 1; j < len(nodes); j++ {
			// Check if nodes are approximately aligned vertically
			if areVerticallyAligned(nodes[i], nodes[j]) {
				spacing := nodes[j].X - (nodes[i].X + nodes[i].Width)
				if spacing > 0 {
					spacings = append(spacings, spacing)
				}
			}
		}
	}

	if len(spacings) < 2 {
		return 1.0
	}

	// Calculate coefficient of variation
	mean := average(spacings)
	if mean == 0 {
		return 1.0
	}
	cv := math.Sqrt(variance(spacings, mean)) / mean

	// Convert to score (lower CV = higher score)
	if cv < 0.2 {
		return 1.0
	} else if cv < 0.4 {
		return 0.9
	} else if cv < 0.6 {
		return 0.75
	}
	return 0.5
}

// areVerticallyAligned checks if two nodes are approximately vertically aligned.
func areVerticallyAligned(n1, n2 SVGNode) bool {
	const tolerance = 20.0
	centerY1 := n1.Y + n1.Height/2
	centerY2 := n2.Y + n2.Height/2
	return abs(centerY1-centerY2) < tolerance
}

// calculateClusterBalanceFromSVG calculates cluster balance from SVG.
func calculateClusterBalanceFromSVG(svg SVGRoot) float64 {
	// Count nodes per cluster and calculate balance
	clusterSizes := make(map[string]int)
	for _, group := range svg.Groups {
		if group.Class == "cluster" || strings.HasPrefix(group.ID, "cluster") {
			clusterSizes[group.ID] = len(group.Nodes)
		}
	}

	if len(clusterSizes) == 0 {
		return 0.9 // Assume reasonable balance if no clusters
	}

	// Calculate size variance
	sizes := make([]float64, 0, len(clusterSizes))
	for _, size := range clusterSizes {
		sizes = append(sizes, float64(size))
	}

	if len(sizes) == 0 {
		return 0.9
	}

	mean := average(sizes)
	if mean == 0 {
		return 0.9
	}

	cv := math.Sqrt(variance(sizes, mean)) / mean

	// Convert to score (lower variance = higher score)
	if cv < 0.3 {
		return 1.0
	} else if cv < 0.5 {
		return 0.85
	} else if cv < 0.7 {
		return 0.7
	}
	return 0.5
}
