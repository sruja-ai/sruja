// pkg/engine/validator_test.go
package engine

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

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

func TestValidator_Validate_MultipleRules(_ *testing.T) {
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
				{From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{"B"}}},
				{From: language.QualifiedIdent{Parts: []string{"B"}}, To: language.QualifiedIdent{Parts: []string{"C"}}},
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
				{From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{"B"}}},
				{From: language.QualifiedIdent{Parts: []string{"B"}}, To: language.QualifiedIdent{Parts: []string{"A"}}},
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
						{From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{"B"}}},
						{From: language.QualifiedIdent{Parts: []string{"B"}}, To: language.QualifiedIdent{Parts: []string{"A"}}},
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

func TestValidator_RegisterDefaultRules(t *testing.T) {
	v := NewValidator()
	v.RegisterDefaultRules()

	// Check that default rules are registered
	if len(v.Rules) == 0 {
		t.Error("RegisterDefaultRules should register at least one rule")
	}

	// Verify some expected rules are present
	ruleNames := make(map[string]bool)
	for _, rule := range v.Rules {
		ruleNames[rule.Name()] = true
	}

	// Check that we have the expected number of default rules
	// RegisterDefaultRules should register at least 8 rules
	if len(v.Rules) < 8 {
		t.Errorf("Expected at least 8 default rules, got %d", len(v.Rules))
	}

	// Verify some key rules are present by checking their names
	expectedRuleNames := map[string]bool{
		"Unique IDs":                  false,
		"Valid References":            false,
		"CycleDetection":              false,
		"OrphanDetection":             false,
		"SimplicityGuidance":          false,
		"Layer Violation":             false,
		"ScenarioReferenceValidation": false,
		"SLO Validation":              false,
	}

	for _, rule := range v.Rules {
		ruleName := rule.Name()
		for expected := range expectedRuleNames {
			if strings.Contains(ruleName, expected) {
				expectedRuleNames[expected] = true
			}
		}
	}

	for expected, found := range expectedRuleNames {
		if !found {
			t.Errorf("Expected rule containing '%s' not found in registered rules. Got: %v", expected, ruleNames)
		}
	}
}

func TestValidator_RegisterDefaultRules_CanValidate(t *testing.T) {
    t.Helper()
    v := NewValidator()
	v.RegisterDefaultRules()

	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{ID: "Sys1", Label: "System 1"},
				{ID: "Sys2", Label: "System 2"},
			},
		},
	}

	// Should run without panicking
	diags := v.Validate(prog)
	_ = diags // We're just checking it doesn't panic
}

func TestDatabaseIsolationRule_Name(t *testing.T) {
	rule := &DatabaseIsolationRule{}
	if rule.Name() != "Database Isolation" {
		t.Errorf("Expected name 'Database Isolation', got '%s'", rule.Name())
	}
}

func TestPublicInterfaceDocumentationRule_Name(t *testing.T) {
	rule := &PublicInterfaceDocumentationRule{}
	if rule.Name() != "Public Interface Documentation" {
		t.Errorf("Expected name 'Public Interface Documentation', got '%s'", rule.Name())
	}
}

func TestLayerViolationRule_Name(t *testing.T) {
	rule := &LayerViolationRule{}
	if rule.Name() == "" {
		t.Error("Name() should return rule name")
	}
}
