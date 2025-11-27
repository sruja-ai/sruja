// pkg/kernel/diff.go
// Diff engine for comparing architecture models and detecting changes

package kernel

import (
	"encoding/json"
	"fmt"

	"github.com/sruja-ai/sruja/pkg/model"
)

// DiffResult represents the differences between two models.
type DiffResult struct {
	AddedElements       []model.Element     `json:"addedElements,omitempty"`
	RemovedElements     []model.Element     `json:"removedElements,omitempty"`
	ModifiedElements    []ElementDiff       `json:"modifiedElements,omitempty"`
	AddedRelations      []model.Relation    `json:"addedRelations,omitempty"`
	RemovedRelations    []model.Relation    `json:"removedRelations,omitempty"`
	ModifiedRelations   []RelationDiff      `json:"modifiedRelations,omitempty"`
	AddedRequirements   []model.Requirement `json:"addedRequirements,omitempty"`
	RemovedRequirements []model.Requirement `json:"removedRequirements,omitempty"`
	Conflicts           []Conflict          `json:"conflicts,omitempty"`
}

// ElementDiff represents changes to an element.
type ElementDiff struct {
	ElementID string                 `json:"elementId"`
	Old       model.Element          `json:"old"`
	New       model.Element          `json:"new"`
	Changes   map[string]FieldChange `json:"changes"`
}

// RelationDiff represents changes to a relation.
type RelationDiff struct {
	From    string                 `json:"from"`
	To      string                 `json:"to"`
	Old     model.Relation         `json:"old"`
	New     model.Relation         `json:"new"`
	Changes map[string]FieldChange `json:"changes"`
}

// FieldChange represents a change to a single field.
type FieldChange struct {
	Old interface{} `json:"old,omitempty"`
	New interface{} `json:"new,omitempty"`
}

// Conflict represents a merge conflict.
type Conflict struct {
	Type         string      `json:"type"` // "element", "relation", "requirement"
	ElementID    string      `json:"elementId,omitempty"`
	Field        string      `json:"field,omitempty"`
	BaseValue    interface{} `json:"baseValue,omitempty"`
	VariantValue interface{} `json:"variantValue,omitempty"`
	CurrentValue interface{} `json:"currentValue,omitempty"`
	Description  string      `json:"description"`
}

// DiffEngine compares two architecture models and computes differences.
type DiffEngine struct{}

// NewDiffEngine creates a new diff engine.
func NewDiffEngine() *DiffEngine {
	return &DiffEngine{}
}

// DiffModels computes the differences between two models.
func (de *DiffEngine) DiffModels(base, variant *model.Model) *DiffResult {
	result := &DiffResult{
		AddedElements:       []model.Element{},
		RemovedElements:     []model.Element{},
		ModifiedElements:    []ElementDiff{},
		AddedRelations:      []model.Relation{},
		RemovedRelations:    []model.Relation{},
		ModifiedRelations:   []RelationDiff{},
		AddedRequirements:   []model.Requirement{},
		RemovedRequirements: []model.Requirement{},
		Conflicts:           []Conflict{},
	}

	if base == nil || base.Architecture == nil {
		// If base is empty, everything in variant is added
		if variant != nil && variant.Architecture != nil {
			result.AddedElements = variant.Architecture.Elements
			result.AddedRelations = variant.Architecture.Relations
			result.AddedRequirements = variant.Architecture.Requirements
		}
		return result
	}

	if variant == nil || variant.Architecture == nil {
		// If variant is empty, everything in base is removed
		result.RemovedElements = base.Architecture.Elements
		result.RemovedRelations = base.Architecture.Relations
		result.RemovedRequirements = base.Architecture.Requirements
		return result
	}

	baseArch := base.Architecture
	variantArch := variant.Architecture

	// Diff elements
	de.diffElements(baseArch.Elements, variantArch.Elements, result)

	// Diff relations
	de.diffRelations(baseArch.Relations, variantArch.Relations, result)

	// Diff requirements
	de.diffRequirements(baseArch.Requirements, variantArch.Requirements, result)

	return result
}

// diffElements compares elements between base and variant.
func (de *DiffEngine) diffElements(base, variant []model.Element, result *DiffResult) {
	// Create maps for efficient lookup
	baseMap := make(map[string]model.Element)
	for _, elem := range base {
		baseMap[elem.ID] = elem
	}

	variantMap := make(map[string]model.Element)
	for _, elem := range variant {
		variantMap[elem.ID] = elem
	}

	// Find added and modified elements
	for id, variantElem := range variantMap {
		baseElem, exists := baseMap[id]
		if !exists {
			result.AddedElements = append(result.AddedElements, variantElem)
		} else {
			// Check if element was modified
			if de.elementsDiffer(baseElem, variantElem) {
				changes := de.computeElementChanges(baseElem, variantElem)
				result.ModifiedElements = append(result.ModifiedElements, ElementDiff{
					ElementID: id,
					Old:       baseElem,
					New:       variantElem,
					Changes:   changes,
				})
			}
		}
	}

	// Find removed elements
	for id, baseElem := range baseMap {
		if _, exists := variantMap[id]; !exists {
			result.RemovedElements = append(result.RemovedElements, baseElem)
		}
	}
}

// diffRelations compares relations between base and variant.
func (de *DiffEngine) diffRelations(base, variant []model.Relation, result *DiffResult) {
	// Create maps for efficient lookup (key: from-to)
	baseMap := make(map[string]model.Relation)
	for _, rel := range base {
		key := fmt.Sprintf("%s->%s", rel.From, rel.To)
		baseMap[key] = rel
	}

	variantMap := make(map[string]model.Relation)
	for _, rel := range variant {
		key := fmt.Sprintf("%s->%s", rel.From, rel.To)
		variantMap[key] = rel
	}

	// Find added and modified relations
	for key, variantRel := range variantMap {
		baseRel, exists := baseMap[key]
		if !exists {
			result.AddedRelations = append(result.AddedRelations, variantRel)
		} else {
			// Check if relation was modified
			if de.relationsDiffer(baseRel, variantRel) {
				changes := de.computeRelationChanges(baseRel, variantRel)
				result.ModifiedRelations = append(result.ModifiedRelations, RelationDiff{
					From:    variantRel.From,
					To:      variantRel.To,
					Old:     baseRel,
					New:     variantRel,
					Changes: changes,
				})
			}
		}
	}

	// Find removed relations
	for key, baseRel := range baseMap {
		if _, exists := variantMap[key]; !exists {
			result.RemovedRelations = append(result.RemovedRelations, baseRel)
		}
	}
}

// diffRequirements compares requirements between base and variant.
func (de *DiffEngine) diffRequirements(base, variant []model.Requirement, result *DiffResult) {
	baseMap := make(map[string]model.Requirement)
	for _, req := range base {
		baseMap[req.ID] = req
	}

	variantMap := make(map[string]model.Requirement)
	for _, req := range variant {
		variantMap[req.ID] = req
	}

	// Find added requirements
	for id, variantReq := range variantMap {
		if _, exists := baseMap[id]; !exists {
			result.AddedRequirements = append(result.AddedRequirements, variantReq)
		}
	}

	// Find removed requirements
	for id, baseReq := range baseMap {
		if _, exists := variantMap[id]; !exists {
			result.RemovedRequirements = append(result.RemovedRequirements, baseReq)
		}
	}
}

// elementsDiffer checks if two elements are different.
func (de *DiffEngine) elementsDiffer(a, b model.Element) bool {
	if a.ID != b.ID || a.Type != b.Type || a.Name != b.Name {
		return true
	}
	if a.Description != b.Description || a.Technology != b.Technology {
		return true
	}
	// Compare tags
	if len(a.Tags) != len(b.Tags) {
		return true
	}
	tagMap := make(map[string]bool)
	for _, tag := range a.Tags {
		tagMap[tag] = true
	}
	for _, tag := range b.Tags {
		if !tagMap[tag] {
			return true
		}
	}
	// Compare metadata (simplified - just check if different)
	aMetaJSON, _ := json.Marshal(a.Metadata)
	bMetaJSON, _ := json.Marshal(b.Metadata)
	return string(aMetaJSON) != string(bMetaJSON)
}

// relationsDiffer checks if two relations are different.
func (de *DiffEngine) relationsDiffer(a, b model.Relation) bool {
	if a.From != b.From || a.To != b.To || a.Type != b.Type {
		return true
	}
	if a.Description != b.Description {
		return true
	}
	return false
}

// computeElementChanges computes field-level changes for an element.
func (de *DiffEngine) computeElementChanges(old, new model.Element) map[string]FieldChange {
	changes := make(map[string]FieldChange)

	if old.Name != new.Name {
		changes["name"] = FieldChange{Old: old.Name, New: new.Name}
	}
	if old.Description != new.Description {
		changes["description"] = FieldChange{Old: old.Description, New: new.Description}
	}
	if old.Technology != new.Technology {
		changes["technology"] = FieldChange{Old: old.Technology, New: new.Technology}
	}
	// Tags comparison
	if len(old.Tags) != len(new.Tags) {
		changes["tags"] = FieldChange{Old: old.Tags, New: new.Tags}
	} else {
		// Check if tags are different
		tagMap := make(map[string]bool)
		for _, tag := range old.Tags {
			tagMap[tag] = true
		}
		for _, tag := range new.Tags {
			if !tagMap[tag] {
				changes["tags"] = FieldChange{Old: old.Tags, New: new.Tags}
				break
			}
		}
	}

	return changes
}

// computeRelationChanges computes field-level changes for a relation.
func (de *DiffEngine) computeRelationChanges(old, new model.Relation) map[string]FieldChange {
	changes := make(map[string]FieldChange)

	if old.Type != new.Type {
		changes["type"] = FieldChange{Old: old.Type, New: new.Type}
	}
	if old.Description != new.Description {
		changes["description"] = FieldChange{Old: old.Description, New: new.Description}
	}

	return changes
}

// DetectConflicts detects conflicts between base, variant, and current models.
func (de *DiffEngine) DetectConflicts(base, variant, current *model.Model) []Conflict {
	conflicts := []Conflict{}

	if base == nil || variant == nil || current == nil {
		return conflicts
	}

	baseArch := base.Architecture
	variantArch := variant.Architecture
	currentArch := current.Architecture

	if baseArch == nil || variantArch == nil || currentArch == nil {
		return conflicts
	}

	// Detect element conflicts
	de.detectElementConflicts(baseArch.Elements, variantArch.Elements, currentArch.Elements, &conflicts)

	// Detect relation conflicts
	de.detectRelationConflicts(baseArch.Relations, variantArch.Relations, currentArch.Relations, &conflicts)

	return conflicts
}

// detectElementConflicts detects conflicts in elements.
func (de *DiffEngine) detectElementConflicts(base, variant, current []model.Element, conflicts *[]Conflict) {
	baseMap := make(map[string]model.Element)
	for _, elem := range base {
		baseMap[elem.ID] = elem
	}

	variantMap := make(map[string]model.Element)
	for _, elem := range variant {
		variantMap[elem.ID] = elem
	}

	currentMap := make(map[string]model.Element)
	for _, elem := range current {
		currentMap[elem.ID] = elem
	}

	// Check for conflicts: element modified in both variant and current
	for id, variantElem := range variantMap {
		baseElem, inBase := baseMap[id]
		currentElem, inCurrent := currentMap[id]

		if inBase && inCurrent {
			// Element exists in both base and current
			// Check if both variant and current modified it differently
			variantChanged := de.elementsDiffer(baseElem, variantElem)
			currentChanged := de.elementsDiffer(baseElem, currentElem)

			if variantChanged && currentChanged {
				// Both modified - check if they conflict
				if de.elementsDiffer(variantElem, currentElem) {
					// Conflict detected
					changes := de.computeElementChanges(variantElem, currentElem)
					for field, change := range changes {
						*conflicts = append(*conflicts, Conflict{
							Type:         "element",
							ElementID:    id,
							Field:        field,
							BaseValue:    getFieldValue(baseElem, field),
							VariantValue: change.New,
							CurrentValue: change.Old,
							Description:  fmt.Sprintf("Element '%s' field '%s' modified differently in variant and current", id, field),
						})
					}
				}
			}
		}
	}
}

// detectRelationConflicts detects conflicts in relations.
func (de *DiffEngine) detectRelationConflicts(base, variant, current []model.Relation, conflicts *[]Conflict) {
	baseMap := make(map[string]model.Relation)
	for _, rel := range base {
		key := fmt.Sprintf("%s->%s", rel.From, rel.To)
		baseMap[key] = rel
	}

	variantMap := make(map[string]model.Relation)
	for _, rel := range variant {
		key := fmt.Sprintf("%s->%s", rel.From, rel.To)
		variantMap[key] = rel
	}

	currentMap := make(map[string]model.Relation)
	for _, rel := range current {
		key := fmt.Sprintf("%s->%s", rel.From, rel.To)
		currentMap[key] = rel
	}

	// Check for conflicts: relation modified in both variant and current
	for key, variantRel := range variantMap {
		baseRel, inBase := baseMap[key]
		currentRel, inCurrent := currentMap[key]

		if inBase && inCurrent {
			variantChanged := de.relationsDiffer(baseRel, variantRel)
			currentChanged := de.relationsDiffer(baseRel, currentRel)

			if variantChanged && currentChanged {
				if de.relationsDiffer(variantRel, currentRel) {
					*conflicts = append(*conflicts, Conflict{
						Type:         "relation",
						ElementID:    key,
						BaseValue:    baseRel,
						VariantValue: variantRel,
						CurrentValue: currentRel,
						Description:  fmt.Sprintf("Relation '%s' modified differently in variant and current", key),
					})
				}
			}
		}
	}
}

// getFieldValue extracts a field value from an element (helper function).
func getFieldValue(elem model.Element, field string) interface{} {
	switch field {
	case "name":
		return elem.Name
	case "description":
		return elem.Description
	case "technology":
		return elem.Technology
	case "tags":
		return elem.Tags
	default:
		return nil
	}
}
