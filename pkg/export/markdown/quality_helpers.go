// pkg/export/markdown/quality_helpers.go
// Package markdown provides helper functions for quality attributes extraction.
package markdown

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

// extractQualityRequirements extracts performance/constraint requirements
func extractQualityRequirements(arch *language.Architecture) []string {
	var items []string
	for _, r := range arch.Requirements {
		if strVal(r.Type) == "performance" || strVal(r.Type) == "constraint" {
			items = append(items, fmt.Sprintf("- %s\n", strVal(r.Description)))
		}
	}
	return items
}

// extractQualityProperties extracts quality-related properties
func extractQualityProperties(arch *language.Architecture) []string {
	var items []string
	if arch.Properties == nil {
		return items
	}
	qualityKeys := []string{"qps", "latency", "availability", "throughput"}
	for k, v := range arch.Properties {
		if contains(qualityKeys, k) {
			items = append(items, fmt.Sprintf("- %s: %s\n", k, v))
		}
	}
	return items
}

// contains checks if a slice contains a string
func contains(slice []string, item string) bool {
	for _, s := range slice {
		if s == item {
			return true
		}
	}
	return false
}



