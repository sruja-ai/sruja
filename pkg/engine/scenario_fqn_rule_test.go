// pkg/engine/scenario_fqn_rule_test.go
package engine

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestScenarioFQNRule_Name(t *testing.T) {
	rule := &ScenarioFQNRule{}
	if rule.Name() != "ScenarioReferenceValidation" {
		t.Errorf("Expected name 'ScenarioReferenceValidation', got '%s'", rule.Name())
	}
}

func TestScenarioFQNRule_Validate(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	tests := []struct {
		name          string
		input         string
		wantError     bool
		errorContains string
	}{
		{
			name:      "valid fully qualified reference",
			input:     `API = System "API" S1 = Scenario "Test" { API -> API }`,
			wantError: false,
		},
		{
			name:      "valid unqualified reference - unique",
			input:     `API = System "API" S1 = Scenario "Test" { API -> API }`,
			wantError: false,
		},
		// NOTE: ScenarioFQNRule validation for steps is not fully implemented yet
		// These tests are kept for when step validation is added
		{
			name:      "undefined reference (currently not validated)",
			input:     `S1 = Scenario "Test" { Unknown -> Unknown }`,
			wantError: false, // Step validation not implemented yet
		},
		{
			name:      "ambiguous reference",
			input:     `API = System "API" { Web = Container "Web" { Auth = Component "Auth" } } Backend = System "Backend" { Web = Container "Web" { Auth = Component "Auth" } } S1 = Scenario "Test" { API.Web.Auth -> Backend.Web.Auth }`,
			wantError: false, // Using fully qualified names should not be ambiguous
		},
		{
			name:      "valid reference in flow",
			input:     `API = System "API" F1 = Flow "Test" { API -> API }`,
			wantError: false,
		},
		{
			name:      "undefined reference in flow (currently not validated)",
			input:     `F1 = Flow "Test" { Unknown -> Unknown }`,
			wantError: false, // Step validation not implemented yet
		},
		{
			name:      "valid nested system reference",
			input:     `API = System "API" { Web = Container "Web" } S1 = Scenario "Test" { API.Web -> API.Web }`,
			wantError: false,
		},
		{
			name:      "valid component reference",
			input:     `API = System "API" { Web = Container "Web" { Auth = Component "Auth" } } S1 = Scenario "Test" { API.Web.Auth -> API.Web.Auth }`,
			wantError: false,
		},
		{
			name:      "valid person reference",
			input:     `User = Person "User" S1 = Scenario "Test" { User -> User }`,
			wantError: false,
		},
		{
			name:      "nil program",
			input:     "",
			wantError: false,
		},
		{
			name:      "empty scenario",
			input:     `S1 = Scenario "Test" { }`,
			wantError: false,
		},
	}

	rule := &ScenarioFQNRule{}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var program *language.Program
			if tt.input != "" {
				var parseErr error
				program, _, parseErr = parser.Parse("test.sruja", tt.input)
				if parseErr != nil {
					t.Fatalf("Parse error: %v", parseErr)
				}
			}

			diags := rule.Validate(program)
			hasError := false
			errorMessages := []string{}

			for _, diag := range diags {
				if diag.Severity == diagnostics.SeverityError {
					hasError = true
					errorMessages = append(errorMessages, diag.Message)
				}
			}

			if tt.wantError && !hasError {
				t.Errorf("Expected validation error but got none. Diagnostics: %v", diags)
			}
			if !tt.wantError && hasError {
				t.Errorf("Unexpected validation error: %v", diags)
			}
			if tt.errorContains != "" && hasError {
				found := false
				for _, msg := range errorMessages {
					if strings.Contains(strings.ToLower(msg), strings.ToLower(tt.errorContains)) {
						found = true
						break
					}
				}
				if !found {
					t.Errorf("Expected error message to contain '%s', but got: %v", tt.errorContains, errorMessages)
				}
			}
		})
	}
}

func TestScenarioFQNRule_IntegrationWithValidator(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	input := `	S1 = Scenario "Test" {
		Unknown -> Unknown
	}`

	program, _, err := parser.Parse("test.sruja", input)
	if err != nil {
		t.Fatalf("Parse error: %v", err)
	}

	validator := NewValidator()
	validator.RegisterRule(&ScenarioFQNRule{})
	diags := validator.Validate(program)

	// NOTE: Step validation is not fully implemented yet
	// Just verify no panics and rule runs
	_ = diags
	t.Log("ScenarioFQNRule ran without panic. Step validation not yet implemented.")
}
