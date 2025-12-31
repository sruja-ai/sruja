package engine

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestResolver(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	// Setup a program with ambiguous and unambiguous references
	dsl := `	SysA = System "System A" {
		ContA = Container "Container A" {
			UniqueComp = Component "Unique Component"
			SharedComp = Component "Shared Component"
		}
	}
	SysB = System "System B" {
		ContB = Container "Container B" {
			UniqueCompB = Component "Unique Component B"
			SharedComp = Component "Shared Component"
		}
	}
	TestFlow = Flow "Test Flow" {
		UniqueComp -> UniqueCompB
		SharedComp -> Unknown
	}`

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	// Run Resolution
	RunResolution(program)

	// Verify - find the flow in Model.Items (Now it's an ElementDef with Kind "Flow" or "flow")
	var flowElem *language.ElementDef
	for _, item := range program.Model.Items {
		if item.ElementDef != nil && strings.EqualFold(item.ElementDef.GetKind(), "flow") {
			flowElem = item.ElementDef
			break
		}
	}

	if flowElem == nil {
		t.Fatal("Expected to find TestFlow")
	}

	// Get the flow body items for stepping through
	flowBody := flowElem.GetBody()
	if flowBody == nil || len(flowBody.Items) == 0 {
		t.Fatal("Expected flow to have body items (steps)")
	}

	// Step 1: UniqueComp -> SysA.ContA.UniqueComp
	// flowBody.Items contain Steps as BodyItem with Step field
	if len(flowBody.Items) < 2 {
		t.Fatalf("Expected at least 2 step items, got %d", len(flowBody.Items))
	}
	step1 := flowBody.Items[0].Step
	if step1 == nil {
		// It might be a Relation instead
		rel1 := flowBody.Items[0].Relation
		if rel1 == nil {
			t.Fatal("Expected step or relation at index 0")
		}
		if rel1.From.String() != "SysA.ContA.UniqueComp" {
			t.Errorf("Expected UniqueComp to resolve to SysA.ContA.UniqueComp, got %s", rel1.From.String())
		}
		if rel1.To.String() != "SysB.ContB.UniqueCompB" {
			t.Errorf("Expected UniqueCompB to resolve to SysB.ContB.UniqueCompB, got %s", rel1.To.String())
		}
	} else {
		if step1.From.String() != "SysA.ContA.UniqueComp" {
			t.Errorf("Expected UniqueComp to resolve to SysA.ContA.UniqueComp, got %s", step1.From.String())
		}
		if step1.To.String() != "SysB.ContB.UniqueCompB" {
			t.Errorf("Expected UniqueCompB to resolve to SysB.ContB.UniqueCompB, got %s", step1.To.String())
		}
	}

	// Step 2: SharedComp -> SharedComp (Ambiguous, unchanged), Unknown -> Unknown
	step2 := flowBody.Items[1].Step
	if step2 == nil {
		rel2 := flowBody.Items[1].Relation
		if rel2 == nil {
			t.Fatal("Expected step or relation at index 1")
		}
		if rel2.From.String() != "SharedComp" {
			t.Errorf("Expected SharedComp to remain 'SharedComp' (ambiguous), got %s", rel2.From.String())
		}
		if rel2.To.String() != "Unknown" {
			t.Errorf("Expected Unknown to remain 'Unknown', got %s", rel2.To.String())
		}
	} else {
		if step2.From.String() != "SharedComp" {
			t.Errorf("Expected SharedComp to remain 'SharedComp' (ambiguous), got %s", step2.From.String())
		}
		if step2.To.String() != "Unknown" {
			t.Errorf("Expected Unknown to remain 'Unknown', got %s", step2.To.String())
		}
	}
}
