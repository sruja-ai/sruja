// pkg/engine/slo_rule_test.go
package engine

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestSLOValidationRule(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	tests := []struct {
		name            string
		input           string
		wantError       bool
		wantWarning     bool
		errorContains   string
		warningContains string
	}{
		{
			name:        "valid SLO block with availability",
			input:       `system API "API Service" { slo { availability { target "99.9%" window "30 days" } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:          "invalid percentage format - missing percent",
			input:         `system API "API Service" { slo { availability { target "99.9" window "30 days" } } }`,
			wantError:     true,
			errorContains: "percentage",
		},
		{
			name:          "invalid percentage format - not a number",
			input:         `system API "API Service" { slo { availability { target "abc%" window "30 days" } } }`,
			wantError:     true,
			errorContains: "percentage",
		},
		{
			name:          "invalid percentage format - decimal without number",
			input:         `system API "API Service" { slo { availability { target ".9%" window "30 days" } } }`,
			wantError:     true,
			errorContains: "percentage",
		},
		{
			name:        "valid percentage formats",
			input:       `system API "API Service" { slo { availability { target "99%" window "30 days" } errorRate { target "0.1%" window "7 days" } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:          "invalid current percentage",
			input:         `system API "API Service" { slo { availability { target "99.9%" window "30 days" current "99" } } }`,
			wantError:     true,
			errorContains: "current",
		},
		{
			name:        "valid current percentage",
			input:       `system API "API Service" { slo { availability { target "99.9%" window "30 days" current "99.95%" } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:            "invalid time window format",
			input:           `system API "API Service" { slo { availability { target "99.9%" window "30" } } }`,
			wantWarning:     true,
			warningContains: "time period",
		},
		{
			name:        "valid time window formats",
			input:       `system API "API Service" { slo { availability { target "99.9%" window "30 days" } latency { p95 "200ms" p99 "500ms" window "7 days" } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:          "invalid duration format - missing unit",
			input:         `system API "API Service" { slo { latency { p95 "200" p99 "500ms" window "7 days" } } }`,
			wantError:     true,
			errorContains: "duration",
		},
		{
			name:          "invalid duration format - invalid unit",
			input:         `system API "API Service" { slo { latency { p95 "200xyz" p99 "500ms" window "7 days" } } }`,
			wantError:     true,
			errorContains: "duration",
		},
		{
			name:        "valid duration formats",
			input:       `system API "API Service" { slo { latency { p95 "200ms" p99 "500ms" window "7 days" } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:        "valid duration with different units",
			input:       `system API "API Service" { slo { latency { p95 "1s" p99 "2s" window "7 days" } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:          "invalid current latency p95",
			input:         `system API "API Service" { slo { latency { p95 "200ms" p99 "500ms" window "7 days" current { p95 "180" p99 "420ms" } } } }`,
			wantError:     true,
			errorContains: "current p95",
		},
		{
			name:          "invalid current latency p99",
			input:         `system API "API Service" { slo { latency { p95 "200ms" p99 "500ms" window "7 days" current { p95 "180ms" p99 "420" } } } }`,
			wantError:     true,
			errorContains: "current p99",
		},
		{
			name:        "valid current latency",
			input:       `system API "API Service" { slo { latency { p95 "200ms" p99 "500ms" window "7 days" current { p95 "180ms" p99 "420ms" } } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:          "invalid error rate target",
			input:         `system API "API Service" { slo { errorRate { target "0.1" window "7 days" } } }`,
			wantError:     true,
			errorContains: "percentage",
		},
		{
			name:          "invalid error rate current",
			input:         `system API "API Service" { slo { errorRate { target "0.1%" window "7 days" current "0.08" } } }`,
			wantError:     true,
			errorContains: "current",
		},
		{
			name:        "valid error rate",
			input:       `system API "API Service" { slo { errorRate { target "0.1%" window "7 days" current "0.08%" } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:            "invalid throughput target format",
			input:           `system API "API Service" { slo { throughput { target "10000" window "peak hour" } } }`,
			wantWarning:     true,
			warningContains: "rate",
		},
		{
			name:            "invalid throughput current format",
			input:           `system API "API Service" { slo { throughput { target "10000 req/s" window "peak hour" current "8500" } } }`,
			wantWarning:     true,
			warningContains: "rate",
		},
		{
			name:        "valid throughput formats",
			input:       `system API "API Service" { slo { throughput { target "10000 req/s" window "1 hour" current "8500 req/s" } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:        "valid throughput without unit",
			input:       `system API "API Service" { slo { throughput { target "10000/s" window "1 hour" } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:            "empty SLO block - no SLO types",
			input:           `system API "API Service" { slo { } }`,
			wantWarning:     true,
			warningContains: "at least one SLO type",
		},
		{
			name:        "valid SLO with all types",
			input:       `system API "API Service" { slo { availability { target "99.9%" window "30 days" } latency { p95 "200ms" p99 "500ms" window "7 days" } errorRate { target "0.1%" window "7 days" } throughput { target "10000 req/s" window "1 hour" } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:        "valid SLO in container",
			input:       `system API "API Service" { container Web "Web Server" { slo { availability { target "99.9%" window "30 days" } } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:          "invalid SLO in container",
			input:         `system API "API Service" { container Web "Web Server" { slo { availability { target "99.9" window "30 days" } } } }`,
			wantError:     true,
			errorContains: "percentage",
		},
	}

	rule := &SLOValidationRule{}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			program, _, err := parser.Parse("test.sruja", tt.input)
			if err != nil {
				t.Fatalf("Parse error: %v", err)
			}

			diags := rule.Validate(program)
			hasError := false
			hasWarning := false
			errorMessages := []string{}
			warningMessages := []string{}

			for _, diag := range diags {
				if diag.Severity == diagnostics.SeverityError {
					hasError = true
					errorMessages = append(errorMessages, diag.Message)
				}
				if diag.Severity == diagnostics.SeverityWarning {
					hasWarning = true
					warningMessages = append(warningMessages, diag.Message)
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

			if tt.wantWarning && !hasWarning {
				t.Errorf("Expected validation warning but got none. Diagnostics: %v", diags)
			}
			if !tt.wantWarning && hasWarning {
				t.Errorf("Unexpected validation warning: %v", diags)
			}
			if tt.warningContains != "" && hasWarning {
				found := false
				for _, msg := range warningMessages {
					if strings.Contains(strings.ToLower(msg), strings.ToLower(tt.warningContains)) {
						found = true
						break
					}
				}
				if !found {
					t.Errorf("Expected warning message to contain '%s', but got: %v", tt.warningContains, warningMessages)
				}
			}
		})
	}
}

func TestSLOEnforcementRule(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	tests := []struct {
		name             string
		input            string
		expectSuggestion bool
		suggestionFor    string // "system" or "container"
	}{
        {
            name: "has SLA requirement but no SLO",
            input: `architecture "A" {
                requirement R1 performance "Must maintain 99.9% availability SLA"
                system API "API Service" {}
            }`,
            expectSuggestion: true,
            suggestionFor:    "system",
        },
        {
            name: "has SLA requirement and SLO",
            input: `architecture "A" {
                requirement R1 performance "Must maintain 99.9% availability SLA"
                system API "API Service" {
                    slo {
                        availability {
                            target "99.9%"
                            window "30 days"
                        }
                    }
                }
            }`,
            expectSuggestion: false,
        },
        {
            name: "no SLA requirement",
            input: `architecture "A" {
                requirement R1 functional "Must handle user login"
                system API "API Service" {}
            }`,
            expectSuggestion: false,
        },
        {
            name: "has availability requirement",
            input: `architecture "A" {
                requirement R1 performance "System must have high availability"
                system API "API Service" {}
            }`,
            expectSuggestion: true,
            suggestionFor:    "system",
        },
        {
            name: "has latency requirement",
            input: `architecture "A" {
                requirement R1 performance "Response time must be under 200ms"
                system API "API Service" {}
            }`,
            expectSuggestion: true,
            suggestionFor:    "system",
        },
        {
            name: "has uptime requirement",
            input: `architecture "A" {
                requirement R1 performance "System uptime must be 99.9%"
                system API "API Service" {}
            }`,
            expectSuggestion: true,
            suggestionFor:    "system",
        },
        {
            name: "has error rate requirement",
            input: `architecture "A" {
                requirement R1 performance "Error rate must be below 0.1%"
                system API "API Service" {}
            }`,
            expectSuggestion: true,
            suggestionFor:    "system",
        },
        {
            name: "has throughput requirement",
            input: `architecture "A" {
                requirement R1 performance "Must handle 10000 requests per second throughput"
                system API "API Service" {}
            }`,
            expectSuggestion: true,
            suggestionFor:    "system",
        },
        {
            name: "has nonfunctional requirement with SLA",
            input: `architecture "A" {
                requirement R1 nonfunctional "Must maintain SLA"
                system API "API Service" {}
            }`,
            expectSuggestion: true,
            suggestionFor:    "system",
        },
        {
            name: "container has SLA requirement but no SLO",
            input: `architecture "A" {
                requirement R1 performance "Must maintain 99.9% availability SLA"
                system API "API Service" {
                    container Web "Web Server" {}
                }
            }`,
            expectSuggestion: true,
            suggestionFor:    "container",
        },
        {
            name: "container has SLA requirement and SLO",
            input: `architecture "A" {
                requirement R1 performance "Must maintain 99.9% availability SLA"
                system API "API Service" {
                    container Web "Web Server" {
                        slo {
                            availability {
                                target "99.9%"
                                window "30 days"
                            }
                        }
                    }
                }
            }`,
            expectSuggestion: true,
            suggestionFor:    "system",
        },
        {
            name: "multiple systems - one with SLA requirement",
            input: `architecture "A" {
                requirement R1 performance "Must maintain 99.9% availability SLA"
                system API "API Service" {}
                system DB "Database" {}
            }`,
            expectSuggestion: true,
            suggestionFor:    "system",
        },
	}

	rule := &SLOEnforcementRule{}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			program, _, err := parser.Parse("test.sruja", tt.input)
			if err != nil {
				t.Fatalf("Parse error: %v", err)
			}

			diags := rule.Validate(program)
			hasSuggestion := false
			suggestionMessages := []string{}

			for _, diag := range diags {
				if diag.Severity == diagnostics.SeverityInfo && diag.Message != "" {
					hasSuggestion = true
					suggestionMessages = append(suggestionMessages, diag.Message)
				}
			}

			if tt.expectSuggestion && !hasSuggestion {
				t.Errorf("Expected suggestion but got none. Diagnostics: %v", diags)
			}
			if !tt.expectSuggestion && hasSuggestion {
				t.Errorf("Unexpected suggestion: %v", diags)
			}
			if tt.expectSuggestion && tt.suggestionFor != "" {
				found := false
				for _, msg := range suggestionMessages {
					if strings.Contains(strings.ToLower(msg), tt.suggestionFor) {
						found = true
						break
					}
				}
				if !found {
					t.Errorf("Expected suggestion for %s but got: %v", tt.suggestionFor, suggestionMessages)
				}
			}
		})
	}
}

func TestSLOValidationRuleName(t *testing.T) {
	rule := &SLOValidationRule{}
	if rule.Name() != "SLO Validation" {
		t.Errorf("Expected name 'SLO Validation', got '%s'", rule.Name())
	}
}

func TestSLOEnforcementRuleName(t *testing.T) {
	rule := &SLOEnforcementRule{}
	if rule.Name() != "SLO Enforcement" {
		t.Errorf("Expected name 'SLO Enforcement', got '%s'", rule.Name())
	}
}

func TestSLOValidationRule_IntegrationWithValidator(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	tests := []struct {
		name        string
		input       string
		wantError   bool
		wantWarning bool
	}{
		{
			name:        "valid SLO passes validation",
			input:       `system API "API Service" { slo { availability { target "99.9%" window "30 days" } } }`,
			wantError:   false,
			wantWarning: false,
		},
		{
			name:        "invalid SLO caught by validator",
			input:       `system API "API Service" { slo { availability { target "99.9" window "30 days" } } }`,
			wantError:   true,
			wantWarning: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			program, _, err := parser.Parse("test.sruja", tt.input)
			if err != nil {
				t.Fatalf("Parse error: %v", err)
			}

			validator := NewValidator()
			validator.RegisterRule(&SLOValidationRule{})
			diags := validator.Validate(program)

			hasError := false
			hasWarning := false
			for _, diag := range diags {
				if diag.Severity == diagnostics.SeverityError {
					hasError = true
				}
				if diag.Severity == diagnostics.SeverityWarning {
					hasWarning = true
				}
			}

			if tt.wantError && !hasError {
				t.Errorf("Expected validation error but got none. Diagnostics: %v", diags)
			}
			if !tt.wantError && hasError {
				t.Errorf("Unexpected validation error: %v", diags)
			}
			if tt.wantWarning && !hasWarning {
				t.Errorf("Expected validation warning but got none. Diagnostics: %v", diags)
			}
			if !tt.wantWarning && hasWarning {
				t.Errorf("Unexpected validation warning: %v", diags)
			}
		})
	}
}

func TestSLOEnforcementRule_IntegrationWithValidator(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	tests := []struct {
		name             string
		input            string
		expectSuggestion bool
	}{
        {
            name: "enforcement rule suggests SLO when SLA requirement exists",
            input: `architecture "A" {
                requirement R1 performance "Must maintain 99.9% availability SLA"
                system API "API Service" {}
            }`,
            expectSuggestion: true,
        },
        {
            name: "enforcement rule does not suggest when SLO already exists",
            input: `architecture "A" {
                requirement R1 performance "Must maintain 99.9% availability SLA"
                system API "API Service" {
                    slo {
                        availability {
                            target "99.9%"
                            window "30 days"
                        }
                    }
                }
            }`,
            expectSuggestion: false,
        },
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			program, _, err := parser.Parse("test.sruja", tt.input)
			if err != nil {
				t.Fatalf("Parse error: %v", err)
			}

			validator := NewValidator()
			validator.RegisterRule(&SLOEnforcementRule{})
			diags := validator.Validate(program)

			hasSuggestion := false
			for _, diag := range diags {
				if diag.Severity == diagnostics.SeverityInfo && strings.Contains(diag.Message, "SLO") {
					hasSuggestion = true
					break
				}
			}

			if tt.expectSuggestion && !hasSuggestion {
				t.Errorf("Expected suggestion but got none. Diagnostics: %v", diags)
			}
			if !tt.expectSuggestion && hasSuggestion {
				t.Errorf("Unexpected suggestion: %v", diags)
			}
		})
	}
}

func TestSLOValidationAndEnforcementRules_Together(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	// Test that both rules work together
    input := `architecture "A" {
        requirement R1 performance "Must maintain 99.9% availability SLA"
        system API "API Service" {
            slo {
                availability {
                    target "99.9"
                    window "30 days"
                }
            }
        }
    }`

	program, _, err := parser.Parse("test.sruja", input)
	if err != nil {
		t.Fatalf("Parse error: %v", err)
	}

	validator := NewValidator()
	validator.RegisterRule(&SLOValidationRule{})
	validator.RegisterRule(&SLOEnforcementRule{})
	diags := validator.Validate(program)

	// Should have validation error (invalid percentage format)
	// Should NOT have suggestion (SLO block already exists)
	hasError := false
	hasSuggestion := false

	for _, diag := range diags {
		if diag.Severity == diagnostics.SeverityError {
			hasError = true
		}
		if diag.Severity == diagnostics.SeverityInfo && strings.Contains(diag.Message, "SLO") {
			hasSuggestion = true
		}
	}

	if !hasError {
		t.Errorf("Expected validation error for invalid percentage format, but got none. Diagnostics: %v", diags)
	}
	if hasSuggestion {
		t.Errorf("Should not suggest SLO when SLO block already exists. Diagnostics: %v", diags)
	}
}

func TestSLOValidationHelperFunctions(t *testing.T) {
	rule := &SLOValidationRule{}

	tests := []struct {
		name     string
		function func(string) bool
		value    string
		want     bool
	}{
		// isValidPercentage tests
		{
			name:     "valid percentage - integer",
			function: rule.isValidPercentage,
			value:    "99%",
			want:     true,
		},
		{
			name:     "valid percentage - decimal",
			function: rule.isValidPercentage,
			value:    "99.9%",
			want:     true,
		},
		{
			name:     "valid percentage - multiple decimals",
			function: rule.isValidPercentage,
			value:    "99.99%",
			want:     true,
		},
		{
			name:     "invalid percentage - missing percent",
			function: rule.isValidPercentage,
			value:    "99.9",
			want:     false,
		},
		{
			name:     "invalid percentage - not a number",
			function: rule.isValidPercentage,
			value:    "abc%",
			want:     false,
		},
		{
			name:     "invalid percentage - empty",
			function: rule.isValidPercentage,
			value:    "",
			want:     false,
		},
		{
			name:     "invalid percentage - just percent",
			function: rule.isValidPercentage,
			value:    "%",
			want:     false,
		},
		// isValidDuration tests
		{
			name:     "valid duration - milliseconds",
			function: rule.isValidDuration,
			value:    "200ms",
			want:     true,
		},
		{
			name:     "valid duration - seconds",
			function: rule.isValidDuration,
			value:    "1s",
			want:     true,
		},
		{
			name:     "valid duration - minutes",
			function: rule.isValidDuration,
			value:    "5m",
			want:     true,
		},
		{
			name:     "valid duration - hours",
			function: rule.isValidDuration,
			value:    "2h",
			want:     true,
		},
		{
			name:     "valid duration - decimal",
			function: rule.isValidDuration,
			value:    "1.5s",
			want:     true,
		},
		{
			name:     "invalid duration - missing unit",
			function: rule.isValidDuration,
			value:    "200",
			want:     false,
		},
		{
			name:     "invalid duration - invalid unit",
			function: rule.isValidDuration,
			value:    "200xyz",
			want:     false,
		},
		{
			name:     "invalid duration - empty",
			function: rule.isValidDuration,
			value:    "",
			want:     false,
		},
		// isValidTimeWindow tests
		{
			name:     "valid time window - days",
			function: rule.isValidTimeWindow,
			value:    "30 days",
			want:     true,
		},
		{
			name:     "valid time window - day",
			function: rule.isValidTimeWindow,
			value:    "1 day",
			want:     true,
		},
		{
			name:     "valid time window - hours",
			function: rule.isValidTimeWindow,
			value:    "24 hours",
			want:     true,
		},
		{
			name:     "valid time window - hour",
			function: rule.isValidTimeWindow,
			value:    "1 hour",
			want:     true,
		},
		{
			name:     "valid time window - weeks",
			function: rule.isValidTimeWindow,
			value:    "2 weeks",
			want:     true,
		},
		{
			name:     "valid time window - months",
			function: rule.isValidTimeWindow,
			value:    "6 months",
			want:     true,
		},
		{
			name:     "invalid time window - missing unit",
			function: rule.isValidTimeWindow,
			value:    "30",
			want:     false,
		},
		{
			name:     "invalid time window - invalid unit",
			function: rule.isValidTimeWindow,
			value:    "30 years",
			want:     false,
		},
		{
			name:     "invalid time window - empty",
			function: rule.isValidTimeWindow,
			value:    "",
			want:     false,
		},
		// isValidRate tests
		{
			name:     "valid rate - with unit",
			function: rule.isValidRate,
			value:    "10000 req/s",
			want:     true,
		},
		{
			name:     "valid rate - without unit",
			function: rule.isValidRate,
			value:    "1000/s",
			want:     true,
		},
		{
			name:     "valid rate - different unit",
			function: rule.isValidRate,
			value:    "500 messages/min",
			want:     true,
		},
		{
			name:     "invalid rate - missing slash",
			function: rule.isValidRate,
			value:    "10000 req",
			want:     false,
		},
		{
			name:     "invalid rate - missing number",
			function: rule.isValidRate,
			value:    "req/s",
			want:     false,
		},
		{
			name:     "invalid rate - empty",
			function: rule.isValidRate,
			value:    "",
			want:     false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := tt.function(tt.value)
			if got != tt.want {
				t.Errorf("isValid function for '%s' = %v, want %v", tt.value, got, tt.want)
			}
		})
	}
}
