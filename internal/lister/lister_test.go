package lister

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func parseDSL(t *testing.T, dsl string) *language.Program {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}
	return prog
}

func TestListSystems(t *testing.T) {
	tests := []struct {
		name string
		prog *language.Program
		want int
	}{
		{
			name: "nil program",
			prog: nil,
			want: 0,
		},
		{
			name: "empty program",
			prog: &language.Program{Model: &language.ModelBlock{}},
			want: 0,
		},
		{
			name: "with systems",
			prog: parseDSL(t, `model {
				S1 = system "System 1" {
					description "Desc 1"
				}
				S2 = system "System 2"
			}`),
			want: 2,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := ListSystems(tt.prog)
			if len(result) != tt.want {
				t.Errorf("ListSystems() = %d items, want %d", len(result), tt.want)
			}
			if tt.want > 0 && result[0].ID != "S1" {
				t.Errorf("Expected first system ID 'S1', got '%s'", result[0].ID)
			}
		})
	}
}

func TestListContainers(t *testing.T) {
	dsl := `model {
		S1 = system "System 1" {
			C1 = container "Container 1" {
				description "Test"
			}
			C2 = container "Container 2"
		}
	}`
	prog := parseDSL(t, dsl)

	result := ListContainers(prog)
	if len(result) != 2 {
		t.Errorf("Expected 2 containers, got %d", len(result))
	}
	if result[0].SystemID != "S1" {
		t.Errorf("Expected SystemID 'S1', got '%s'", result[0].SystemID)
	}
	if result[0].Description != "Test" {
		t.Errorf("Expected description 'Test', got '%s'", result[0].Description)
	}
}

func TestListComponents(t *testing.T) {
	dsl := `model {
		S1 = system "System 1" {
			Comp1 = component "Component 1"
			C1 = container "Container 1" {
				Comp2 = component "Component 2" {
					description "Nested"
				}
			}
		}
	}`
	prog := parseDSL(t, dsl)

	result := ListComponents(prog)
	if len(result) != 2 {
		t.Errorf("Expected 2  components, got %d", len(result))
	}

	// Check system-level component
	if result[0].SystemID != "S1" {
		t.Errorf("Expected SystemID 'S1', got '%s'", result[0].SystemID)
	}
	if result[0].ContainerID != "" {
		t.Errorf("System-level component should have empty ContainerID")
	}

	// Check container-level component
	if result[1].ContainerID != "S1.C1" {
		t.Errorf("Expected ContainerID 'S1.C1', got '%s'", result[1].ContainerID)
	}
	if result[1].Description != "Nested" {
		t.Errorf("Expected description 'Nested', got '%s'", result[1].Description)
	}
}

func TestListPersons(t *testing.T) {
	dsl := `model {
		User1 = person "End User"
		Admin1 = person "Administrator"
	}`
	prog := parseDSL(t, dsl)

	result := ListPersons(prog)
	if len(result) != 2 {
		t.Errorf("Expected 2 persons, got %d", len(result))
	}
	if result[0].ID != "User1" {
		t.Errorf("Expected ID 'User1', got '%s'", result[0].ID)
	}
}

func TestListDataStores(t *testing.T) {
	dsl := `model {
		S1 = system "System 1" {
			DB1 = database "Database 1"
		}
	}`
	prog := parseDSL(t, dsl)

	result := ListDataStores(prog)
	if len(result) != 1 {
		t.Errorf("Expected 1 datastore, got %d", len(result))
	}
	if result[0].SystemID != "S1" {
		t.Errorf("Expected SystemID 'S1', got '%s'", result[0].SystemID)
	}
}

func TestListQueues(t *testing.T) {
	dsl := `model {
		S1 = system "System 1" {
			Q1 = queue "Queue 1"
		}
	}`
	prog := parseDSL(t, dsl)

	result := ListQueues(prog)
	if len(result) != 1 {
		t.Errorf("Expected 1 queue, got %d", len(result))
	}
}

func TestListScenarios(t *testing.T) {
	dsl := `model {
		scenario S1 "User Login" "User logs into system"
	}`
	prog := parseDSL(t, dsl)

	result := ListScenarios(prog)
	if len(result) != 1 {
		t.Errorf("Expected 1 scenario, got %d", len(result))
	}
}

func TestListADRs(t *testing.T) {
	dsl := `model {
		adr ADR001 "Use JWT"
		adr ADR002 "ADR002"
	}`
	prog := parseDSL(t, dsl)

	result := ListADRs(prog)
	if len(result) != 2 {
		t.Errorf("Expected 2 ADRs, got %d", len(result))
	}
	if result[0].Title != "Use JWT" {
		t.Errorf("Expected title 'Use JWT', got '%s'", result[0].Title)
	}
	if result[1].Title != "ADR002" {
		t.Errorf("Expected title 'ADR002' for ADR002, got '%s'", result[1].Title)
	}
}

func TestNilProgramHandling(t *testing.T) {
	// Test that all functions handle nil gracefully
	if result := ListSystems(nil); result != nil {
		t.Error("ListSystems(nil) should return nil")
	}
	if result := ListContainers(nil); result != nil {
		t.Error("ListContainers(nil) should return nil")
	}
	if result := ListComponents(nil); result != nil {
		t.Error("ListComponents(nil) should return nil")
	}
	if result := ListPersons(nil); result != nil {
		t.Error("ListPersons(nil) should return nil")
	}
	if result := ListDataStores(nil); result != nil {
		t.Error("ListDataStores(nil) should return nil")
	}
	if result := ListQueues(nil); result != nil {
		t.Error("ListQueues(nil) should return nil")
	}
	if result := ListScenarios(nil); result != nil {
		t.Error("ListScenarios(nil) should return nil")
	}
	if result := ListADRs(nil); result != nil {
		t.Error("ListADRs(nil) should return nil")
	}
}
