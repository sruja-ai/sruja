// pkg/kernel/store.go
// Package kernel provides the Architecture Kernel for Sruja Notebooks.
package kernel

import (
	"encoding/json"
	"sync"
	"time"

	"github.com/sruja-ai/sruja/pkg/model"
)

// ArchitectureStore maintains the stateful architecture model (IR) for the kernel.
//
// This is the canonical internal representation used by:
//   - Semantic model builder
//   - Validators
//   - Query engine
//   - Diff/versioning engine
//   - Event lifecycle simulation
//   - Diagram generation
//   - LSP
//   - AI layer
//   - MCP integration
type ArchitectureStore struct {
	mu       sync.RWMutex
	model    *model.Model // Current architecture model (IR)
	metadata map[string]string
	version  int64 // Internal version counter for change tracking
}

// NewArchitectureStore creates a new empty ArchitectureStore.
func NewArchitectureStore() *ArchitectureStore {
	return &ArchitectureStore{
		model: &model.Model{
			Version:     "1.0",
			GeneratedAt: time.Now(),
			Architecture: &model.Architecture{
				Elements:     []model.Element{},
				Relations:    []model.Relation{},
				Requirements: []model.Requirement{},
				ADRs:         []model.ADR{},
				Journeys:     []model.Journey{},
			},
		},
		metadata: make(map[string]string),
		version:  0,
	}
}

// GetModel returns a copy of the current architecture model.
func (s *ArchitectureStore) GetModel() *model.Model {
	s.mu.RLock()
	defer s.mu.RUnlock()

	// Return a deep copy to prevent external mutations
	return s.model.Clone()
}

// UpdateModel merges updates from a new model into the store.
//
// This performs incremental updates:
//   - New elements are added
//   - Existing elements are updated
//   - Elements with same ID are replaced
func (s *ArchitectureStore) UpdateModel(newModel *model.Model) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	if newModel == nil || newModel.Architecture == nil {
		return nil // No-op for empty models
	}

	// Merge elements by ID
	elementMap := make(map[string]model.Element)
	for _, elem := range s.model.Architecture.Elements {
		elementMap[elem.ID] = elem
	}
	for _, elem := range newModel.Architecture.Elements {
		elementMap[elem.ID] = elem
	}

	// Merge relations (append new ones, deduplicate)
	relationMap := make(map[string]model.Relation)
	for _, rel := range s.model.Architecture.Relations {
		key := rel.From + ":" + string(rel.Type) + ":" + rel.To
		relationMap[key] = rel
	}
	for _, rel := range newModel.Architecture.Relations {
		key := rel.From + ":" + string(rel.Type) + ":" + rel.To
		relationMap[key] = rel
	}

	// Merge requirements
	reqMap := make(map[string]model.Requirement)
	for _, req := range s.model.Architecture.Requirements {
		reqMap[req.ID] = req
	}
	for _, req := range newModel.Architecture.Requirements {
		reqMap[req.ID] = req
	}

	// Merge ADRs
	adrMap := make(map[string]model.ADR)
	for _, adr := range s.model.Architecture.ADRs {
		adrMap[adr.ID] = adr
	}
	for _, adr := range newModel.Architecture.ADRs {
		adrMap[adr.ID] = adr
	}

	// Merge journeys
	journeyMap := make(map[string]model.Journey)
	for _, journey := range s.model.Architecture.Journeys {
		journeyMap[journey.ID] = journey
	}
	for _, journey := range newModel.Architecture.Journeys {
		journeyMap[journey.ID] = journey
	}

	// Rebuild slices
	elements := make([]model.Element, 0, len(elementMap))
	for _, elem := range elementMap {
		elements = append(elements, elem)
	}
	relations := make([]model.Relation, 0, len(relationMap))
	for _, rel := range relationMap {
		relations = append(relations, rel)
	}
	requirements := make([]model.Requirement, 0, len(reqMap))
	for _, req := range reqMap {
		requirements = append(requirements, req)
	}
	adrs := make([]model.ADR, 0, len(adrMap))
	for _, adr := range adrMap {
		adrs = append(adrs, adr)
	}
	journeys := make([]model.Journey, 0, len(journeyMap))
	for _, journey := range journeyMap {
		journeys = append(journeys, journey)
	}

	// Update model
	s.model.Architecture.Elements = elements
	s.model.Architecture.Relations = relations
	s.model.Architecture.Requirements = requirements
	s.model.Architecture.ADRs = adrs
	s.model.Architecture.Journeys = journeys
	s.model.GeneratedAt = time.Now()
	s.version++

	return nil
}

// RemoveElementsByCell removes elements that were defined in a specific cell.
//
// This is used for incremental updates when a cell is re-executed.
func (s *ArchitectureStore) RemoveElementsByCell(cellID string) {
	s.mu.Lock()
	defer s.mu.Unlock()

	// Filter out elements from this cell
	var elements []model.Element
	for _, elem := range s.model.Architecture.Elements {
		if elem.Location.File != cellID {
			elements = append(elements, elem)
		}
	}
	s.model.Architecture.Elements = elements

	// Filter relations
	var relations []model.Relation
	for _, rel := range s.model.Architecture.Relations {
		if rel.Location.File != cellID {
			relations = append(relations, rel)
		}
	}
	s.model.Architecture.Relations = relations

	s.version++
}

// ToJSON serializes the store to JSON.
func (s *ArchitectureStore) ToJSON() ([]byte, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	return json.MarshalIndent(s.model, "", "  ")
}

// FromJSON deserializes a store from JSON.
func (s *ArchitectureStore) FromJSON(data []byte) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	var m model.Model
	if err := json.Unmarshal(data, &m); err != nil {
		return err
	}

	s.model = &m
	s.version++

	return nil
}

// GetVersion returns the current version number (for change tracking).
func (s *ArchitectureStore) GetVersion() int64 {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.version
}

// SetMetadata sets a metadata key-value pair.
func (s *ArchitectureStore) SetMetadata(key, value string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.metadata[key] = value
}

// GetMetadata gets a metadata value by key.
func (s *ArchitectureStore) GetMetadata(key string) (string, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	val, ok := s.metadata[key]
	return val, ok
}

// Reset clears the store to an empty state.
func (s *ArchitectureStore) Reset() {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.model = &model.Model{
		Version:     "1.0",
		GeneratedAt: time.Now(),
		Architecture: &model.Architecture{
			Elements:     []model.Element{},
			Relations:    []model.Relation{},
			Requirements: []model.Requirement{},
			ADRs:         []model.ADR{},
			Journeys:     []model.Journey{},
		},
	}
	s.metadata = make(map[string]string)
	s.version = 0
}
