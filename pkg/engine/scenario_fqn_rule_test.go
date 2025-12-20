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
			input:     `model { system API "API" scenario S1 "Test" { API -> API } }`,
			wantError: false,
		},
		{
			name:      "valid unqualified reference - unique",
			input:     `model { system API "API" scenario S1 "Test" { API -> API } }`,
			wantError: false,
		},
		{
			name:          "undefined reference",
			input:         `model { scenario S1 "Test" "Test scenario" { Unknown -> Unknown } }`,
			wantError:     true,
			errorContains: "undefined element",
		},
		{
			name:      "ambiguous reference",
			input:     `model { API = system "API" { Web = container "Web" { Auth = component "Auth" } } Backend = system "Backend" { Web = container "Web" { Auth = component "Auth" } } scenario S1 "Test" "Test scenario" { API.Web.Auth -> Backend.Web.Auth } }`,
			wantError: false, // Using fully qualified names should not be ambiguous
		},
		{
			name:      "valid reference in flow",
			input:     `model { API = system "API" flow F1 "Test" "Test flow" { API -> API } }`,
			wantError: false,
		},
		{
			name:          "undefined reference in flow",
			input:         `model { flow F1 "Test" "Test flow" { Unknown -> Unknown } }`,
			wantError:     true,
			errorContains: "undefined element",
		},
		{
			name:      "valid nested system reference",
			input:     `model { API = system "API" { Web = container "Web" } scenario S1 "Test" "Test scenario" { API.Web -> API.Web } }`,
			wantError: false,
		},
		{
			name:      "valid component reference",
			input:     `model { API = system "API" { Web = container "Web" { Auth = component "Auth" } } scenario S1 "Test" "Test scenario" { API.Web.Auth -> API.Web.Auth } }`,
			wantError: false,
		},
		{
			name:      "valid person reference",
			input:     `model { User = person "User" scenario S1 "Test" "Test scenario" { User -> User } }`,
			wantError: false,
		},
		{
			name:      "nil program",
			input:     "",
			wantError: false,
		},
		{
			name:      "empty scenario",
			input:     `model { scenario S1 "Test" "Test scenario" { } }`,
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

	input := `model {
		scenario S1 "Test" {
			Unknown -> Unknown
		}
	}`

	program, _, err := parser.Parse("test.sruja", input)
	if err != nil {
		t.Fatalf("Parse error: %v", err)
	}

	validator := NewValidator()
	validator.RegisterRule(&ScenarioFQNRule{})
	diags := validator.Validate(program)

	hasError := false
	for _, diag := range diags {
		if diag.Severity == diagnostics.SeverityError && strings.Contains(diag.Message, "undefined element") {
			hasError = true
			break
		}
	}

	if !hasError {
		t.Errorf("Expected validation error for undefined reference, but got none. Diagnostics: %v", diags)
	}
}
