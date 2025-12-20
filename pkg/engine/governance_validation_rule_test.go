package engine

import (
	"testing"

	"github.com/alecthomas/participle/v2/lexer"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestGovernanceValidationRule_DuplicateRequirements(t *testing.T) {
	prog := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{Requirement: &language.Requirement{ID: "REQ001", Pos: lexer.Position{Line: 1}}},
				{Requirement: &language.Requirement{ID: "REQ001", Pos: lexer.Position{Line: 5}}}, // Duplicate
			},
		},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 1 {
		t.Errorf("Expected 1 diagnostic, got %d", len(diags))
	}

	if len(diags) > 0 {
		if diags[0].Code != diagnostics.CodeDuplicateIdentifier {
			t.Errorf("Expected code '%s', got '%s'", diagnostics.CodeDuplicateIdentifier, diags[0].Code)
		}
		if diags[0].Severity != diagnostics.SeverityError {
			t.Errorf("Expected severity Error, got %v", diags[0].Severity)
		}
	}
}

func TestGovernanceValidationRule_DuplicateADRs(t *testing.T) {
	prog := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{ADR: &language.ADR{ID: "ADR001", Pos: lexer.Position{Line: 1}}},
				{ADR: &language.ADR{ID: "ADR001", Pos: lexer.Position{Line: 10}}}, // Duplicate
			},
		},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 1 {
		t.Errorf("Expected 1 diagnostic, got %d", len(diags))
	}

	if len(diags) > 0 {
		if diags[0].Code != "duplicate-adr-id" {
			t.Errorf("Expected code 'duplicate-adr-id', got '%s'", diags[0].Code)
		}
	}
}

func TestGovernanceValidationRule_DuplicatePolicies(t *testing.T) {
	prog := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{Policy: &language.Policy{ID: "POL001", Description: "Test", Pos: lexer.Position{Line: 1}}},
				{Policy: &language.Policy{ID: "POL001", Description: "Test", Pos: lexer.Position{Line: 15}}}, // Duplicate
			},
		},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 1 {
		t.Errorf("Expected 1 diagnostic, got %d", len(diags))
	}

	if len(diags) > 0 {
		if diags[0].Code != "duplicate-policy-id" {
			t.Errorf("Expected code 'duplicate-policy-id', got '%s'", diags[0].Code)
		}
	}
}

func TestGovernanceValidationRule_DuplicateScenarios(t *testing.T) {
	prog := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{Scenario: &language.Scenario{ID: "S001", Pos: lexer.Position{Line: 1}}},
				{Scenario: &language.Scenario{ID: "S001", Pos: lexer.Position{Line: 20}}}, // Duplicate
			},
		},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 1 {
		t.Errorf("Expected 1 diagnostic, got %d", len(diags))
	}

	if len(diags) > 0 {
		if diags[0].Code != "duplicate-scenario-id" {
			t.Errorf("Expected code 'duplicate-scenario-id', got '%s'", diags[0].Code)
		}
	}
}

func TestGovernanceValidationRule_DuplicateFlows(t *testing.T) {
	prog := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{Flow: &language.Flow{ID: "F001", Pos: lexer.Position{Line: 1}}},
				{Flow: &language.Flow{ID: "F001", Pos: lexer.Position{Line: 25}}}, // Duplicate
			},
		},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 1 {
		t.Errorf("Expected 1 diagnostic, got %d", len(diags))
	}

	if len(diags) > 0 {
		if diags[0].Code != "duplicate-flow-id" {
			t.Errorf("Expected code 'duplicate-flow-id', got '%s'", diags[0].Code)
		}
	}
}

func TestGovernanceValidationRule_UniqueIDs(t *testing.T) {
	prog := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{Requirement: &language.Requirement{ID: "REQ001", Pos: lexer.Position{Line: 1}}},
				{Requirement: &language.Requirement{ID: "REQ002", Pos: lexer.Position{Line: 2}}},
				{ADR: &language.ADR{ID: "ADR001", Pos: lexer.Position{Line: 3}}},
				{ADR: &language.ADR{ID: "ADR002", Pos: lexer.Position{Line: 4}}},
				{Policy: &language.Policy{ID: "POL001", Description: "Test", Pos: lexer.Position{Line: 5}}},
			},
		},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 0 {
		t.Errorf("Expected no diagnostics for unique IDs, got %d", len(diags))
	}
}

func TestGovernanceValidationRule_NilProgram(t *testing.T) {
	rule := &GovernanceValidationRule{}
	diags := rule.Validate(nil)

	if len(diags) != 0 {
		t.Errorf("Expected no diagnostics for nil program, got %d", len(diags))
	}
}

func TestGovernanceValidationRule_EmptyModel(t *testing.T) {
	prog := &language.Program{
		Model: &language.ModelBlock{},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 0 {
		t.Errorf("Expected no diagnostics for empty model, got %d", len(diags))
	}
}
