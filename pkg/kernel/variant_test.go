// pkg/kernel/variant_test.go
package kernel

import (
	"testing"
)

func TestVariantManager_CreateVariant(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)
	vm := NewVariantManager(sm, store)

	// Create a base snapshot first
	_, err := sm.CreateSnapshot("base", "Base snapshot")
	if err != nil {
		t.Fatalf("Failed to create base snapshot: %v", err)
	}

	// Create variant
	variant, err := vm.CreateVariant("test-variant", "base", "Test variant")
	if err != nil {
		t.Fatalf("Failed to create variant: %v", err)
	}

	if variant == nil {
		t.Fatal("Variant is nil")
	}

	if variant.Name != "test-variant" {
		t.Errorf("Expected name 'test-variant', got '%s'", variant.Name)
	}

	if variant.Base != "base" {
		t.Errorf("Expected base 'base', got '%s'", variant.Base)
	}
}

func TestVariantManager_CreateVariant_InvalidBase(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)
	vm := NewVariantManager(sm, store)

	_, err := vm.CreateVariant("test", "nonexistent", "Test")
	if err == nil {
		t.Error("Expected error when base snapshot doesn't exist")
	}
}

func TestVariantManager_GetVariant(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)
	vm := NewVariantManager(sm, store)

	sm.CreateSnapshot("base", "Base")
	vm.CreateVariant("test", "base", "Test")

	variant, ok := vm.GetVariant("test")
	if !ok {
		t.Fatal("Variant not found")
	}

	if variant.Name != "test" {
		t.Errorf("Expected name 'test', got '%s'", variant.Name)
	}
}

func TestVariantManager_ListVariants(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)
	vm := NewVariantManager(sm, store)

	sm.CreateSnapshot("base", "Base")
	vm.CreateVariant("variant-1", "base", "First")
	vm.CreateVariant("variant-2", "base", "Second")

	variants := vm.ListVariants()
	if len(variants) != 2 {
		t.Errorf("Expected 2 variants, got %d", len(variants))
	}
}

func TestVariantManager_ApplyVariant(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)
	vm := NewVariantManager(sm, store)

	// Create base snapshot
	sm.CreateSnapshot("base", "Base")

	// Create variant
	vm.CreateVariant("test", "base", "Test")

	// Apply variant
	if err := vm.ApplyVariant("test"); err != nil {
		t.Fatalf("Failed to apply variant: %v", err)
	}

	// Verify store was updated (by checking version)
	version := store.GetVersion()
	if version == 0 {
		t.Error("Store version should be updated after applying variant")
	}
}

func TestVariantManager_DeleteVariant(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)
	vm := NewVariantManager(sm, store)

	sm.CreateSnapshot("base", "Base")
	vm.CreateVariant("test", "base", "Test")

	if err := vm.DeleteVariant("test"); err != nil {
		t.Fatalf("Failed to delete variant: %v", err)
	}

	_, ok := vm.GetVariant("test")
	if ok {
		t.Error("Variant should be deleted")
	}
}

func TestKernel_SnapshotOperations(t *testing.T) {
	k, err := NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	// Create snapshot
	snapshot, err := k.CreateSnapshot("test", "Test snapshot")
	if err != nil {
		t.Fatalf("Failed to create snapshot: %v", err)
	}

	if snapshot == nil {
		t.Fatal("Snapshot is nil")
	}

	// Get snapshot
	got, ok := k.GetSnapshot("test")
	if !ok {
		t.Fatal("Snapshot not found")
	}

	if got.Name != "test" {
		t.Errorf("Expected name 'test', got '%s'", got.Name)
	}

	// List snapshots
	snapshots := k.ListSnapshots()
	if len(snapshots) != 1 {
		t.Errorf("Expected 1 snapshot, got %d", len(snapshots))
	}

	// Delete snapshot
	if err := k.DeleteSnapshot("test"); err != nil {
		t.Fatalf("Failed to delete snapshot: %v", err)
	}
}

func TestKernel_VariantOperations(t *testing.T) {
	k, err := NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	// Create base snapshot
	k.CreateSnapshot("base", "Base snapshot")

	// Create variant
	variant, err := k.CreateVariant("test-variant", "base", "Test variant")
	if err != nil {
		t.Fatalf("Failed to create variant: %v", err)
	}

	if variant == nil {
		t.Fatal("Variant is nil")
	}

	// Get variant
	got, ok := k.GetVariant("test-variant")
	if !ok {
		t.Fatal("Variant not found")
	}

	if got.Name != "test-variant" {
		t.Errorf("Expected name 'test-variant', got '%s'", got.Name)
	}

	// List variants
	variants := k.ListVariants()
	if len(variants) != 1 {
		t.Errorf("Expected 1 variant, got %d", len(variants))
	}
}

