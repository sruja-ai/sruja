package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
)

func TestValidReferences_PersonContainer(t *testing.T) {
	dsl := `
Customer = person "Customer"
Order = system "Order System" {
  API = container "Order API"
}
Customer -> API "Uses"
`
	program := parse(t, dsl)

	rule := &engine.ValidReferenceRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Fatalf("Expected 0 reference errors, got %d: %+v", len(errs), errs)
	}
}

func TestRuleNames(t *testing.T) {
	rules := []engine.Rule{
		&engine.ExternalDependencyRule{},
		&engine.SimplicityRule{},
		&engine.OrphanDetectionRule{}, // Corrected from OrphanRule
		&engine.CycleDetectionRule{},  // Corrected from CycleRule
		&engine.UniqueIDRule{},
		&engine.ValidReferenceRule{},
		&engine.CompletenessRule{},
	}

	expected := []string{
		"ExternalDependency",
		"SimplicityGuidance",
		"OrphanDetection",
		"CycleDetection",
		"Unique IDs",
		"Valid References",
		"CompletenessCheck",
	}

	for i, r := range rules {
		if r.Name() != expected[i] {
			t.Errorf("Expected rule name %s, got %s", expected[i], r.Name())
		}
	}
}

func TestOrphanDetection_ParentSystemMarkedUsed(t *testing.T) {
	dsl := `
Order = system "Order System" {
  API = container "Order API"
}
Customer = person "Customer"
Customer -> API "Uses"
`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	for _, e := range errs {
		if e.Message == "Orphan element 'Order' is defined but never used in any relation." {
			t.Fatalf("Parent system should be marked used when its container is used")
		}
	}
}

func TestUniqueIDRule_DuplicateIDs(t *testing.T) {
	dsl := `
A = system "System A" { X = container "X" }
B = system "System B" { X = container "X Duplicate" }
`
	program := parse(t, dsl)

	rule := &engine.UniqueIDRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Fatalf("Expected duplicate ID errors, got none")
	}
}
