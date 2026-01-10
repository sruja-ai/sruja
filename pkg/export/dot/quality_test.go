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
