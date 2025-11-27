// pkg/kernel/diff_test.go
// Tests for diff engine

package kernel

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/model"
)

func TestDiffModels_AddedElements(t *testing.T) {
	engine := NewDiffEngine()

	base := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1"},
			},
		},
	}

	variant := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1"},
				{ID: "elem2", Type: model.ElementTypeContainer, Name: "Container1"},
			},
		},
	}

	diff := engine.DiffModels(base, variant)

	if len(diff.AddedElements) != 1 {
		t.Errorf("Expected 1 added element, got %d", len(diff.AddedElements))
	}

	if diff.AddedElements[0].ID != "elem2" {
		t.Errorf("Expected added element ID 'elem2', got '%s'", diff.AddedElements[0].ID)
	}
}

func TestDiffModels_RemovedElements(t *testing.T) {
	engine := NewDiffEngine()

	base := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1"},
				{ID: "elem2", Type: model.ElementTypeContainer, Name: "Container1"},
			},
		},
	}

	variant := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1"},
			},
		},
	}

	diff := engine.DiffModels(base, variant)

	if len(diff.RemovedElements) != 1 {
		t.Errorf("Expected 1 removed element, got %d", len(diff.RemovedElements))
	}

	if diff.RemovedElements[0].ID != "elem2" {
		t.Errorf("Expected removed element ID 'elem2', got '%s'", diff.RemovedElements[0].ID)
	}
}

func TestDiffModels_ModifiedElements(t *testing.T) {
	engine := NewDiffEngine()

	base := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1", Description: "Old description"},
			},
		},
	}

	variant := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1", Description: "New description"},
			},
		},
	}

	diff := engine.DiffModels(base, variant)

	if len(diff.ModifiedElements) != 1 {
		t.Errorf("Expected 1 modified element, got %d", len(diff.ModifiedElements))
	}

	if diff.ModifiedElements[0].ElementID != "elem1" {
		t.Errorf("Expected modified element ID 'elem1', got '%s'", diff.ModifiedElements[0].ElementID)
	}

	if diff.ModifiedElements[0].Changes["description"].New != "New description" {
		t.Errorf("Expected description change to 'New description', got '%v'", diff.ModifiedElements[0].Changes["description"].New)
	}
}

func TestDetectConflicts(t *testing.T) {
	engine := NewDiffEngine()

	base := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1", Description: "Base description"},
			},
		},
	}

	variant := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1", Description: "Variant description"},
			},
		},
	}

	current := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1", Description: "Current description"},
			},
		},
	}

	conflicts := engine.DetectConflicts(base, variant, current)

	if len(conflicts) == 0 {
		t.Error("Expected conflicts to be detected, but none were found")
	}

	if conflicts[0].Type != "element" {
		t.Errorf("Expected conflict type 'element', got '%s'", conflicts[0].Type)
	}

	if conflicts[0].ElementID != "elem1" {
		t.Errorf("Expected conflict element ID 'elem1', got '%s'", conflicts[0].ElementID)
	}
}

func TestDiffModels_Relations(t *testing.T) {
	engine := NewDiffEngine()

	base := &model.Model{
		Architecture: &model.Architecture{
			Relations: []model.Relation{
				{From: "elem1", To: "elem2", Type: model.RelationTypeUses},
			},
		},
	}

	variant := &model.Model{
		Architecture: &model.Architecture{
			Relations: []model.Relation{
				{From: "elem1", To: "elem2", Type: model.RelationTypeUses},
				{From: "elem2", To: "elem3", Type: model.RelationTypeDepends},
			},
		},
	}

	diff := engine.DiffModels(base, variant)

	if len(diff.AddedRelations) != 1 {
		t.Errorf("Expected 1 added relation, got %d", len(diff.AddedRelations))
	}
}
