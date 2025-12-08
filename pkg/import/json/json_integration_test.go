package json_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/import/json"
)

func TestJSONImport_Comprehensive(t *testing.T) {
	// Complex JSON input covering all elements
	input := `{
		"metadata": {
			"name": "Complex System"
		},
		"architecture": {
			"systems": [
				{
					"id": "Sys1",
					"label": "System 1",
					"description": "A complex system",
					"containers": [
						{
							"id": "Cont1",
							"label": "Container 1",
							"description": "A container",
							"technology": "Go",
							"components": [
								{
									"id": "Comp1",
									"label": "Component 1",
									"description": "A component",
									"technology": "Struct"
								}
							]
						}
					]
				}
			],
			"policies": [
				{
					"id": "Pol1",
					"label": "Policy 1"
				}
			],
			"flows": [
				{
					"id": "Flow1",
					"title": "Login Flow",
					"steps": [
						{
							"id": "Step1",
							"description": "User enters credentials",
							"order": 1
						}
					]
				}
			],
			"scenarios": [
				{
					"id": "Scen1",
					"title": "Scenario 1",
					"description": "A scenario"
				}
			]
		}
	}`

	converter := json.NewConverter()
	arch, err := converter.ToArchitecture([]byte(input))
	if err != nil {
		t.Fatalf("ToArchitecture failed: %v", err)
	}

	// Verify System
	if len(arch.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(arch.Systems))
	}
	sys := arch.Systems[0]
	if sys.ID != "Sys1" {
		t.Errorf("Expected system ID Sys1, got %s", sys.ID)
	}

	// Verify Container
	if len(sys.Containers) != 1 {
		t.Fatalf("Expected 1 container, got %d", len(sys.Containers))
	}
	cont := sys.Containers[0]
	if cont.ID != "Cont1" {
		t.Errorf("Expected container ID Cont1, got %s", cont.ID)
	}

	// Verify Component
	if len(cont.Components) != 1 {
		t.Fatalf("Expected 1 component, got %d", len(cont.Components))
	}
	comp := cont.Components[0]
	if comp.ID != "Comp1" {
		t.Errorf("Expected component ID Comp1, got %s", comp.ID)
	}

	// Verify Policy
	if len(arch.Policies) != 1 {
		t.Fatalf("Expected 1 policy, got %d", len(arch.Policies))
	}
	pol := arch.Policies[0]
	if pol.ID != "Pol1" {
		t.Errorf("Expected policy ID Pol1, got %s", pol.ID)
	}

	// Verify Flow
	if len(arch.Flows) != 1 {
		t.Fatalf("Expected 1 flow, got %d", len(arch.Flows))
	}
	flow := arch.Flows[0]
	if flow.ID != "Flow1" {
		t.Errorf("Expected flow ID Flow1, got %s", flow.ID)
	}

	// Verify Scenario
	if len(arch.Scenarios) != 1 {
		t.Fatalf("Expected 1 scenario, got %d", len(arch.Scenarios))
	}
	scen := arch.Scenarios[0]
	if scen.ID != "Scen1" {
		t.Errorf("Expected scenario ID Scen1, got %s", scen.ID)
	}
}

func TestExtractSpecifics(t *testing.T) {
	input := `{
		"metadata": {
			"name": "Test System"
		},
		"architecture": {
			"scenarios": [{"id": "S1", "label": "T1"}],
			"requirements": [{"id": "R1", "title": "Req1"}],
			"adrs": [{"id": "ADR1", "title": "Dec1"}]
		}
	}`

	converter := json.NewConverter()

	// Test ExtractScenariosOnly (was 0% coverage)
	scens, err := converter.ToArchitecture([]byte(input))
	if err != nil {
		t.Fatalf("ToArchitecture failed: %v", err)
	}
	if len(scens.Scenarios) != 1 {
		t.Errorf("Expected 1 scenario, got %d", len(scens.Scenarios))
	}
}
