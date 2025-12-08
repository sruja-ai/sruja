// pkg/language/ast_postprocess_implied_test.go
package language

import (
	"testing"
)

func TestArchitecture_InferImpliedRelationships(t *testing.T) {
	tests := []struct {
		name     string
		dsl      string
		expected int // Expected number of relationships after inference
	}{
		{
			name: "Simple implied relationship",
			dsl: `
architecture "Test" {
    person User "User"
    system API "API Service" {
        container WebApp "Web Application"
    }
    
    User -> API.WebApp "Uses"
}`,
			expected: 2, // User -> API.WebApp (explicit) + User -> API (implied)
		},
		{
			name: "Multiple implied relationships",
			dsl: `
architecture "Test" {
    person User "User"
    system Shop "Shop" {
        container WebApp "Web Application"
        container API "API Gateway"
    }
    
    User -> Shop.WebApp "Uses"
    Shop.WebApp -> Shop.API "Calls"
}`,
			expected: 3, // User -> Shop.WebApp, Shop.WebApp -> Shop.API (explicit)
			// + User -> Shop (implied from User -> Shop.WebApp)
			// Note: Shop.WebApp -> Shop.API does NOT imply Shop -> Shop.API
			// because Shop.WebApp is already inside Shop
		},
		{
			name: "No implied if parent exists",
			dsl: `
architecture "Test" {
    person User "User"
    system API "API Service" {
        container WebApp "Web Application"
    }
    
    User -> API "Uses"
    User -> API.WebApp "Uses"
}`,
			expected: 2, // Both explicit, no duplicate implied
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			parser, err := NewParser()
			if err != nil {
				t.Fatalf("Failed to create parser: %v", err)
			}

			program, _, err := parser.Parse("test.sruja", tt.dsl)
			if err != nil {
				t.Fatalf("Failed to parse: %v", err)
			}

			if program.Architecture == nil {
				t.Fatal("Architecture is nil")
			}

			actual := len(program.Architecture.Relations)
			if actual != tt.expected {
				t.Errorf("Expected %d relationships, got %d", tt.expected, actual)
				// Print actual relationships for debugging
				for i, rel := range program.Architecture.Relations {
					t.Logf("  Relationship %d: %s -> %s", i+1, rel.From.String(), rel.To.String())
				}
			}
		})
	}
}
