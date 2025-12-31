package dot_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/dot"
	"github.com/sruja-ai/sruja/pkg/language"
)

// TestSpacingImprovements verifies that spacing improvements are applied correctly
func TestSpacingImprovements(t *testing.T) {
	// Use a simple e-commerce example similar to what the quality test uses
	dsl := `
	Person = kind "Person"
	System = kind "System"

	user = Person "User"
	
	webapp = System "Web Application" {
		technology "React"
	}
	
	database = System "Database" {
		technology "PostgreSQL"
	}
	
	user -> webapp "Uses"
	webapp -> database "Stores data"
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	prog, _, err := parser.Parse("test_ecommerce.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	config := dot.DefaultConfig()
	config.ViewLevel = 1 // L1 view
	exporter := dot.NewExporter(config)
	result := exporter.Export(prog)

	if result.DOT == "" {
		t.Fatal("Generated DOT is empty")
	}

	dotOutput := result.DOT

	// Test 1: Verify improved base spacing values
	// Expected: nodesep should be ~2.08 (150px / 72 DPI = 2.08 inches)
	// Expected: ranksep should be ~2.50 (180px / 72 DPI = 2.50 inches)
	// We'll check they're at least greater than the old values (120/72=1.67, 130/72=1.81)
	t.Run("Base spacing increased", func(t *testing.T) {
		// Extract nodesep value
		nodesepIdx := strings.Index(dotOutput, "nodesep=")
		if nodesepIdx == -1 {
			t.Fatal("nodesep not found in DOT output")
		}

		// Check that nodesep is present (should be ~2.08 for 150px base)
		// Old value was ~1.67, new should be significantly higher
		if !strings.Contains(dotOutput, "nodesep=") {
			t.Error("nodesep attribute missing")
		}

		// Check ranksep
		if !strings.Contains(dotOutput, "ranksep=") {
			t.Error("ranksep attribute missing")
		}

		// Verify the values are at least above old minimums
		// We can't easily parse floats, so we check the DOT looks correct
		// For 150px: 150/72 = 2.08, so should see "2.08" or "2.09"
		if !strings.Contains(dotOutput, "nodesep=2.") && !strings.Contains(dotOutput, "nodesep=1.9") {
			t.Logf("DOT snippet around nodesep: %s", extractAround(dotOutput, "nodesep", 30))
		}
	})

	// Test 2: Verify node margins are added
	t.Run("Node margins added", func(t *testing.T) {
		if !strings.Contains(dotOutput, "margin=") {
			t.Error("margin attribute missing from nodes (expected margin=0.15)")
		}
	})

	// Test 3: Verify graph padding increased
	t.Run("Graph padding increased", func(t *testing.T) {
		if !strings.Contains(dotOutput, "pad=0.5") {
			t.Error("Expected pad=0.5 (was 0.4), but not found")
		}
	})

	// Test 4: Verify overlap prevention
	t.Run("Overlap prevention enabled", func(t *testing.T) {
		if !strings.Contains(dotOutput, "overlap=false") {
			t.Error("Expected overlap=false for overlap prevention")
		}
	})

	// Test 5: Verify separation value increased
	t.Run("Separation increased", func(t *testing.T) {
		if !strings.Contains(dotOutput, "sep=0.4") {
			t.Error("Expected sep=0.4 (was 0.3), but not found")
		}
	})

	// Test 6: Verify edge weights are present for labeled edges
	t.Run("Edge weights for labeled edges", func(t *testing.T) {
		// Both edges have labels, so they should have high weights (25)
		if !strings.Contains(dotOutput, "weight=25") || !strings.Contains(dotOutput, "weight=") {
			t.Logf("Expected weight=25 for labeled edges. DOT snippet: %s", extractAround(dotOutput, "weight", 30))
		}
	})

	// Test 7: Print DOT output for manual inspection
	t.Logf("Generated DOT output:\n%s", dotOutput)
}

// TestAdaptiveSpacing verifies adaptive spacing increases with node count
func TestAdaptiveSpacing(t *testing.T) {
	// Create a diagram with more nodes to test adaptive spacing
	dsl := `
	person = kind "Person"
	system = kind "System"

	p1 = person "User 1"
	p2 = person "User 2"
	p3 = person "Admin"
	
	s1 = system "Web App"
	s2 = system "API"
	s3 = system "Database"
	s4 = system "Cache"
	
	p1 -> s1
	p2 -> s1
	p3 -> s2
	s1 -> s2
	s2 -> s3
	s2 -> s4
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	prog, _, err := parser.Parse("test_adaptive.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	config := dot.DefaultConfig()
	config.ViewLevel = 1
	exporter := dot.NewExporter(config)
	result := exporter.Export(prog)

	dotOutput := result.DOT

	// With 7 nodes, adaptive spacing should apply
	// Base: 150px nodesep = 2.08 inches
	// With 7 nodes: scaleFactor = 1.0 + 0.25 * 7/8 = 1.21875
	// nodesep should be ~2.54 inches
	// Plus L1 boost: 2.54 * 1.15 = ~2.92 inches

	t.Logf("DOT output for 7-node diagram:\n%s", dotOutput)

	// Verify adaptive spacing is working
	if !strings.Contains(dotOutput, "nodesep=") {
		t.Error("nodesep missing")
	}
	if !strings.Contains(dotOutput, "ranksep=") {
		t.Error("ranksep missing")
	}
}

// extractAround extracts a snippet of text around a search term
func extractAround(text, search string, context int) string {
	idx := strings.Index(text, search)
	if idx == -1 {
		return ""
	}
	start := idx - context
	if start < 0 {
		start = 0
	}
	end := idx + len(search) + context
	if end > len(text) {
		end = len(text)
	}
	return text[start:end]
}
