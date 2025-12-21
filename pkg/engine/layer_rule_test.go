package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestLayerViolationRule_Validate(t *testing.T) {
	tests := []struct {
		name     string
		dsl      string
		expected int // Number of expected errors
	}{
		{
			name: "Valid Layering (Web -> API)",
			dsl: `
				model {
					WebApp = container "Web" {
						metadata { layer "web" }
					}
					APIService = container "API" {
						metadata { layer "api" }
					}
					WebApp -> APIService
				}
			`,
			expected: 0,
		},
		{
			name: "Invalid Layering (API -> Web)",
			dsl: `
				model {
					WebApp = container "Web" {
						metadata { layer "web" }
					}
					APIService = container "API" {
						metadata { layer "api" }
					}
					APIService -> WebApp
				}
			`,
			expected: 1,
		},
		{
			name: "Valid Layering (Name Convention)",
			dsl: `
				model {
					MyWeb = container "Web"
					MyAPI = container "API"
					MyWeb -> MyAPI
				}
			`,
			expected: 0,
		},
		{
			name: "Invalid Layering (Name Convention)",
			dsl: `
				model {
					MyWeb = container "Web"
					MyAPI = container "API"
					MyAPI -> MyWeb
				}
			`,
			expected: 1,
		},
		{
			name: "Unknown Layers (Ignored)",
			dsl: `
				model {
					A = container "A"
					B = container "B"
					A -> B
				}
			`,
			expected: 0,
		},
		{
			name: "Relaxed Layering (Web -> Data)",
			dsl: `
				model {
					WebApp = container "Web" {
						metadata { layer "web" }
					}
					DB = container "Database" {
						metadata { layer "data" }
					}
					WebApp -> DB
				}
			`,
			expected: 0, // Skipping layers is allowed in this rule implementation
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			parser, _ := language.NewParser()
			program, diags, err := parser.Parse("test.sruja", tt.dsl)
			if err != nil {
				t.Fatalf("Failed to parse DSL: %v", err)
			}
			if len(diags) > 0 {
				t.Fatalf("Parser reported diagnostics: %v", diags)
			}
			if program == nil {
				t.Fatalf("Program is nil")
			}

			rule := &LayerViolationRule{}
			validationDiags := rule.Validate(program)

			count := 0
			for _, d := range validationDiags {
				if d.Code == diagnostics.CodeLayerViolation {
					count++
				}
			}

			if count != tt.expected {
				t.Errorf("Expected %d layer violations, got %d", tt.expected, count)
				for _, d := range validationDiags {
					t.Logf("Diagnostic: %s", d.Message)
				}
			}
		})
	}
}
