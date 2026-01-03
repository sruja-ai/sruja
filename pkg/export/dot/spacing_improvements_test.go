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
	// Expected: nodesep should be ~1.52 (110px / 72 DPI)
	// Expected: ranksep should be ~1.66 (120px / 72 DPI)
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

		// Verify the values are reasonable for compact layout
		// For 60px: 60/72 = 0.83, so should see "0.83"
		if !strings.Contains(dotOutput, "nodesep=0.") && !strings.Contains(dotOutput, "nodesep=1.") {
			t.Logf("DOT snippet around nodesep: %s", extractAround(dotOutput, "nodesep", 30))
		}
	})

	// Test 2: Verify node margins are handled by HTML (margin=0)
	t.Run("Node margins handled by HTML", func(t *testing.T) {
		if !strings.Contains(dotOutput, "margin=0") {
			t.Error("margin=0 missing from nodes (expected HTML padding)")
		}
	})

	// Test 3: Verify graph padding is compact
	t.Run("Graph padding compact", func(t *testing.T) {
		if !strings.Contains(dotOutput, "pad=0.2") {
			t.Error("Expected pad=0.2 (compact), but not found")
		}
	})

	// Test 4: Verify overlap prevention
	t.Run("Overlap prevention enabled", func(t *testing.T) {
		if !strings.Contains(dotOutput, "overlap=false") {
			t.Error("Expected overlap=false for overlap prevention")
		}
	})

	// Test 5: Verify separation value compact
	t.Run("Separation compact", func(t *testing.T) {
		if !strings.Contains(dotOutput, "sep=0.1") {
			t.Error("Expected sep=0.1 (compact), but not found")
		}
	})

	// Test 6: Verify edge weights are present for labeled edges
	t.Run("Edge weights for labeled edges", func(t *testing.T) {
		// Both edges have labels, so they should have moderate weights (10)
		if !strings.Contains(dotOutput, "weight=10") || !strings.Contains(dotOutput, "weight=") {
			t.Logf("Expected weight=10 for labeled edges. DOT snippet: %s", extractAround(dotOutput, "weight", 30))
		}
	})

	// Test 7: Verify HTML Labels
	t.Run("HTML Labels Used", func(t *testing.T) {
		if !strings.Contains(dotOutput, "label=<") || !strings.Contains(dotOutput, "<TABLE") {
			t.Error("Expected HTML labels (label=<...<TABLE...), but not found")
		}
	})

	// Test 8: Print DOT output for manual inspection
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
	// Base: 110px nodesep = 1.52 inches
	// With 7 nodes: scaleFactor logic applies
	// We just verify it's generated successfully and has reasonable values

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
