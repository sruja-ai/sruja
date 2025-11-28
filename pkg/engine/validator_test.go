// pkg/engine/validator_test.go
package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestValidationError_String(t *testing.T) {
	err := ValidationError{
		Message: "Test error",
		Line:    10,
		Column:  5,
	}
	str := err.String()
	if str == "" {
		t.Error("String() should return error message")
	}
	if !contains(str, "Test error") {
		t.Error("String() should contain error message")
	}
}

func TestValidationError_Error(t *testing.T) {
	err := ValidationError{
		Message: "Test error",
		Line:    5,
		Column:  3,
	}
	errStr := err.Error()
	if errStr == "" {
		t.Error("Error() should return error message")
	}
}

func TestNewValidator(t *testing.T) {
	v := NewValidator()
	if v == nil {
		t.Fatal("NewValidator should not return nil")
	}
	if len(v.Rules) != 0 {
		t.Error("New validator should have no rules")
	}
}

func TestValidator_RegisterRule(t *testing.T) {
	v := NewValidator()
	rule := &UniqueIDRule{}
	v.RegisterRule(rule)
	if len(v.Rules) != 1 {
		t.Errorf("Expected 1 rule, got %d", len(v.Rules))
	}
}

func TestValidator_Validate(t *testing.T) {
	v := NewValidator()
	rule := &UniqueIDRule{}
	v.RegisterRule(rule)

	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{ID: "Sys1", Label: "System 1"},
				{ID: "Sys2", Label: "System 2"},
			},
		},
	}

	errs := v.Validate(prog)
	// Should have no errors for unique IDs
	if len(errs) != 0 {
		t.Errorf("Expected no errors, got %d", len(errs))
	}
}

func TestValidator_Validate_MultipleRules(t *testing.T) {
	v := NewValidator()
	v.RegisterRule(&UniqueIDRule{})
	v.RegisterRule(&ValidReferenceRule{})

	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{ID: "Sys1", Label: "System 1"},
			},
		},
	}

	errs := v.Validate(prog)
	// Should run all rules
	_ = errs
}

func TestCycleDetectionRule_Name(t *testing.T) {
	rule := &CycleDetectionRule{}
	if rule.Name() != "CycleDetection" {
		t.Errorf("Expected 'CycleDetection', got '%s'", rule.Name())
	}
}

func TestCycleDetectionRule_Validate_NoCycle(t *testing.T) {
	rule := &CycleDetectionRule{}
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Relations: []*language.Relation{
				{From: "A", To: "B"},
				{From: "B", To: "C"},
			},
		},
	}

	errs := rule.Validate(prog)
	if len(errs) != 0 {
		t.Errorf("Expected no errors, got %d", len(errs))
	}
}

func TestCycleDetectionRule_Validate_Cycle(t *testing.T) {
	rule := &CycleDetectionRule{}
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Relations: []*language.Relation{
				{From: "A", To: "B"},
				{From: "B", To: "A"},
			},
		},
	}

	errs := rule.Validate(prog)
	if len(errs) == 0 {
		t.Error("Expected cycle detection error")
	}
}

func TestCycleDetectionRule_Validate_NilArchitecture(t *testing.T) {
	rule := &CycleDetectionRule{}
	prog := &language.Program{}

	errs := rule.Validate(prog)
	if len(errs) != 0 {
		t.Errorf("Expected no errors for nil architecture, got %d", len(errs))
	}
}

func TestCycleDetectionRule_Validate_SystemRelations(t *testing.T) {
	rule := &CycleDetectionRule{}
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys",
					Relations: []*language.Relation{
						{From: "A", To: "B"},
						{From: "B", To: "A"},
					},
				},
			},
		},
	}

	errs := rule.Validate(prog)
	if len(errs) == 0 {
		t.Error("Expected cycle detection error in system relations")
	}
}

func TestUniqueIDRule_Name(t *testing.T) {
	rule := &UniqueIDRule{}
	if rule.Name() == "" {
		t.Error("Name() should return rule name")
	}
}

func TestValidReferenceRule_Name(t *testing.T) {
	rule := &ValidReferenceRule{}
	if rule.Name() == "" {
		t.Error("Name() should return rule name")
	}
}

func TestOrphanDetectionRule_Name(t *testing.T) {
	rule := &OrphanDetectionRule{}
	if rule.Name() == "" {
		t.Error("Name() should return rule name")
	}
}

func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(substr) == 0 ||
		(len(s) > len(substr) && (s[:len(substr)] == substr ||
			s[len(s)-len(substr):] == substr ||
			containsMiddle(s, substr))))
}

func containsMiddle(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
