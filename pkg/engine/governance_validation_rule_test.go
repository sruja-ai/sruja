package engine

import (
	"testing"

	"github.com/alecthomas/participle/v2/lexer"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestGovernanceValidationRule_DuplicateRequirements(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "requirement", Name: "REQ001", Pos: lexer.Position{Line: 1}}}},
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "requirement", Name: "REQ001", Pos: lexer.Position{Line: 5}}}}, // Duplicate
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
		Model: &language.Model{
			Items: []language.ModelItem{
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "adr", Name: "ADR001", Pos: lexer.Position{Line: 1}}}},
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "adr", Name: "ADR001", Pos: lexer.Position{Line: 10}}}}, // Duplicate
			},
		},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 1 {
		t.Errorf("Expected 1 diagnostic, got %d", len(diags))
	}

	if len(diags) > 0 {
		if diags[0].Code != "E201" { // Generic duplicate ID code
			t.Errorf("Expected code 'E201', got '%s'", diags[0].Code)
		}
	}
}

func TestGovernanceValidationRule_DuplicatePolicies(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "policy", Name: "POL001", Pos: lexer.Position{Line: 1}}}},
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "policy", Name: "POL001", Pos: lexer.Position{Line: 15}}}}, // Duplicate
			},
		},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 1 {
		t.Errorf("Expected 1 diagnostic, got %d", len(diags))
	}

	if len(diags) > 0 {
		if diags[0].Code != "E201" { // Generic duplicate ID code
			t.Errorf("Expected code 'E201', got '%s'", diags[0].Code)
		}
	}
}

func TestGovernanceValidationRule_DuplicateScenarios(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "scenario", Name: "S001", Pos: lexer.Position{Line: 1}}}},
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "scenario", Name: "S001", Pos: lexer.Position{Line: 20}}}}, // Duplicate
			},
		},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 1 {
		t.Errorf("Expected 1 diagnostic, got %d", len(diags))
	}

	if len(diags) > 0 {
		if diags[0].Code != "E201" { // Generic duplicate ID code
			t.Errorf("Expected code 'E201', got '%s'", diags[0].Code)
		}
	}
}

func TestGovernanceValidationRule_DuplicateFlows(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "flow", Name: "F001", Pos: lexer.Position{Line: 1}}}},
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "flow", Name: "F001", Pos: lexer.Position{Line: 25}}}}, // Duplicate
			},
		},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 1 {
		t.Errorf("Expected 1 diagnostic, got %d", len(diags))
	}

	if len(diags) > 0 {
		if diags[0].Code != "E201" { // Generic duplicate ID code
			t.Errorf("Expected code 'E201', got '%s'", diags[0].Code)
		}
	}
}

func TestGovernanceValidationRule_UniqueIDs(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "requirement", Name: "REQ001", Pos: lexer.Position{Line: 1}}}},
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "requirement", Name: "REQ002", Pos: lexer.Position{Line: 2}}}},
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "adr", Name: "ADR001", Pos: lexer.Position{Line: 3}}}},
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "adr", Name: "ADR002", Pos: lexer.Position{Line: 4}}}},
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Kind: "policy", Name: "POL001", Pos: lexer.Position{Line: 5}}}},
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
		Model: &language.Model{},
	}

	rule := &GovernanceValidationRule{}
	diags := rule.Validate(prog)

	if len(diags) != 0 {
		t.Errorf("Expected no diagnostics for empty model, got %d", len(diags))
	}
}
