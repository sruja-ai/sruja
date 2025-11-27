// pkg/kernel/snapshot.go
// Package kernel provides snapshot and variant management for architecture state.
package kernel

import (
	"encoding/json"
	"fmt"
	"sync"
	"time"
)

// Snapshot represents a saved state of the architecture model.
type Snapshot struct {
	ID          string          `json:"id"`
	Name        string          `json:"name"`
	Description string          `json:"description,omitempty"`
	Timestamp   time.Time       `json:"timestamp"`
	IR          json.RawMessage `json:"ir"` // ArchitectureStore as JSON
	Version     int64           `json:"version"` // Store version when snapshot was taken
}

// SnapshotManager manages architecture snapshots.
type SnapshotManager struct {
	mu         sync.RWMutex
	snapshots  map[string]*Snapshot
	store      *ArchitectureStore
}

// NewSnapshotManager creates a new snapshot manager.
func NewSnapshotManager(store *ArchitectureStore) *SnapshotManager {
	return &SnapshotManager{
		snapshots: make(map[string]*Snapshot),
		store:     store,
	}
}

// CreateSnapshot creates a snapshot of the current architecture state.
func (sm *SnapshotManager) CreateSnapshot(name, description string) (*Snapshot, error) {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	// Check if snapshot with this name already exists
	if _, exists := sm.snapshots[name]; exists {
		return nil, fmt.Errorf("snapshot '%s' already exists", name)
	}

	// Export current IR
	irJSON, err := sm.store.ToJSON()
	if err != nil {
		return nil, fmt.Errorf("failed to export IR: %w", err)
	}

	version := sm.store.GetVersion()

	snapshot := &Snapshot{
		ID:          fmt.Sprintf("snapshot-%d", time.Now().Unix()),
		Name:        name,
		Description: description,
		Timestamp:   time.Now(),
		IR:          json.RawMessage(irJSON),
		Version:     version,
	}

	sm.snapshots[name] = snapshot

	return snapshot, nil
}

// GetSnapshot retrieves a snapshot by name.
func (sm *SnapshotManager) GetSnapshot(name string) (*Snapshot, bool) {
	sm.mu.RLock()
	defer sm.mu.RUnlock()

	snapshot, ok := sm.snapshots[name]
	return snapshot, ok
}

// ListSnapshots returns all snapshots.
func (sm *SnapshotManager) ListSnapshots() []*Snapshot {
	sm.mu.RLock()
	defer sm.mu.RUnlock()

	snapshots := make([]*Snapshot, 0, len(sm.snapshots))
	for _, snapshot := range sm.snapshots {
		snapshots = append(snapshots, snapshot)
	}
	return snapshots
}

// LoadSnapshot loads a snapshot into the architecture store.
func (sm *SnapshotManager) LoadSnapshot(name string) error {
	sm.mu.RLock()
	snapshot, ok := sm.snapshots[name]
	sm.mu.RUnlock()

	if !ok {
		return fmt.Errorf("snapshot '%s' not found", name)
	}

	// Load IR into store
	if err := sm.store.FromJSON(snapshot.IR); err != nil {
		return fmt.Errorf("failed to load snapshot IR: %w", err)
	}

	return nil
}

// DeleteSnapshot removes a snapshot.
func (sm *SnapshotManager) DeleteSnapshot(name string) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	if _, exists := sm.snapshots[name]; !exists {
		return fmt.Errorf("snapshot '%s' not found", name)
	}

	delete(sm.snapshots, name)
	return nil
}

