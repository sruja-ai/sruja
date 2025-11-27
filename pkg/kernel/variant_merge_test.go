// pkg/kernel/variant_merge_test.go
// Tests for enhanced variant merge functionality

package kernel

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/model"
)

func TestComputeVariantDiff(t *testing.T) {
	sm := NewSnapshotManager(NewArchitectureStore())
	vm := NewVariantManager(sm, NewArchitectureStore())

	// Create base snapshot
	baseStore := NewArchitectureStore()
	baseModel := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1"},
			},
		},
	}
	baseStore.UpdateModel(baseModel)
	sm.store = baseStore
	_, err := sm.CreateSnapshot("base1", "Base snapshot")
	if err != nil {
		t.Fatalf("Failed to create snapshot: %v", err)
	}

	// Create variant
	_, err = vm.CreateVariant("test-variant", "base1", "Test variant")
	if err != nil {
		t.Fatalf("Failed to create variant: %v", err)
	}

	// Modify variant
	variantStore, _ := vm.GetVariantStore("test-variant")
	variantModel := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1"},
				{ID: "elem2", Type: model.ElementTypeContainer, Name: "Container1"},
			},
		},
	}
	variantStore.UpdateModel(variantModel)

	// Compute diff
	patches, err := vm.ComputeVariantDiff("test-variant")
	if err != nil {
		t.Fatalf("Failed to compute diff: %v", err)
	}

	// Should have one add patch for elem2
	addPatches := 0
	for _, patch := range patches {
		if patch.Operation == "add" && patch.ElementID == "elem2" {
			addPatches++
		}
	}

	if addPatches != 1 {
		t.Errorf("Expected 1 add patch for elem2, got %d", addPatches)
	}
}

func TestMergeVariant_NoConflicts(t *testing.T) {
	sm := NewSnapshotManager(NewArchitectureStore())
	vm := NewVariantManager(sm, NewArchitectureStore())

	// Create base snapshot
	baseStore := NewArchitectureStore()
	baseModel := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1"},
			},
		},
	}
	baseStore.UpdateModel(baseModel)
	sm.store = baseStore
	_, err := sm.CreateSnapshot("base1", "Base snapshot")
	if err != nil {
		t.Fatalf("Failed to create snapshot: %v", err)
	}

	// Create variant and add element
	_, err = vm.CreateVariant("test-variant", "base1", "Test variant")
	if err != nil {
		t.Fatalf("Failed to create variant: %v", err)
	}

	variantStore, _ := vm.GetVariantStore("test-variant")
	variantModel := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1"},
				{ID: "elem2", Type: model.ElementTypeContainer, Name: "Container1"},
			},
		},
	}
	variantStore.UpdateModel(variantModel)

	// Current model is same as base (no changes)
	currentModel := baseModel.Clone()
	vm.baseStore.UpdateModel(currentModel)

	// Merge variant
	result, err := vm.MergeVariant("test-variant")
	if err != nil {
		t.Fatalf("Failed to merge variant: %v", err)
	}

	if !result.Success {
		t.Error("Expected merge to succeed, but it failed")
	}

	if len(result.Conflicts) > 0 {
		t.Errorf("Expected no conflicts, got %d", len(result.Conflicts))
	}

	// Check that merged model has both elements
	if result.MergedModel == nil || result.MergedModel.Architecture == nil {
		t.Fatal("Merged model is nil")
	}

	elemCount := len(result.MergedModel.Architecture.Elements)
	if elemCount != 2 {
		t.Errorf("Expected 2 elements in merged model, got %d", elemCount)
	}
}

func TestMergeVariant_WithConflicts(t *testing.T) {
	sm := NewSnapshotManager(NewArchitectureStore())
	vm := NewVariantManager(sm, NewArchitectureStore())

	// Create base snapshot
	baseStore := NewArchitectureStore()
	baseModel := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1", Description: "Base description"},
			},
		},
	}
	baseStore.UpdateModel(baseModel)
	sm.store = baseStore
	_, err := sm.CreateSnapshot("base1", "Base snapshot")
	if err != nil {
		t.Fatalf("Failed to create snapshot: %v", err)
	}

	// Create variant and modify element
	_, err = vm.CreateVariant("test-variant", "base1", "Test variant")
	if err != nil {
		t.Fatalf("Failed to create variant: %v", err)
	}

	variantStore, _ := vm.GetVariantStore("test-variant")
	variantModel := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1", Description: "Variant description"},
			},
		},
	}
	variantStore.UpdateModel(variantModel)

	// Current model also modified the same element
	currentModel := &model.Model{
		Architecture: &model.Architecture{
			Elements: []model.Element{
				{ID: "elem1", Type: model.ElementTypeSystem, Name: "System1", Description: "Current description"},
			},
		},
	}
	vm.baseStore.UpdateModel(currentModel)

	// Merge variant
	result, err := vm.MergeVariant("test-variant")
	if err != nil {
		t.Fatalf("Failed to merge variant: %v", err)
	}

	// Should have conflicts
	if len(result.Conflicts) == 0 {
		t.Error("Expected conflicts, but none were detected")
	}

	if result.Success {
		t.Error("Expected merge to fail due to conflicts, but it succeeded")
	}
}
