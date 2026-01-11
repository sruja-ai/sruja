package dot

import (
	"testing"
)

func TestLayoutQuality_NeedsRefinement(t *testing.T) {
	tests := []struct {
		name     string
		quality  LayoutQuality
		expected bool
	}{
		{
			name:     "high score no issues",
			quality:  LayoutQuality{Score: 0.9, EdgeCrossings: 0, NodeOverlaps: 0},
			expected: false,
		},
		{
			name:     "low score",
			quality:  LayoutQuality{Score: 0.5, EdgeCrossings: 0, NodeOverlaps: 0},
			expected: true,
		},
		{
			name:     "too many edge crossings",
			quality:  LayoutQuality{Score: 0.8, EdgeCrossings: 10, NodeOverlaps: 0},
			expected: true,
		},
		{
			name:     "node overlaps",
			quality:  LayoutQuality{Score: 0.8, EdgeCrossings: 0, NodeOverlaps: 1},
			expected: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.quality.NeedsRefinement(); got != tt.expected {
				t.Errorf("NeedsRefinement() = %v, want %v", got, tt.expected)
			}
		})
	}
}

func TestLayoutQuality_CalculateScore(t *testing.T) {
	tests := []struct {
		name     string
		quality  LayoutQuality
		minScore float64
		maxScore float64
	}{
		{
			name:     "perfect (no issues)",
			quality:  LayoutQuality{EdgeCrossings: 0, NodeOverlaps: 0, LabelOverlaps: 0, RankAlignment: 0.95, ClusterBalance: 0.9, SpacingConsistency: 0.9},
			minScore: 0.9,
			maxScore: 1.0,
		},
		{
			name:     "edge crossings",
			quality:  LayoutQuality{EdgeCrossings: 5, NodeOverlaps: 0, LabelOverlaps: 0, RankAlignment: 0.95, ClusterBalance: 0.9, SpacingConsistency: 0.9},
			minScore: 0.4,
			maxScore: 0.6,
		},
		{
			name:     "node overlaps (heavy penalty)",
			quality:  LayoutQuality{EdgeCrossings: 0, NodeOverlaps: 2, LabelOverlaps: 0, RankAlignment: 0.95, ClusterBalance: 0.9, SpacingConsistency: 0.9},
			minScore: 0.2,
			maxScore: 0.4,
		},
		{
			name:     "label overlaps",
			quality:  LayoutQuality{EdgeCrossings: 0, NodeOverlaps: 0, LabelOverlaps: 2, RankAlignment: 0.95, ClusterBalance: 0.9, SpacingConsistency: 0.9},
			minScore: 0.5,
			maxScore: 0.7,
		},
		{
			name:     "poor rank alignment",
			quality:  LayoutQuality{EdgeCrossings: 0, NodeOverlaps: 0, LabelOverlaps: 0, RankAlignment: 0.5, ClusterBalance: 0.9, SpacingConsistency: 0.9},
			minScore: 0.7,
			maxScore: 0.9,
		},
		{
			name:     "poor cluster balance",
			quality:  LayoutQuality{EdgeCrossings: 0, NodeOverlaps: 0, LabelOverlaps: 0, RankAlignment: 0.95, ClusterBalance: 0.5, SpacingConsistency: 0.9},
			minScore: 0.9,
			maxScore: 1.0,
		},
		{
			name:     "all maximum penalties",
			quality:  LayoutQuality{EdgeCrossings: 20, NodeOverlaps: 10, LabelOverlaps: 10, RankAlignment: 0.0, ClusterBalance: 0.0, SpacingConsistency: 0.0},
			minScore: 0.0,
			maxScore: 0.1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.quality.CalculateScore()
			if tt.quality.Score < tt.minScore || tt.quality.Score > tt.maxScore {
				t.Errorf("CalculateScore() = %v, want between %v and %v", tt.quality.Score, tt.minScore, tt.maxScore)
			}
		})
	}
}

func TestMeasureQualityFromSVG(t *testing.T) {
	svgContent := `<?xml version="1.0" encoding="UTF-8"?>
<svg width="500" height="300" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
  <g id="nodeA" transform="translate(50,50)">
    <polygon points="0,0 100,0 100,50 0,50" fill="#ffffff" stroke="#333333"/>
    <text x="50" y="30" text-anchor="middle">System A</text>
  </g>
  <g id="nodeB" transform="translate(200,50)">
    <polygon points="0,0 100,0 100,50 0,50" fill="#ffffff" stroke="#333333"/>
    <text x="50" y="30" text-anchor="middle">Container B</text>
  </g>
  <path id="edgeA_B" d="M150,75 C175,75 175,75 200,75" fill="none" stroke="#596980" stroke-width="2"/>
</svg>`

	quality := MeasureQualityFromSVG(svgContent)

	if quality.Score < 0 || quality.Score > 1.0 {
		t.Errorf("MeasureQualityFromSVG score out of range: %v", quality.Score)
	}
}

func TestMeasureQualityFromSVG_EmptySVG(t *testing.T) {
	svgContent := ""

	quality := MeasureQualityFromSVG(svgContent)

	if quality.Score < 0 || quality.Score > 1.0 {
		t.Errorf("MeasureQualityFromSVG should handle empty SVG, got score: %v", quality.Score)
	}
}

func TestEscapeID(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"simple", "simple"},
		{"with space", "with space"}, // escapeID doesn't quote
		{"with.dot", "with.dot"},     // escapeID doesn't quote
		{"with-dash", "with-dash"},   // escapeID doesn't quote
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			got := escapeID(tt.input)
			if got != tt.expected {
				t.Errorf("escapeID(%q) = %q, want %q", tt.input, got, tt.expected)
			}
		})
	}
}

func TestEscapeLabel(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"simple", "simple"},
		{`with "quotes"`, `with \"quotes\"`},
		{"with\nnewline", "with\nnewline"},     // escapeLabel keeps literal newlines
		{"with\\backslash", "with\\backslash"}, // escapeLabel keeps single backslash
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			got := escapeLabel(tt.input)
			if got != tt.expected {
				t.Errorf("escapeLabel(%q) = %q, want %q", tt.input, got, tt.expected)
			}
		})
	}
}

func TestPxToInch(t *testing.T) {
	// 72 DPI means 72 pixels = 1 inch
	result := pxToInchFloat(72)
	if result != 1.0 {
		t.Errorf("pxToInchFloat(72) = %v, want 1.0", result)
	}

	result = pxToInchFloat(144)
	if result != 2.0 {
		t.Errorf("pxToInchFloat(144) = %v, want 2.0", result)
	}
}

func TestDefaultConfig(t *testing.T) {
	config := DefaultConfig()

	// Test expected default values
	if config.RankDir != "TB" {
		t.Errorf("Expected RankDir TB, got %s", config.RankDir)
	}
	if config.NodeSep != DefaultNodeSep {
		t.Errorf("Expected NodeSep %d, got %d", DefaultNodeSep, config.NodeSep)
	}
	if !config.UseRankConstraints {
		t.Error("Expected UseRankConstraints to be true")
	}
	if !config.UseEdgeWeights {
		t.Error("Expected UseEdgeWeights to be true")
	}
}

func TestNewExporter(t *testing.T) {
	config := DefaultConfig()
	exporter := NewExporter(config)

	if exporter == nil {
		t.Fatal("NewExporter returned nil")
	}
	if exporter.Config.RankDir != config.RankDir {
		t.Error("Config not set correctly")
	}
}

func TestAbs(t *testing.T) {
	tests := []struct {
		name     string
		input    float64
		expected float64
	}{
		{"positive", 5.0, 5.0},
		{"negative", -5.0, 5.0},
		{"zero", 0.0, 0.0},
		{"small positive", 0.1, 0.1},
		{"small negative", -0.1, 0.1},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := abs(tt.input)
			if got != tt.expected {
				t.Errorf("abs(%v) = %v, want %v", tt.input, got, tt.expected)
			}
		})
	}
}

func TestPointsEqual(t *testing.T) {
	tests := []struct {
		name     string
		p1       Point
		p2       Point
		expected bool
	}{
		{"identical", Point{X: 10, Y: 20}, Point{X: 10, Y: 20}, true},
		{"within epsilon", Point{X: 10, Y: 20}, Point{X: 10.5, Y: 20.5}, true},
		{"outside epsilon", Point{X: 10, Y: 20}, Point{X: 12, Y: 22}, false},
		{"x different", Point{X: 10, Y: 20}, Point{X: 15, Y: 20}, false},
		{"y different", Point{X: 10, Y: 20}, Point{X: 10, Y: 25}, false},
		{"far apart", Point{X: 0, Y: 0}, Point{X: 100, Y: 100}, false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := pointsEqual(tt.p1, tt.p2)
			if got != tt.expected {
				t.Errorf("pointsEqual(%v, %v) = %v, want %v", tt.p1, tt.p2, got, tt.expected)
			}
		})
	}
}

func TestEdgesShareNode(t *testing.T) {
	tests := []struct {
		name  string
		path1 []Point
		path2 []Point
		want  bool
	}{
		{
			name:  "share start point",
			path1: []Point{{0, 0}, {10, 10}},
			path2: []Point{{0, 0}, {20, 20}},
			want:  true,
		},
		{
			name:  "share end point",
			path1: []Point{{0, 0}, {10, 10}},
			path2: []Point{{5, 5}, {10, 10}},
			want:  true,
		},
		{
			name:  "first start matches second end",
			path1: []Point{{0, 0}, {10, 10}},
			path2: []Point{{10, 10}, {20, 20}},
			want:  true,
		},
		{
			name:  "first end matches second start",
			path1: []Point{{0, 0}, {10, 10}},
			path2: []Point{{10, 10}, {20, 20}},
			want:  true,
		},
		{
			name:  "no shared points",
			path1: []Point{{0, 0}, {10, 10}},
			path2: []Point{{20, 20}, {30, 30}},
			want:  false,
		},
		{
			name:  "empty path1",
			path1: []Point{},
			path2: []Point{{0, 0}, {10, 10}},
			want:  false,
		},
		{
			name:  "empty path2",
			path1: []Point{{0, 0}, {10, 10}},
			path2: []Point{},
			want:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := edgesShareNode(tt.path1, tt.path2)
			if got != tt.want {
				t.Errorf("edgesShareNode() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDirection(t *testing.T) {
	tests := []struct {
		name     string
		p1       Point
		p2       Point
		p3       Point
		expected float64
	}{
		{"collinear horizontal", Point{X: 0, Y: 0}, Point{X: 1, Y: 0}, Point{X: 2, Y: 0}, 0.0},
		{"collinear vertical", Point{X: 0, Y: 0}, Point{X: 0, Y: 1}, Point{X: 0, Y: 2}, 0.0},
		{"left turn", Point{X: 0, Y: 0}, Point{X: 1, Y: 0}, Point{X: 1, Y: 1}, 1.0},
		{"right turn", Point{X: 0, Y: 0}, Point{X: 1, Y: 0}, Point{X: 1, Y: -1}, -1.0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := direction(tt.p1, tt.p2, tt.p3)
			if tt.expected != 0.0 && (got > 0) != (tt.expected > 0) {
				t.Errorf("direction(%v, %v, %v) = %v, expected sign %v", tt.p1, tt.p2, tt.p3, got, tt.expected)
			}
		})
	}
}

func TestLineSegmentsIntersect(t *testing.T) {
	tests := []struct {
		name string
		p1   Point
		p2   Point
		p3   Point
		p4   Point
		want bool
	}{
		{
			name: "intersecting",
			p1:   Point{X: 0, Y: 0},
			p2:   Point{X: 2, Y: 2},
			p3:   Point{X: 0, Y: 2},
			p4:   Point{X: 2, Y: 0},
			want: true,
		},
		{
			name: "parallel horizontal",
			p1:   Point{X: 0, Y: 0},
			p2:   Point{X: 2, Y: 0},
			p3:   Point{X: 0, Y: 1},
			p4:   Point{X: 2, Y: 1},
			want: false,
		},
		{
			name: "parallel vertical",
			p1:   Point{X: 0, Y: 0},
			p2:   Point{X: 0, Y: 2},
			p3:   Point{X: 1, Y: 0},
			p4:   Point{X: 1, Y: 2},
			want: false,
		},
		{
			name: "non-intersecting",
			p1:   Point{X: 0, Y: 0},
			p2:   Point{X: 1, Y: 1},
			p3:   Point{X: 2, Y: 2},
			p4:   Point{X: 3, Y: 3},
			want: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := lineSegmentsIntersect(tt.p1, tt.p2, tt.p3, tt.p4)
			if got != tt.want {
				t.Errorf("lineSegmentsIntersect() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestPolylinesCross(t *testing.T) {
	tests := []struct {
		name  string
		poly1 []Point
		poly2 []Point
		want  bool
	}{
		{
			name:  "crossing polylines",
			poly1: []Point{{0, 0}, {2, 2}},
			poly2: []Point{{0, 2}, {2, 0}},
			want:  true,
		},
		{
			name:  "non-crossing",
			poly1: []Point{{0, 0}, {1, 1}},
			poly2: []Point{{2, 2}, {3, 3}},
			want:  false,
		},
		{
			name:  "parallel polylines",
			poly1: []Point{{0, 0}, {2, 0}},
			poly2: []Point{{0, 1}, {2, 1}},
			want:  false,
		},
		{
			name:  "single point poly1",
			poly1: []Point{{0, 0}},
			poly2: []Point{{0, 1}, {1, 1}},
			want:  false,
		},
		{
			name:  "single point poly2",
			poly1: []Point{{0, 0}, {1, 1}},
			poly2: []Point{{0, 1}},
			want:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := polylinesCross(tt.poly1, tt.poly2)
			if got != tt.want {
				t.Errorf("polylinesCross() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestGetBounds(t *testing.T) {
	tests := []struct {
		name     string
		node     SVGNode
		expected Bounds
	}{
		{
			name:     "basic node",
			node:     SVGNode{X: 10, Y: 20, Width: 100, Height: 50},
			expected: Bounds{Left: 8, Right: 112, Top: 18, Bottom: 72},
		},
		{
			name:     "zero position",
			node:     SVGNode{X: 0, Y: 0, Width: 50, Height: 30},
			expected: Bounds{Left: -2, Right: 52, Top: -2, Bottom: 32},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := getBounds(tt.node)
			if got.Left != tt.expected.Left || got.Right != tt.expected.Right ||
				got.Top != tt.expected.Top || got.Bottom != tt.expected.Bottom {
				t.Errorf("getBounds() = %v, want %v", got, tt.expected)
			}
		})
	}
}

func TestBoxesOverlap(t *testing.T) {
	tests := []struct {
		name string
		n1   SVGNode
		n2   SVGNode
		want bool
	}{
		{
			name: "overlapping",
			n1:   SVGNode{X: 10, Y: 10, Width: 50, Height: 50},
			n2:   SVGNode{X: 30, Y: 30, Width: 50, Height: 50},
			want: true,
		},
		{
			name: "not overlapping",
			n1:   SVGNode{X: 10, Y: 10, Width: 50, Height: 50},
			n2:   SVGNode{X: 100, Y: 100, Width: 50, Height: 50},
			want: false,
		},
		{
			name: "one inside other",
			n1:   SVGNode{X: 10, Y: 10, Width: 100, Height: 100},
			n2:   SVGNode{X: 30, Y: 30, Width: 20, Height: 20},
			want: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := boxesOverlap(tt.n1, tt.n2)
			if got != tt.want {
				t.Errorf("boxesOverlap() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAverageNodeHeight(t *testing.T) {
	tests := []struct {
		name  string
		nodes []SVGNode
		want  float64
	}{
		{
			name:  "empty slice",
			nodes: []SVGNode{},
			want:  0.0,
		},
		{
			name:  "single node",
			nodes: []SVGNode{{Height: 50}},
			want:  50.0,
		},
		{
			name:  "multiple nodes",
			nodes: []SVGNode{{Height: 30}, {Height: 50}, {Height: 70}},
			want:  50.0,
		},
		{
			name:  "same heights",
			nodes: []SVGNode{{Height: 40}, {Height: 40}, {Height: 40}},
			want:  40.0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := averageNodeHeight(tt.nodes)
			if got != tt.want {
				t.Errorf("averageNodeHeight() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAreVerticallyAligned(t *testing.T) {
	tests := []struct {
		name string
		n1   SVGNode
		n2   SVGNode
		want bool
	}{
		{
			name: "aligned within tolerance",
			n1:   SVGNode{Y: 10, Height: 20}, // center at Y=20
			n2:   SVGNode{Y: 20, Height: 20}, // center at Y=30, diff=10 < 20
			want: true,
		},
		{
			name: "perfectly aligned",
			n1:   SVGNode{Y: 10, Height: 20}, // center at Y=20
			n2:   SVGNode{Y: 10, Height: 20}, // center at Y=20
			want: true,
		},
		{
			name: "not aligned",
			n1:   SVGNode{Y: 10, Height: 20}, // center at Y=20
			n2:   SVGNode{Y: 50, Height: 20}, // center at Y=60, diff=40
			want: false,
		},
		{
			name: "just outside tolerance",
			n1:   SVGNode{Y: 10, Height: 20}, // center at Y=20
			n2:   SVGNode{Y: 31, Height: 20}, // center at Y=41, diff=21
			want: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := areVerticallyAligned(tt.n1, tt.n2)
			if got != tt.want {
				t.Errorf("areVerticallyAligned() = %v, want %v", got, tt.want)
			}
		})
	}
}
