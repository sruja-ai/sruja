package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestResolver(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	// Setup a program with ambiguous and unambiguous references
	dsl := `model {
		SysA = system "System A" {
			ContA = container "Container A" {
				UniqueComp = component "Unique Component"
				SharedComp = component "Shared Component"
			}
		}
		SysB = system "System B" {
			ContB = container "Container B" {
				UniqueCompB = component "Unique Component B"
				SharedComp = component "Shared Component"
			}
		}
		flow TestFlow {
			UniqueComp -> UniqueCompB
			SharedComp -> Unknown
		}
	}`

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	// Run Resolution
	RunResolution(program)

	// Verify - find the flow in Model.Items
	var flow *language.Flow
	for _, item := range program.Model.Items {
		if item.Flow != nil {
			flow = item.Flow
			break
		}
	}

	if flow == nil {
		t.Fatal("Expected to find TestFlow")
	}

	// Step 1: UniqueComp -> SysA.ContA.UniqueComp
	step1 := flow.Items[0].Step
	if step1.From.String() != "SysA.ContA.UniqueComp" {
		t.Errorf("Expected UniqueComp to resolve to SysA.ContA.UniqueComp, got %s", step1.From.String())
	}
	if step1.To.String() != "SysB.ContB.UniqueCompB" {
		t.Errorf("Expected UniqueCompB to resolve to SysB.ContB.UniqueCompB, got %s", step1.To.String())
	}

	// Step 2: SharedComp -> SharedComp (Ambiguous, unchanged), Unknown -> Unknown
	step2 := flow.Items[1].Step
	if step2.From.String() != "SharedComp" {
		t.Errorf("Expected SharedComp to remain 'SharedComp' (ambiguous), got %s", step2.From.String())
	}
	if step2.To.String() != "Unknown" {
		t.Errorf("Expected Unknown to remain 'Unknown', got %s", step2.To.String())
	}
}
