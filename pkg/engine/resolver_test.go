package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestResolver(t *testing.T) {
	// Setup a program with ambiguous and unambiguous references
	arch := &language.Architecture{
		Systems: []*language.System{
			{
				ID: "SysA",
				Containers: []*language.Container{
					{
						ID: "ContA",
						Components: []*language.Component{
							{ID: "UniqueComp"},
							{ID: "SharedComp"},
						},
					},
				},
			},
			{
				ID: "SysB",
				Containers: []*language.Container{
					{
						ID: "ContB",
						Components: []*language.Component{
							{ID: "UniqueCompB"}, // Correct unique suffix
							{ID: "SharedComp"},  // Ambiguous with SysA.ContA.SharedComp
						},
					},
				},
			},
		},
		Flows: []*language.Flow{
			{
				ID: "TestFlow",
				Items: []*language.ScenarioItem{
					{
						Step: &language.ScenarioStep{
							From: language.QualifiedIdent{Parts: []string{"UniqueComp"}},
							To:   language.QualifiedIdent{Parts: []string{"UniqueCompB"}},
						},
					},
					{
						Step: &language.ScenarioStep{
							From: language.QualifiedIdent{Parts: []string{"SharedComp"}}, // Ambiguous
							To:   language.QualifiedIdent{Parts: []string{"Unknown"}},    // Unknown
						},
					},
				},
			},
		},
	}
	program := &language.Program{Architecture: arch}

	// Run Resolution
	RunResolution(program)

	// Verify
	flow := program.Architecture.Flows[0]

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
