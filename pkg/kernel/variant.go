// pkg/kernel/variant.go
// Package kernel provides variant management for architecture experimentation.
package kernel

import (
	"encoding/json"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/sruja-ai/sruja/pkg/model"
)

// ModelPatch represents a change to the architecture model.
type ModelPatch struct {
	Operation   string          `json:"operation"`   // "add", "update", "remove"
	ElementType string          `json:"elementType"` // "element", "relation", "requirement", etc.
	ElementID   string          `json:"elementId"`
	Payload     json.RawMessage `json:"payload,omitempty"` // Element data for add/update
}

// Variant represents an experimental variant of the architecture.
type Variant struct {
	ID          string       `json:"id"`
	Name        string       `json:"name"`
	Base        string       `json:"base"` // Base snapshot name
	Description string       `json:"description,omitempty"`
	CreatedAt   time.Time    `json:"createdAt"`
	Patches     []ModelPatch `json:"patches"` // Changes from base
	store       *ArchitectureStore
}

// VariantManager manages architecture variants.
type VariantManager struct {
	mu         sync.RWMutex
	variants   map[string]*Variant
	snapshots  *SnapshotManager
	baseStore  *ArchitectureStore // Store the base state when variant is created
	diffEngine *DiffEngine        // Diff engine for computing differences
}

// NewVariantManager creates a new variant manager.
func NewVariantManager(snapshots *SnapshotManager, store *ArchitectureStore) *VariantManager {
	return &VariantManager{
		variants:   make(map[string]*Variant),
		snapshots:  snapshots,
		baseStore:  store,
		diffEngine: NewDiffEngine(),
	}
}

// CreateVariant creates a new variant from a base snapshot.
func (vm *VariantManager) CreateVariant(name, baseSnapshot, description string) (*Variant, error) {
	vm.mu.Lock()
	defer vm.mu.Unlock()

	// Check if variant already exists
	if _, exists := vm.variants[name]; exists {
		return nil, fmt.Errorf("variant '%s' already exists", name)
	}

	// Verify base snapshot exists
	base, ok := vm.snapshots.GetSnapshot(baseSnapshot)
	if !ok {
		return nil, fmt.Errorf("base snapshot '%s' not found", baseSnapshot)
	}

	// Create variant with empty patches
	variant := &Variant{
		ID:          fmt.Sprintf("variant-%d", time.Now().Unix()),
		Name:        name,
		Base:        baseSnapshot,
		Description: description,
		CreatedAt:   time.Now(),
		Patches:     []ModelPatch{},
		store:       NewArchitectureStore(), // Separate store for variant
	}

	// Load base snapshot into variant store
	if err := variant.store.FromJSON(base.IR); err != nil {
		return nil, fmt.Errorf("failed to load base snapshot into variant: %w", err)
	}

	vm.variants[name] = variant

	return variant, nil
}

// GetVariant retrieves a variant by name.
func (vm *VariantManager) GetVariant(name string) (*Variant, bool) {
	vm.mu.RLock()
	defer vm.mu.RUnlock()

	variant, ok := vm.variants[name]
	return variant, ok
}

// ListVariants returns all variants.
func (vm *VariantManager) ListVariants() []*Variant {
	vm.mu.RLock()
	defer vm.mu.RUnlock()

	variants := make([]*Variant, 0, len(vm.variants))
	for _, variant := range vm.variants {
		variants = append(variants, variant)
	}
	return variants
}

// ApplyVariant loads a variant's state into the main store.
func (vm *VariantManager) ApplyVariant(name string) error {
	vm.mu.RLock()
	variant, ok := vm.variants[name]
	vm.mu.RUnlock()

	if !ok {
		return fmt.Errorf("variant '%s' not found", name)
	}

	// Export variant's store
	variantIR, err := variant.store.ToJSON()
	if err != nil {
		return fmt.Errorf("failed to export variant IR: %w", err)
	}

	// Load into main store
	if err := vm.baseStore.FromJSON(variantIR); err != nil {
		return fmt.Errorf("failed to load variant into main store: %w", err)
	}

	return nil
}

// GetVariantStore returns the ArchitectureStore for a variant (for modifications).
func (vm *VariantManager) GetVariantStore(name string) (*ArchitectureStore, error) {
	vm.mu.RLock()
	variant, ok := vm.variants[name]
	vm.mu.RUnlock()

	if !ok {
		return nil, fmt.Errorf("variant '%s' not found", name)
	}

	return variant.store, nil
}

// MergeVariant attempts to merge a variant into the main architecture.
//
// This performs a three-way merge:
//   - Base: The snapshot the variant was created from
//   - Variant: The variant's current state
//   - Current: The main architecture store's current state
//
// Returns:
//   - MergeResult with conflicts and explanation
//   - Error if merge cannot proceed
func (vm *VariantManager) MergeVariant(name string) (*MergeResult, error) {
	vm.mu.RLock()
	variant, ok := vm.variants[name]
	vm.mu.RUnlock()

	if !ok {
		return nil, fmt.Errorf("variant '%s' not found", name)
	}

	// Get base snapshot
	base, ok := vm.snapshots.GetSnapshot(variant.Base)
	if !ok {
		return nil, fmt.Errorf("base snapshot '%s' not found", variant.Base)
	}

	// Load models
	baseStore := NewArchitectureStore()
	if err := baseStore.FromJSON(base.IR); err != nil {
		return nil, fmt.Errorf("failed to load base snapshot: %w", err)
	}
	baseModel := baseStore.GetModel()

	variantModel := variant.store.GetModel()
	currentModel := vm.baseStore.GetModel()

	// Detect conflicts
	conflicts := vm.diffEngine.DetectConflicts(baseModel, variantModel, currentModel)

	// Perform three-way merge
	mergedModel, err := vm.threeWayMerge(baseModel, variantModel, currentModel, conflicts)
	if err != nil {
		return nil, fmt.Errorf("merge failed: %w", err)
	}

	// Generate merge explanation
	explanation := vm.generateMergeExplanation(baseModel, variantModel, currentModel, conflicts, mergedModel)

	result := &MergeResult{
		Success:     len(conflicts) == 0,
		Conflicts:   conflicts,
		MergedModel: mergedModel,
		Explanation: explanation,
	}

	// If no conflicts, apply the merge
	if len(conflicts) == 0 {
		vm.mu.Lock()
		if err := vm.baseStore.UpdateModel(mergedModel); err != nil {
			vm.mu.Unlock()
			return nil, fmt.Errorf("failed to apply merged model: %w", err)
		}
		vm.mu.Unlock()
	}

	return result, nil
}

// MergeResult represents the result of a merge operation.
type MergeResult struct {
	Success     bool         `json:"success"`
	Conflicts   []Conflict   `json:"conflicts,omitempty"`
	MergedModel *model.Model `json:"mergedModel,omitempty"`
	Explanation string       `json:"explanation"`
}

// threeWayMerge performs a three-way merge of base, variant, and current models.
func (vm *VariantManager) threeWayMerge(base, variant, current *model.Model, conflicts []Conflict) (*model.Model, error) {
	// Start with current model as base
	merged := current.Clone()

	if merged.Architecture == nil {
		merged.Architecture = &model.Architecture{
			Elements:     []model.Element{},
			Relations:    []model.Relation{},
			Requirements: []model.Requirement{},
		}
	}

	// Create maps for efficient lookup
	variantElemMap := make(map[string]model.Element)
	if variant != nil && variant.Architecture != nil {
		for _, elem := range variant.Architecture.Elements {
			variantElemMap[elem.ID] = elem
		}
	}

	baseElemMap := make(map[string]model.Element)
	if base != nil && base.Architecture != nil {
		for _, elem := range base.Architecture.Elements {
			baseElemMap[elem.ID] = elem
		}
	}

	// Merge elements
	mergedElemMap := make(map[string]model.Element)
	for _, elem := range merged.Architecture.Elements {
		mergedElemMap[elem.ID] = elem
	}

	// Apply variant changes (if no conflicts)
	for id, variantElem := range variantElemMap {
		_, inBase := baseElemMap[id]
		_, inCurrent := mergedElemMap[id]

		// Check if this element has conflicts
		hasConflict := false
		for _, conflict := range conflicts {
			if conflict.Type == "element" && conflict.ElementID == id {
				hasConflict = true
				break
			}
		}

		if !hasConflict {
			if inBase && inCurrent {
				// Element exists in both - check if variant modified it
				if vm.diffEngine.elementsDiffer(baseElemMap[id], variantElem) {
					// Variant modified it - apply variant's version
					mergedElemMap[id] = variantElem
				}
			} else if !inBase && inCurrent {
				// Element added in current but not in base - keep current
				// (variant can't have modified it if it wasn't in base)
			} else if inBase && !inCurrent {
				// Element removed in current - check if variant still has it
				if _, variantHas := variantElemMap[id]; variantHas {
					// Variant still has it - restore it
					mergedElemMap[id] = variantElem
				}
			} else {
				// Element added in variant - add it
				mergedElemMap[id] = variantElem
			}
		}
	}

	// Convert map back to slice
	merged.Architecture.Elements = make([]model.Element, 0, len(mergedElemMap))
	for _, elem := range mergedElemMap {
		merged.Architecture.Elements = append(merged.Architecture.Elements, elem)
	}

	// Similar logic for relations
	variantRelMap := make(map[string]model.Relation)
	if variant != nil && variant.Architecture != nil {
		for _, rel := range variant.Architecture.Relations {
			key := fmt.Sprintf("%s->%s", rel.From, rel.To)
			variantRelMap[key] = rel
		}
	}

	baseRelMap := make(map[string]model.Relation)
	if base != nil && base.Architecture != nil {
		for _, rel := range base.Architecture.Relations {
			key := fmt.Sprintf("%s->%s", rel.From, rel.To)
			baseRelMap[key] = rel
		}
	}

	mergedRelMap := make(map[string]model.Relation)
	for _, rel := range merged.Architecture.Relations {
		key := fmt.Sprintf("%s->%s", rel.From, rel.To)
		mergedRelMap[key] = rel
	}

	for key, variantRel := range variantRelMap {
		hasConflict := false
		for _, conflict := range conflicts {
			if conflict.Type == "relation" && conflict.ElementID == key {
				hasConflict = true
				break
			}
		}

		if !hasConflict {
			if _, inBase := baseRelMap[key]; inBase {
				if _, inCurrent := mergedRelMap[key]; inCurrent {
					// Both exist - apply variant if it changed
					if vm.diffEngine.relationsDiffer(baseRelMap[key], variantRel) {
						mergedRelMap[key] = variantRel
					}
				} else {
					// Removed in current - restore if variant has it
					mergedRelMap[key] = variantRel
				}
			} else {
				// Added in variant - add it
				mergedRelMap[key] = variantRel
			}
		}
	}

	merged.Architecture.Relations = make([]model.Relation, 0, len(mergedRelMap))
	for _, rel := range mergedRelMap {
		merged.Architecture.Relations = append(merged.Architecture.Relations, rel)
	}

	return merged, nil
}

// generateMergeExplanation generates a human-readable explanation of the merge.
func (vm *VariantManager) generateMergeExplanation(base, variant, current *model.Model, conflicts []Conflict, merged *model.Model) string {
	var explanation strings.Builder

	explanation.WriteString("Merge Summary\n")
	explanation.WriteString("=============\n\n")

	if len(conflicts) == 0 {
		explanation.WriteString("✅ Merge successful - no conflicts detected.\n\n")
	} else {
		explanation.WriteString(fmt.Sprintf("⚠️  Merge completed with %d conflict(s). Manual resolution required.\n\n", len(conflicts)))
		explanation.WriteString("Conflicts:\n")
		for i, conflict := range conflicts {
			explanation.WriteString(fmt.Sprintf("  %d. %s\n", i+1, conflict.Description))
		}
		explanation.WriteString("\n")
	}

	// Compute diffs for summary
	variantDiff := vm.diffEngine.DiffModels(base, variant)
	currentDiff := vm.diffEngine.DiffModels(base, current)

	explanation.WriteString("Changes in Variant:\n")
	explanation.WriteString(fmt.Sprintf("  • Added: %d elements, %d relations\n", len(variantDiff.AddedElements), len(variantDiff.AddedRelations)))
	explanation.WriteString(fmt.Sprintf("  • Removed: %d elements, %d relations\n", len(variantDiff.RemovedElements), len(variantDiff.RemovedRelations)))
	explanation.WriteString(fmt.Sprintf("  • Modified: %d elements, %d relations\n\n", len(variantDiff.ModifiedElements), len(variantDiff.ModifiedRelations)))

	explanation.WriteString("Changes in Current:\n")
	explanation.WriteString(fmt.Sprintf("  • Added: %d elements, %d relations\n", len(currentDiff.AddedElements), len(currentDiff.AddedRelations)))
	explanation.WriteString(fmt.Sprintf("  • Removed: %d elements, %d relations\n", len(currentDiff.RemovedElements), len(currentDiff.RemovedRelations)))
	explanation.WriteString(fmt.Sprintf("  • Modified: %d elements, %d relations\n\n", len(currentDiff.ModifiedElements), len(currentDiff.ModifiedRelations)))

	return explanation.String()
}

// DeleteVariant removes a variant.
func (vm *VariantManager) DeleteVariant(name string) error {
	vm.mu.Lock()
	defer vm.mu.Unlock()

	if _, exists := vm.variants[name]; !exists {
		return fmt.Errorf("variant '%s' not found", name)
	}

	delete(vm.variants, name)
	return nil
}

// ComputeVariantDiff computes the differences between a variant and its base.
func (vm *VariantManager) ComputeVariantDiff(name string) ([]ModelPatch, error) {
	vm.mu.RLock()
	variant, ok := vm.variants[name]
	vm.mu.RUnlock()

	if !ok {
		return nil, fmt.Errorf("variant '%s' not found", name)
	}

	// Get base snapshot
	base, ok := vm.snapshots.GetSnapshot(variant.Base)
	if !ok {
		return nil, fmt.Errorf("base snapshot '%s' not found", variant.Base)
	}

	// Load base model
	baseStore := NewArchitectureStore()
	if err := baseStore.FromJSON(base.IR); err != nil {
		return nil, fmt.Errorf("failed to load base snapshot: %w", err)
	}
	baseModel := baseStore.GetModel()

	// Get variant model
	variantModel := variant.store.GetModel()

	// Compute diff using diff engine
	diffResult := vm.diffEngine.DiffModels(baseModel, variantModel)

	// Convert diff result to patches
	patches := vm.diffToPatches(diffResult)

	// Update variant's stored patches
	vm.mu.Lock()
	variant.Patches = patches
	vm.mu.Unlock()

	return patches, nil
}

// diffToPatches converts a DiffResult to ModelPatch slice.
func (vm *VariantManager) diffToPatches(diff *DiffResult) []ModelPatch {
	patches := []ModelPatch{}

	// Add patches for added elements
	for _, elem := range diff.AddedElements {
		elemJSON, _ := json.Marshal(elem)
		patches = append(patches, ModelPatch{
			Operation:   "add",
			ElementType: "element",
			ElementID:   elem.ID,
			Payload:     elemJSON,
		})
	}

	// Add patches for removed elements
	for _, elem := range diff.RemovedElements {
		patches = append(patches, ModelPatch{
			Operation:   "remove",
			ElementType: "element",
			ElementID:   elem.ID,
		})
	}

	// Add patches for modified elements
	for _, elemDiff := range diff.ModifiedElements {
		elemJSON, _ := json.Marshal(elemDiff.New)
		patches = append(patches, ModelPatch{
			Operation:   "update",
			ElementType: "element",
			ElementID:   elemDiff.ElementID,
			Payload:     elemJSON,
		})
	}

	// Add patches for added relations
	for _, rel := range diff.AddedRelations {
		relJSON, _ := json.Marshal(rel)
		patches = append(patches, ModelPatch{
			Operation:   "add",
			ElementType: "relation",
			ElementID:   fmt.Sprintf("%s->%s", rel.From, rel.To),
			Payload:     relJSON,
		})
	}

	// Add patches for removed relations
	for _, rel := range diff.RemovedRelations {
		patches = append(patches, ModelPatch{
			Operation:   "remove",
			ElementType: "relation",
			ElementID:   fmt.Sprintf("%s->%s", rel.From, rel.To),
		})
	}

	// Add patches for modified relations
	for _, relDiff := range diff.ModifiedRelations {
		relJSON, _ := json.Marshal(relDiff.New)
		patches = append(patches, ModelPatch{
			Operation:   "update",
			ElementType: "relation",
			ElementID:   fmt.Sprintf("%s->%s", relDiff.From, relDiff.To),
			Payload:     relJSON,
		})
	}

	return patches
}
