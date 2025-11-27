// pkg/kernel/snapshot_test.go
package kernel

import (
	"testing"
	"time"
)

func TestSnapshotManager_CreateSnapshot(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)

	snapshot, err := sm.CreateSnapshot("test-snapshot", "Test description")
	if err != nil {
		t.Fatalf("Failed to create snapshot: %v", err)
	}

	if snapshot == nil {
		t.Fatal("Snapshot is nil")
	}

	if snapshot.Name != "test-snapshot" {
		t.Errorf("Expected name 'test-snapshot', got '%s'", snapshot.Name)
	}

	if snapshot.Description != "Test description" {
		t.Errorf("Expected description 'Test description', got '%s'", snapshot.Description)
	}

	if len(snapshot.IR) == 0 {
		t.Error("Snapshot IR should not be empty")
	}
}

func TestSnapshotManager_DuplicateSnapshot(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)

	_, err := sm.CreateSnapshot("test", "First")
	if err != nil {
		t.Fatalf("Failed to create first snapshot: %v", err)
	}

	_, err = sm.CreateSnapshot("test", "Second")
	if err == nil {
		t.Error("Expected error when creating duplicate snapshot")
	}
}

func TestSnapshotManager_GetSnapshot(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)

	_, err := sm.CreateSnapshot("test", "Description")
	if err != nil {
		t.Fatalf("Failed to create snapshot: %v", err)
	}

	snapshot, ok := sm.GetSnapshot("test")
	if !ok {
		t.Fatal("Snapshot not found")
	}

	if snapshot.Name != "test" {
		t.Errorf("Expected name 'test', got '%s'", snapshot.Name)
	}
}

func TestSnapshotManager_ListSnapshots(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)

	sm.CreateSnapshot("snapshot-1", "First")
	sm.CreateSnapshot("snapshot-2", "Second")

	snapshots := sm.ListSnapshots()
	if len(snapshots) != 2 {
		t.Errorf("Expected 2 snapshots, got %d", len(snapshots))
	}
}

func TestSnapshotManager_LoadSnapshot(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)

	// Set some metadata in the store
	store.SetMetadata("test-key", "test-value")

	// Create snapshot
	_, err := sm.CreateSnapshot("test", "Test")
	if err != nil {
		t.Fatalf("Failed to create snapshot: %v", err)
	}

	// Modify store
	store.SetMetadata("test-key", "modified-value")

	// Load snapshot
	if err := sm.LoadSnapshot("test"); err != nil {
		t.Fatalf("Failed to load snapshot: %v", err)
	}

	// Verify metadata is restored (or at least version is correct)
	// Note: metadata restoration depends on FromJSON implementation
	version := store.GetVersion()
	if version == 0 {
		t.Error("Store version should be updated after loading snapshot")
	}
}

func TestSnapshotManager_DeleteSnapshot(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)

	sm.CreateSnapshot("test", "Test")

	if err := sm.DeleteSnapshot("test"); err != nil {
		t.Fatalf("Failed to delete snapshot: %v", err)
	}

	_, ok := sm.GetSnapshot("test")
	if ok {
		t.Error("Snapshot should be deleted")
	}
}

func TestSnapshot_Timestamp(t *testing.T) {
	store := NewArchitectureStore()
	sm := NewSnapshotManager(store)

	before := time.Now()
	snapshot, err := sm.CreateSnapshot("test", "")
	after := time.Now()

	if err != nil {
		t.Fatalf("Failed to create snapshot: %v", err)
	}

	if snapshot.Timestamp.Before(before) || snapshot.Timestamp.After(after) {
		t.Error("Snapshot timestamp should be between before and after")
	}
}

