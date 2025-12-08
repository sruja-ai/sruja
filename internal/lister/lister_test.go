package lister

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func stringPtr(s string) *string {
	return &s
}

func TestListSystems(t *testing.T) {
	tests := []struct {
		name string
		arch *language.Architecture
		want int
	}{
		{
			name: "nil architecture",
			arch: nil,
			want: 0,
		},
		{
			name: "empty architecture",
			arch: &language.Architecture{},
			want: 0,
		},
		{
			name: "with systems",
			arch: &language.Architecture{
				Systems: []*language.System{
					{ID: "S1", Label: "System 1", Description: stringPtr("Desc 1")},
					{ID: "S2", Label: "System 2"},
				},
			},
			want: 2,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := ListSystems(tt.arch)
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
	arch := &language.Architecture{
		Systems: []*language.System{
			{
				ID:    "S1",
				Label: "System 1",
				Containers: []*language.Container{
					{ID: "C1", Label: "Container 1", Description: stringPtr("Test")},
					{ID: "C2", Label: "Container 2"},
				},
			},
		},
	}

	result := ListContainers(arch)
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
	arch := &language.Architecture{
		Systems: []*language.System{
			{
				ID:    "S1",
				Label: "System 1",
				Components: []*language.Component{
					{ID: "Comp1", Label: "Component 1"},
				},
				Containers: []*language.Container{
					{
						ID:    "C1",
						Label: "Container 1",
						Components: []*language.Component{
							{ID: "Comp2", Label: "Component 2", Description: stringPtr("Nested")},
						},
					},
				},
			},
		},
	}

	result := ListComponents(arch)
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
	if result[1].ContainerID != "C1" {
		t.Errorf("Expected ContainerID 'C1', got '%s'", result[1].ContainerID)
	}
	if result[1].Description != "Nested" {
		t.Errorf("Expected description 'Nested', got '%s'", result[1].Description)
	}
}

func TestListPersons(t *testing.T) {
	arch := &language.Architecture{
		Persons: []*language.Person{
			{ID: "User1", Label: "End User"},
			{ID: "Admin1", Label: "Administrator"},
		},
	}

	result := ListPersons(arch)
	if len(result) != 2 {
		t.Errorf("Expected 2 persons, got %d", len(result))
	}
	if result[0].ID != "User1" {
		t.Errorf("Expected ID 'User1', got '%s'", result[0].ID)
	}
}

func TestListDataStores(t *testing.T) {
	arch := &language.Architecture{
		Systems: []*language.System{
			{
				ID:    "S1",
				Label: "System 1",
				DataStores: []*language.DataStore{
					{ID: "DB1", Label: "Database 1"},
				},
			},
		},
	}

	result := ListDataStores(arch)
	if len(result) != 1 {
		t.Errorf("Expected 1 datastore, got %d", len(result))
	}
	if result[0].SystemID != "S1" {
		t.Errorf("Expected SystemID 'S1', got '%s'", result[0].SystemID)
	}
}

func TestListQueues(t *testing.T) {
	arch := &language.Architecture{
		Systems: []*language.System{
			{
				ID:    "S1",
				Label: "System 1",
				Queues: []*language.Queue{
					{ID: "Q1", Label: "Queue 1"},
				},
			},
		},
	}

	result := ListQueues(arch)
	if len(result) != 1 {
		t.Errorf("Expected 1 queue, got %d", len(result))
	}
}

func TestListScenarios(t *testing.T) {
	arch := &language.Architecture{
		Scenarios: []*language.Scenario{
			{ID: "S1", Title: "User Login"},
		},
	}

	result := ListScenarios(arch)
	if len(result) != 1 {
		t.Errorf("Expected 1 scenario, got %d", len(result))
	}
}

func TestListADRs(t *testing.T) {
	arch := &language.Architecture{
		ADRs: []*language.ADR{
			{ID: "ADR001", Title: stringPtr("Use JWT")},
			{ID: "ADR002"},
		},
	}

	result := ListADRs(arch)
	if len(result) != 2 {
		t.Errorf("Expected 2 ADRs, got %d", len(result))
	}
	if result[0].Title != "Use JWT" {
		t.Errorf("Expected title 'Use JWT', got '%s'", result[0].Title)
	}
	if result[1].Title != "" {
		t.Errorf("Expected empty title for ADR002, got '%s'", result[1].Title)
	}
}

func TestNilArchitectureHandling(t *testing.T) {
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
