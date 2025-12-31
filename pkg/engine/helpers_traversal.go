package engine

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// Helper functions for AST traversal

// RelationWithScope captures a relation and its parent scope (FQN)
type RelationWithScope struct {
	Relation *language.Relation
	Scope    string
}

// collectElements traverses the model and returns all element definitions (keyed by FQN) and all relations.
// Uses iterative traversal to avoid closure allocation overhead.
func collectElements(model *language.Model) (map[string]*language.ElementDef, []*language.Relation) {
	if model == nil {
		return make(map[string]*language.ElementDef), nil
	}

	// Estimate typical architecture size
	estimatedElements := len(model.Items) * 4 // Assume ~4 nested elements per top-level
	if estimatedElements < 16 {
		estimatedElements = 16
	}

	elements := make(map[string]*language.ElementDef, estimatedElements)
	relations := make([]*language.Relation, 0, estimatedElements)

	// Use explicit stack for iterative traversal
	type frame struct {
		elem   *language.ElementDef
		parent string
	}
	stack := make([]frame, 0, 16)

	// Initialize with top-level elements
	for _, item := range model.Items {
		if item.ElementDef != nil {
			stack = append(stack, frame{elem: item.ElementDef, parent: ""})
		}
		if item.Relation != nil {
			relations = append(relations, item.Relation)
		}
	}

	// Iterative traversal
	for len(stack) > 0 {
		// Pop
		f := stack[len(stack)-1]
		stack = stack[:len(stack)-1]

		elem := f.elem
		if elem == nil {
			continue
		}

		id := elem.GetID()
		if id == "" {
			continue
		}

		fqn := buildQualifiedID(f.parent, id)
		elements[fqn] = elem

		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					stack = append(stack, frame{elem: item.Element, parent: fqn})
				}
				if item.Relation != nil {
					relations = append(relations, item.Relation)
				}
			}
		}
	}

	return elements, relations
}

// buildQualifiedID constructs a FQN from parent context and element ID.
// Optimized to avoid allocation when parent is empty.
func buildQualifiedID(parentID, id string) string {
	if parentID == "" {
		return id
	}
	// Use simple concatenation - Go compiler optimizes this well
	return parentID + "." + id
}

// collectAllRelations traverses the model and returns all relations with their scope.
// Uses iterative traversal to avoid closure allocation overhead.
func collectAllRelations(model *language.Model) []RelationWithScope {
	if model == nil {
		return nil
	}

	// Estimate capacity
	estimatedRelations := len(model.Items) * 2
	if estimatedRelations < 8 {
		estimatedRelations = 8
	}

	relations := make([]RelationWithScope, 0, estimatedRelations)

	// Use explicit stack for iterative traversal
	type frame struct {
		elem   *language.ElementDef
		parent string
	}
	stack := make([]frame, 0, 16)

	// Process top-level items
	for _, item := range model.Items {
		if item.Relation != nil {
			relations = append(relations, RelationWithScope{
				Relation: item.Relation,
				Scope:    "", // Top level
			})
		}
		if item.ElementDef != nil {
			stack = append(stack, frame{elem: item.ElementDef, parent: ""})
		}
	}

	// Iterative traversal
	for len(stack) > 0 {
		// Pop
		f := stack[len(stack)-1]
		stack = stack[:len(stack)-1]

		elem := f.elem
		if elem == nil {
			continue
		}

		id := elem.GetID()
		if id == "" {
			continue
		}

		fqn := buildQualifiedID(f.parent, id)

		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Relation != nil {
					relations = append(relations, RelationWithScope{
						Relation: item.Relation,
						Scope:    fqn,
					})
				}
				if item.Element != nil {
					stack = append(stack, frame{elem: item.Element, parent: fqn})
				}
			}
		}
	}

	return relations
}

// CollectElementsWithCapacity is like collectElements but allows caller to specify expected capacity.
// Use when the caller knows approximate architecture size for better allocation efficiency.
func CollectElementsWithCapacity(model *language.Model, expectedElements int) (map[string]*language.ElementDef, []*language.Relation) {
	if model == nil {
		return make(map[string]*language.ElementDef), nil
	}

	if expectedElements < 16 {
		expectedElements = 16
	}

	elements := make(map[string]*language.ElementDef, expectedElements)
	relations := make([]*language.Relation, 0, expectedElements/2)

	type frame struct {
		elem   *language.ElementDef
		parent string
	}
	stack := make([]frame, 0, 16)

	for _, item := range model.Items {
		if item.ElementDef != nil {
			stack = append(stack, frame{elem: item.ElementDef, parent: ""})
		}
		if item.Relation != nil {
			relations = append(relations, item.Relation)
		}
	}

	for len(stack) > 0 {
		f := stack[len(stack)-1]
		stack = stack[:len(stack)-1]

		elem := f.elem
		if elem == nil {
			continue
		}

		id := elem.GetID()
		if id == "" {
			continue
		}

		fqn := buildQualifiedID(f.parent, id)
		elements[fqn] = elem

		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					stack = append(stack, frame{elem: item.Element, parent: fqn})
				}
				if item.Relation != nil {
					relations = append(relations, item.Relation)
				}
			}
		}
	}

	return elements, relations
}
