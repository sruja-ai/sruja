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
			dsl: `User = person "User"
    API = system "API Service" {
        WebApp = container "Web Application"
    }
    
    User -> API.WebApp "Uses"`,
			expected: 2, // User -> API.WebApp (explicit) + User -> API (implied)
		},
		{
			name: "Multiple implied relationships",
			dsl: `User = person "User"
    Shop = system "Shop" {
        WebApp = container "Web Application"
        API = container "API Gateway"
    }
    
    User -> Shop.WebApp "Uses"
    Shop.WebApp -> Shop.API "Calls"`,
			expected: 3, // User -> Shop.WebApp, Shop.WebApp -> Shop.API (explicit)
			// + User -> Shop (implied from User -> Shop.WebApp)
			// Note: Shop.WebApp -> Shop.API does NOT imply Shop -> Shop.API
			// because Shop.WebApp is already inside Shop
		},
		{
			name: "No implied if parent exists",
			dsl: `User = person "User"
    API = system "API Service" {
        WebApp = container "Web Application"
    }
    
    User -> API "Uses"
    User -> API.WebApp "Uses"`,
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

			if program == nil || program.Model == nil {
				t.Fatal("Program or Model is nil")
			}

			// Count relations in Model
			actual := 0
			for _, item := range program.Model.Items {
				if item.Relation != nil {
					actual++
				}
			}
			if actual != tt.expected {
				t.Errorf("Expected %d relationships, got %d", tt.expected, actual)
				// Print actual relationships for debugging
				for i, item := range program.Model.Items {
					if item.Relation != nil {
						t.Logf("  Relationship %d: %s -> %s", i+1, item.Relation.From.String(), item.Relation.To.String())
					}
				}
			}
		})
	}
}
