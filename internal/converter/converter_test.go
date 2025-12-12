//nolint:gocritic // appendCombine acceptable
package converter

import (
	"encoding/json"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func stringPtr(s string) *string {
	return &s
}

func TestConvertToJSON(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test Arch",
		Systems: []*language.System{
			{
				ID:          "S1",
				Label:       "System 1",
				Description: stringPtr("Desc 1"),
				Items: []language.SystemItem{
					{
						Container: &language.Container{
							ID:    "C1",
							Label: "Container 1",
							Items: []language.ContainerItem{
								{Technology: stringPtr("Go")},
								{
									Component: &language.Component{
										ID:    "Comp1",
										Label: "Component 1",
									},
								},
							},
						},
					},
				},
			},
		},
		Persons: []*language.Person{
			{ID: "P1", Label: "Person 1"},
		},
		Relations: []*language.Relation{
			{
				From: language.QualifiedIdent{Parts: []string{"P1"}},
				To:   language.QualifiedIdent{Parts: []string{"S1"}},
				Verb: stringPtr("uses"),
			},
		},
		ADRs: []*language.ADR{
			{
				ID:    "ADR1",
				Title: stringPtr("Use Go"),
				Body: &language.ADRBody{
					Status: stringPtr("Accepted"),
				},
			},
		},
		Items: []language.ArchitectureItem{
			// Populate Items for processArchitecture
		},
	}

	// Populate Items manually as parser would
	arch.Items = append(arch.Items, language.ArchitectureItem{System: arch.Systems[0]})
	arch.Items = append(arch.Items, language.ArchitectureItem{Person: arch.Persons[0]})
	arch.Items = append(arch.Items, language.ArchitectureItem{Relation: arch.Relations[0]})
	arch.Items = append(arch.Items, language.ArchitectureItem{ADR: arch.ADRs[0]})

	result := ConvertToJSON(arch)

	if result.Metadata.Name != "Test Arch" {
		t.Errorf("Expected metadata name 'Test Arch', got '%s'", result.Metadata.Name)
	}

	if len(result.Architecture.Systems) != 1 {
		t.Errorf("Expected 1 system, got %d", len(result.Architecture.Systems))
	}
	if result.Architecture.Systems[0].ID != "S1" {
		t.Errorf("Expected system ID 'S1', got '%s'", result.Architecture.Systems[0].ID)
	}
	if len(result.Architecture.Systems[0].Containers) != 1 {
		t.Errorf("Expected 1 container, got %d", len(result.Architecture.Systems[0].Containers))
	}
	if result.Architecture.Systems[0].Containers[0].Technology != "Go" {
		t.Errorf("Expected container technology 'Go', got '%s'", result.Architecture.Systems[0].Containers[0].Technology)
	}

	if len(result.Architecture.Persons) != 1 {
		t.Errorf("Expected 1 person, got %d", len(result.Architecture.Persons))
	}

	if len(result.Architecture.Relations) != 1 {
		t.Errorf("Expected 1 relation, got %d", len(result.Architecture.Relations))
	}
	if result.Architecture.Relations[0].From != "P1" {
		t.Errorf("Expected relation from 'P1', got '%s'", result.Architecture.Relations[0].From)
	}

	if len(result.Architecture.ADRs) != 1 {
		t.Errorf("Expected 1 ADR, got %d", len(result.Architecture.ADRs))
	}
	if result.Architecture.ADRs[0].Status != "Accepted" {
		t.Errorf("Expected ADR status 'Accepted', got '%s'", result.Architecture.ADRs[0].Status)
	}
}

func TestConvertFromJSON(t *testing.T) {
	jsonStr := `{
		"metadata": {
			"name": "Test Arch",
			"version": "1.0.0",
			"generated": "2023-01-01T00:00:00Z"
		},
		"architecture": {
			"systems": [
				{
					"id": "S1",
					"label": "System 1",
					"description": "Desc 1",
					"containers": [
						{
							"id": "C1",
							"label": "Container 1",
							"technology": "Go",
							"components": [
								{
									"id": "Comp1",
									"label": "Component 1"
								}
							]
						}
					],
					"datastores": [
						{
							"id": "DB1",
							"label": "Database 1",
							"technology": "Postgres"
						}
					],
					"queues": [
						{
							"id": "Q1",
							"label": "Queue 1"
						}
					]
				}
			],
			"persons": [
				{
					"id": "P1",
					"label": "Person 1"
				}
			],
			"relations": [
				{
					"from": "P1",
					"to": "S1",
					"verb": "uses"
				}
			],
			"adrs": [
				{
					"id": "ADR1",
					"title": "Use Go",
					"status": "Accepted"
				}
			]
		}
	}`

	var archJSON ArchitectureJSON
	err := json.Unmarshal([]byte(jsonStr), &archJSON)
	if err != nil {
		t.Fatalf("Failed to unmarshal JSON: %v", err)
	}

	arch := ConvertFromJSON(archJSON)

	if arch.Name != "Test Arch" {
		t.Errorf("Expected arch name 'Test Arch', got '%s'", arch.Name)
	}

	if len(arch.Systems) != 1 {
		t.Errorf("Expected 1 system, got %d", len(arch.Systems))
	}
	sys := arch.Systems[0]
	if sys.ID != "S1" {
		t.Errorf("Expected system ID 'S1', got '%s'", sys.ID)
	}
	if *sys.Description != "Desc 1" {
		t.Errorf("Expected description 'Desc 1', got '%s'", *sys.Description)
	}

	// Check items populated in system
	foundContainer := false
	foundDB := false
	foundQueue := false
	for _, item := range sys.Items {
		if item.Container != nil {
			foundContainer = true
			if item.Container.ID != "C1" {
				t.Errorf("Expected container ID 'C1', got '%s'", item.Container.ID)
			}
			// Check nested component
			foundComp := false
			for _, cItem := range item.Container.Items {
				if cItem.Component != nil {
					foundComp = true
					if cItem.Component.ID != "Comp1" {
						t.Errorf("Expected component ID 'Comp1', got '%s'", cItem.Component.ID)
					}
				}
			}
			if !foundComp {
				t.Error("Expected component in container items")
			}
		}
		if item.DataStore != nil {
			foundDB = true
			if item.DataStore.ID != "DB1" {
				t.Errorf("Expected datastore ID 'DB1', got '%s'", item.DataStore.ID)
			}
		}
		if item.Queue != nil {
			foundQueue = true
			if item.Queue.ID != "Q1" {
				t.Errorf("Expected queue ID 'Q1', got '%s'", item.Queue.ID)
			}
		}
	}
	if !foundContainer {
		t.Error("Expected container in system items")
	}
	if !foundDB {
		t.Error("Expected datastore in system items")
	}
	if !foundQueue {
		t.Error("Expected queue in system items")
	}

	if len(arch.Persons) != 1 {
		t.Errorf("Expected 1 person, got %d", len(arch.Persons))
	}

	if len(arch.Relations) != 1 {
		t.Errorf("Expected 1 relation, got %d", len(arch.Relations))
	}
	if arch.Relations[0].From.String() != "P1" {
		t.Errorf("Expected relation from 'P1', got '%s'", arch.Relations[0].From.String())
	}

	if len(arch.ADRs) != 1 {
		t.Errorf("Expected 1 ADR, got %d", len(arch.ADRs))
	}
}

func TestMetadataConversion(t *testing.T) {
	meta := &language.MetadataBlock{
		Entries: []*language.MetaEntry{
			{
				Key:   "k1",
				Value: stringPtr("v1"),
			},
			{
				Key:   "tags",
				Array: []string{"t1", "t2"},
			},
		},
	}

	// Test To JSON
	jsonMeta := convertMetadataToJSON(meta)
	if len(jsonMeta) != 2 {
		t.Errorf("Expected 2 metadata entries, got %d", len(jsonMeta))
	}
	if jsonMeta[0].Key != "k1" || *jsonMeta[0].Value != "v1" {
		t.Errorf("Mismatch in simple metadata entry")
	}
	if jsonMeta[1].Key != "tags" || len(jsonMeta[1].Array) != 2 {
		t.Errorf("Mismatch in array metadata entry")
	}

	// Test From JSON
	backMeta := convertMetadataFromJSON(jsonMeta)
	if len(backMeta.Entries) != 2 {
		t.Errorf("Expected 2 metadata entries back, got %d", len(backMeta.Entries))
	}
	if backMeta.Entries[0].Key != "k1" {
		t.Errorf("Mismatch in simple metadata entry back")
	}
}
