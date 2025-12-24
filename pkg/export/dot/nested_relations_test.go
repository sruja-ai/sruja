package dot_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/dot"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExporter_NestedRelations(t *testing.T) {
	// DSL with deeply nested relations
	// Note: Explicit source required for relations (parser doesn't support implicit 'this' yet)
	dsl := `
specification {
	element system
	element container
	element component
}

model {
	sys = system "System" {
		db = container "Database"
		
		backend = container "Backend" {
			// Relation defined inside container
			backend -> db "writes"
		
			api = component "API" {
				// Relation defined INSIDE deeply nested element
				api -> db "reads"
			}
			
			worker = component "Worker" {
				worker -> db "writes async"
			}
		}
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	prog, _, err := parser.Parse("nested.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	config := dot.DefaultConfig()
	config.ViewLevel = 3 // Deep view to see components
	exporter := dot.NewExporter(config)
	result := exporter.Export(prog)

	if result == nil || result.DOT == "" {
		t.Fatal("Expected non-empty DOT output")
	}

	dot := result.DOT

	// Print DOT for debugging if test fails
	t.Logf("Generated DOT:\n%s", dot)

	// Verify nodes
	if !strings.Contains(dot, "\"sys.backend.api\"") {
		t.Error("Missing api node")
	}

	// Verify edges - these were previously missing!

	// 1. Edge from API to DB (defined inside API)
	if !strings.Contains(dot, "\"sys.backend.api\" -> \"sys.db\"") {
		t.Error("Missing deep nested edge: api -> db")
	}

	// 2. Edge from Backend to DB (defined inside Backend)
	if !strings.Contains(dot, "\"sys.backend\" -> \"sys.db\"") {
		t.Error("Missing nested edge: backend -> db")
	}

	// 3. Edge from Worker to DB
	if !strings.Contains(dot, "\"sys.backend.worker\" -> \"sys.db\"") {
		t.Error("Missing nested edge: worker -> db")
	}
}
