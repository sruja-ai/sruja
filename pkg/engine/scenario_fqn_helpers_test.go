// pkg/engine/scenario_fqn_helpers_test.go
// Tests for scenario FQN helper functions
package engine

import (
	"testing"
)

func TestCalculateSimilarityForScenario(t *testing.T) {
	tests := []struct {
		s1       string
		s2       string
		expected float64
	}{
		{"checkout", "checkout", 1.0},
		{"checkout", "Checkout", 0.875}, // Character-by-character: 7/8 match
		{"checkout", "check", 0.7},      // Contains check
		{"payment", "payment", 1.0},
		{"payment", "pay", 0.7}, // Contains check
		{"", "", 1.0},
		{"abc", "def", 0.0},      // No match
		{"abc", "ab", 0.7},       // Contains check
		{"hello", "world", 0.2},  // Character-by-character: 1/5 match (h matches position)
		{"test", "testing", 0.7}, // Contains check
	}

	for _, tt := range tests {
		t.Run(tt.s1+"_"+tt.s2, func(t *testing.T) {
			result := calculateSimilarityForScenario(tt.s1, tt.s2)
			if result != tt.expected {
				t.Errorf("calculateSimilarityForScenario(%q, %q) = %f, want %f", tt.s1, tt.s2, result, tt.expected)
			}
		})
	}
}
