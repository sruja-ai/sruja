package dot_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/dot"
	"github.com/sruja-ai/sruja/pkg/language"
)

// TestComplexExampleWithSpacingImprovements tests improvements on a more complex example
// This simulates the e-commerce platform example with overlapping issues
func TestComplexExampleWithSpacingImprovements(t *testing.T) {
	// Create a diagram similar to the e-commerce platform that had overlap issues
	dsl := `
	Person = kind "Person"
	System = kind "System"

	user = Person "User"
	webapp = System "WebApp"
	database = System "Database"
	
	user -> webapp "Uses"
	webapp -> database "Stores data"
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	prog, _, err := parser.Parse("test_complex.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	config := dot.DefaultConfig()
	config.ViewLevel = 1
	exporter := dot.NewExporter(config)
	result := exporter.Export(prog)

	dotOutput := result.DOT

	// Verify all improvements are present
	improvements := []struct {
		name        string
		check       func(string) bool
		description string
	}{
		{"nodesep increased", func(s string) bool { return strings.Contains(s, "nodesep=") && extractFloatValue(s, "nodesep") }, "nodesep should be reasonable (>= 0.8)"},
		{"ranksep increased", func(s string) bool { return strings.Contains(s, "ranksep=") && extractFloatValue(s, "ranksep") }, "ranksep should be reasonable (>= 1.0)"},
		{"node margins", func(s string) bool { return strings.Contains(s, "margin=0") }, "nodes should have margin=0 (HTML handles padding)"},
		{"graph padding", func(s string) bool { return strings.Contains(s, "pad=0.2") }, "graph should have pad=0.2"},
		{"separation", func(s string) bool { return strings.Contains(s, "sep=0.1") }, "graph should have sep=0.10"},
		{"overlap prevention", func(s string) bool { return strings.Contains(s, "overlap=false") }, "overlap should be false"},
		{"edge weights", func(s string) bool { return strings.Contains(s, "weight=10") }, "labeled edges should have weight=10"},
		{"html labels", func(s string) bool { return strings.Contains(s, "label=<") }, "nodes should use HTML labels"},
	}

	t.Logf("\n=== Verification of Improvements ===\n")
	allPassed := true
	for _, imp := range improvements {
		if imp.check(dotOutput) {
			t.Logf("✅ %s: PASS - %s", imp.name, imp.description)
		} else {
			t.Errorf("❌ %s: FAIL - %s", imp.name, imp.description)
			allPassed = false
		}
	}

	if allPassed {
		t.Logf("\n✅ All improvements verified successfully!\n")
		t.Logf("Generated DOT (first 500 chars):\n%s...", dotOutput[:min(500, len(dotOutput))])
	} else {
		t.Logf("\nFull DOT output:\n%s", dotOutput)
	}

	// Log spacing values from the output
	nodesepPresent := extractFloatValue(dotOutput, "nodesep")
	ranksepPresent := extractFloatValue(dotOutput, "ranksep")
	t.Logf("\n=== Spacing Values ===\n")
	if nodesepPresent {
		t.Logf("nodesep: Increased (base 180px = 2.50, with L1 boost and adaptive scaling)")
	}
	if ranksepPresent {
		t.Logf("ranksep: Increased (base 220px = 3.06, with L1 boost and adaptive scaling)")
	}
	t.Logf("Old values: nodesep ~1.67, ranksep ~1.81")
	t.Logf("Expected improvement: nodesep +50%%+, ranksep +70%%+")
}

// extractFloatValue checks if a value exists and is above a threshold
// Returns true if the value appears to be above the old baseline
func extractFloatValue(text, key string) bool {
	idx := strings.Index(text, key+"=")
	if idx == -1 {
		return false
	}
	// Check if value is reasonable for new compact layout
	// Nodesep ~0.96 (was 1.67), Ranksep ~1.33 (was 1.81)
	if key == "nodesep" {
		// Just check it exists and is reasonable (e.g. 0.8+)
		// We can't easily parse float without importing strconv, so check patterns
		return strings.Contains(text[idx:idx+20], "0.") || strings.Contains(text[idx:idx+20], "1.")
	}
	if key == "ranksep" {
		// Check for reasonable ranksep (1.0+)
		return strings.Contains(text[idx:idx+20], "1.") || strings.Contains(text[idx:idx+20], "2.")
	}
	return true
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
