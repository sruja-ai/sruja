package json

import (
	"testing"
)

func stringPtr(s string) *string {
	return &s
}

func TestToArchitecture(t *testing.T) {
	doc := ArchitectureJSON{
		Metadata: MetadataJSON{
			Name: "Test System",
		},
		Architecture: ArchitectureBody{
			Systems: []SystemJSON{
				{
					ID:    "S1",
					Label: "System 1",
					Containers: []ContainerJSON{
						{
							ID:    "C1",
							Label: "Container 1",
						},
					},
				},
			},
			Persons: []PersonJSON{
				{
					ID:    "P1",
					Label: "Person 1",
				},
			},
			Relations: []RelationJSON{
				{
					From: "P1",
					To:   "S1",
					Verb: stringPtr("uses"),
				},
			},
		},
	}

	arch := ToArchitecture(&doc)

	if arch.Name != "Test System" {
		t.Errorf("Expected name 'Test System', got '%s'", arch.Name)
	}

	if len(arch.Systems) != 1 {
		t.Errorf("Expected 1 system, got %d", len(arch.Systems))
	}
	if arch.Systems[0].ID != "S1" {
		t.Errorf("Expected system ID 'S1', got '%s'", arch.Systems[0].ID)
	}

	if len(arch.Persons) != 1 {
		t.Errorf("Expected 1 person, got %d", len(arch.Persons))
	}

	if len(arch.Relations) != 1 {
		t.Errorf("Expected 1 relation, got %d", len(arch.Relations))
	}
}
