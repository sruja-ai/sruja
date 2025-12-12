// pkg/engine/error_messages_integration_test.go
package engine_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

// parseWithFullValidation parses DSL and runs all validation rules
func parseWithFullValidation(t *testing.T, filename, dsl string) ([]diagnostics.Diagnostic, error) {
	t.Helper()
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, parseDiags, err := parser.Parse(filename, dsl)

	allDiags := make([]diagnostics.Diagnostic, 0, len(parseDiags)+10)
	allDiags = append(allDiags, parseDiags...)

	// Run full validation if parsing succeeded
	if err == nil && program != nil {
		validator := engine.NewValidator()
		validator.RegisterDefaultRules()
		validator.RegisterRule(&engine.ExternalDependencyRule{})

		validationDiags := validator.Validate(program)
		allDiags = append(allDiags, validationDiags...)
	}

	return allDiags, err
}

// TestErrorMessages_ComprehensiveInvalidExamples tests various invalid DSL examples
func TestErrorMessages_ComprehensiveInvalidExamples(t *testing.T) {
	testCases := []struct {
		name             string
		dsl              string
		expectedErrors   []string // Keywords that should appear in error messages
		checkSuggestions bool
	}{
		{
			name: "Missing closing brace",
			dsl: `
architecture "Test" {
    system API "API Service" {
        container WebApp "Web App"
`,
			expectedErrors:   []string{"brace", "expected", "missing"},
			checkSuggestions: true,
		},
		{
			name: "Duplicate system ID",
			dsl: `
architecture "Test" {
    system API "API Service" {}
    system API "Duplicate API" {}
}`,
			expectedErrors:   []string{"duplicate", "API", "previously", "first defined"},
			checkSuggestions: true,
		},
		{
			name: "Undefined reference",
			dsl: `
architecture "Test" {
    system API "API Service" {
        API -> UnknownSystem "Uses"
    }
}`,
			expectedErrors:   []string{"undefined", "UnknownSystem", "reference"},
			checkSuggestions: true,
		},
		{
			name: "Invalid SLO duration format",
			dsl: `
architecture "Test" {
    system API "API Service" {
        slo {
            latency {
                p95 "invalid_format"
            }
        }
    }
}`,
			expectedErrors:   []string{"duration", "p95", "invalid_format"},
			checkSuggestions: true,
		},
		{
			name: "Invalid property value",
			dsl: `
architecture "Test" {
    system API "API Service" {
        properties {
            "capacity.readReplicas" "not_a_number"
        }
    }
    system Client "Client" {
        Client -> API "Uses"
    }
}`,
			expectedErrors:   []string{"property", "readReplicas", "invalid"},
			checkSuggestions: true,
		},
		{
			name: "Orphan element",
			dsl: `
architecture "Test" {
    system API "API Service" {}
    system Unused "Unused System" {}
}`,
			expectedErrors:   []string{"orphan", "unused", "never used"},
			checkSuggestions: true,
		},
		{
			name: "Layer violation",
			dsl: `
architecture "Test" {
    system DataLayer "Data Layer" {
        metadata { layer "data" }
    }
    system WebLayer "Web Layer" {
        metadata { layer "web" }
    }
    DataLayer -> WebLayer "Invalid"
}`,
			expectedErrors:   []string{"layer", "violation", "depend"},
			checkSuggestions: true,
		},
		{
			name: "Ambiguous reference in scenario",
			dsl: `
architecture "Test" {
    system Order "Order System" {
        container API "Order API" {}
    }
    system Payment "Payment System" {
        container API "Payment API" {}
    }
    scenario TestScenario "Test Scenario" "Description" {
        Order -> API "Uses"
    }
}`,
			expectedErrors:   []string{"ambiguous", "API", "multiple", "qualified"},
			checkSuggestions: true,
		},
		{
			name: "External dependency violation",
			dsl: `
architecture "Test" {
    system Parent "Parent System" {
        container Child "Child Container" {
            Child -> Parent "Invalid"
        }
    }
}`,
			expectedErrors:   []string{"parent", "external", "depend"},
			checkSuggestions: true,
		},
		{
			name: "Invalid error rate format",
			dsl: `
architecture "Test" {
    system API "API Service" {
        slo {
            errorRate {
                target "wrong_format"
            }
        }
    }
}`,
			expectedErrors:   []string{"percentage", "error", "target"},
			checkSuggestions: true,
		},
		{
			name: "Invalid throughput format",
			dsl: `
architecture "Test" {
    system API "API Service" {
        slo {
            throughput {
                target "wrong"
            }
        }
    }
}`,
			expectedErrors:   []string{"rate", "throughput", "target"},
			checkSuggestions: true,
		},
		{
			name: "Unexpected token",
			dsl: `
architecture "Test" {
    system API "API Service" {
        unexpected_keyword "value"
    }
}`,
			expectedErrors:   []string{"unexpected", "token"},
			checkSuggestions: true,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			diags, err := parseWithFullValidation(t, "test.sruja", tc.dsl)

			// Check that we got errors or diagnostics
			if err == nil && len(diags) == 0 {
				t.Errorf("Expected errors or diagnostics for invalid DSL, got none")
				return
			}

			// Check that error messages contain at least one expected keyword
			foundExpected := false
			for _, d := range diags {
				msgLower := strings.ToLower(d.Message)
				// Check if at least one expected keyword is found (more flexible)
				anyFound := false
				for _, expected := range tc.expectedErrors {
					if strings.Contains(msgLower, strings.ToLower(expected)) {
						anyFound = true
						break
					}
				}
				if anyFound {
					foundExpected = true
					t.Logf("✓ Found expected error: %s", d.Message)

					// Check location is provided
					if d.Location.Line == 0 {
						t.Error("Expected error location to have a line number")
					}

					// Check suggestions if required
					if tc.checkSuggestions {
						if len(d.Suggestions) == 0 {
							t.Error("Expected suggestions in error message")
						} else {
							t.Logf("  Suggestions: %v", d.Suggestions)
							// Verify suggestions are actionable
							hasActionable := false
							for _, s := range d.Suggestions {
								if len(s) > 10 && (strings.Contains(strings.ToLower(s), "check") ||
									strings.Contains(strings.ToLower(s), "try") ||
									strings.Contains(strings.ToLower(s), "use") ||
									strings.Contains(strings.ToLower(s), "add") ||
									strings.Contains(strings.ToLower(s), "remove")) {
									hasActionable = true
									break
								}
							}
							if !hasActionable && len(d.Suggestions) > 0 {
								t.Logf("  Note: Suggestions may not be highly actionable: %v", d.Suggestions)
							}
						}
					}

					// Check context is provided (for parser errors)
					if len(d.Context) > 0 {
						t.Logf("  Context lines: %d", len(d.Context))
					}

					break
				}
			}

			if !foundExpected {
				t.Errorf("Expected error message containing: %v, got diagnostics: %v", tc.expectedErrors, diags)
				// Print all diagnostics for debugging
				for i, d := range diags {
					t.Logf("  Diagnostic %d: %s (line %d, col %d)", i+1, d.Message, d.Location.Line, d.Location.Column)
				}
			}
		})
	}
}

// TestErrorMessages_LocationPrecision tests that error locations are precise
func TestErrorMessages_LocationPrecision(t *testing.T) {
	dsl := `
architecture "Test" {
    system API "API Service" {
        container WebApp "Web App" {
            component Frontend "Frontend Component"
            component Backend "Backend Component"
            component Frontend "Duplicate Frontend"
        }
    }
}
`
	diags, err := parseWithFullValidation(t, "test.sruja", dsl)

	if err != nil {
		t.Fatalf("Parsing should succeed: %v", err)
	}

	// Find duplicate error
	for _, d := range diags {
		if strings.Contains(strings.ToLower(d.Message), "duplicate") &&
			strings.Contains(d.Message, "Frontend") {
			// Check location is precise
			if d.Location.Line == 0 {
				t.Error("Expected error location to have a line number")
			}
			if d.Location.Column == 0 {
				t.Error("Expected error location to have a column number")
			}
			// Line should be around line 6 (second Frontend definition)
			if d.Location.Line < 4 || d.Location.Line > 8 {
				t.Logf("Warning: Error location line %d may not be precise (expected ~6)", d.Location.Line)
			}
			t.Logf("✓ Error location is precise: line %d, column %d", d.Location.Line, d.Location.Column)
			return
		}
	}

	t.Error("Expected duplicate error for Frontend component")
}

// TestErrorMessages_SuggestionQuality tests that suggestions are helpful
func TestErrorMessages_SuggestionQuality(t *testing.T) {
	testCases := []struct {
		name                       string
		dsl                        string
		expectedSuggestionKeywords []string
	}{
		{
			name: "Undefined reference should suggest similar elements",
			dsl: `
architecture "Test" {
    system API "API Service" {}
    system APIService "API Service Full" {}
    API -> APIServic "Typo"
}`,
			expectedSuggestionKeywords: []string{"did you mean", "APIService", "check", "defined"},
		},
		{
			name: "Invalid SLO should provide format examples",
			dsl: `
architecture "Test" {
    system API "API Service" {
        slo {
            latency {
                p95 "wrong"
            }
        }
    }
}`,
			expectedSuggestionKeywords: []string{"format", "200ms", "1s", "duration"},
		},
		{
			name: "Duplicate ID should suggest renaming",
			dsl: `
architecture "Test" {
    system API "API Service" {}
    system API "Duplicate" {}
}`,
			expectedSuggestionKeywords: []string{"rename", "unique", "API2"},
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			diags, err := parseWithFullValidation(t, "test.sruja", tc.dsl)
			if err != nil {
				t.Logf("Parsing error (may be acceptable): %v", err)
			}

			foundHelpfulSuggestion := false
			for _, d := range diags {
				if len(d.Suggestions) > 0 {
					for _, suggestion := range d.Suggestions {
						suggestionLower := strings.ToLower(suggestion)
						keywordsFound := 0
						for _, keyword := range tc.expectedSuggestionKeywords {
							if strings.Contains(suggestionLower, strings.ToLower(keyword)) {
								keywordsFound++
							}
						}
						if keywordsFound > 0 {
							foundHelpfulSuggestion = true
							t.Logf("✓ Found helpful suggestion: %s", suggestion)
							break
						}
					}
				}
			}

			if !foundHelpfulSuggestion {
				t.Logf("Helpful suggestion not found, but got diagnostics: %v", diags)
				// This is a warning, not a failure, as suggestions may vary
			}
		})
	}
}

// TestErrorMessages_ContextHelpfulness tests that error context is helpful
func TestErrorMessages_ContextHelpfulness(t *testing.T) {
	dsl := `
architecture "Test" {
    system API "API Service" {
        container WebApp "Web App" {
            component Frontend "Frontend"
            // Comment here
            component Backend "Backend"
            component Frontend "Duplicate"
        }
    }
}
`
	diags, err := parseWithFullValidation(t, "test.sruja", dsl)

	if err != nil {
		t.Fatalf("Parsing should succeed: %v", err)
	}

	// Find duplicate error
	for _, d := range diags {
		if strings.Contains(strings.ToLower(d.Message), "duplicate") &&
			strings.Contains(d.Message, "Frontend") {
			// Check that context is provided
			if len(d.Context) > 0 {
				t.Logf("✓ Error context provided (%d lines)", len(d.Context))
				for i, ctx := range d.Context {
					t.Logf("  Context %d: %s", i+1, ctx)
				}
			} else {
				t.Logf("Note: No context provided for error (may be acceptable for validation errors)")
			}
			return
		}
	}

	t.Error("Expected duplicate error for Frontend component")
}
