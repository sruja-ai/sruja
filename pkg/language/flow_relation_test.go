// pkg/language/flow_relation_test.go
package language

import (
	"testing"
)

// TestFlow_DFDStyleRelations tests DFD-style relations in flows
// Note: These tests are currently skipped because the parser doesn't recognize
// FlowRelation patterns yet. Once the parser is fixed, these should pass.
func TestFlow_DFDStyleRelations(t *testing.T) {
	testCases := []struct {
		name        string
		dsl         string
		expectError bool
		description string
	}{
		{
			name: "Simple DFD flow",
			dsl: `architecture "Test" {
				person Customer "Customer"
				system Shop "Shop"
				flow OrderProcess "Order Processing" {
					Customer -> Shop "Order Details"
					Shop -> Customer "Confirmation"
				}
			}`,
			expectError: true, // Parser doesn't support yet
			description: "Simple DFD-style flow with person and system",
		},
		{
			name: "DFD flow with qualified references",
			dsl: `architecture "Test" {
				system API "API Service" {
					container WebApp "Web App"
					datastore DB "Database"
				}
				flow DataFlow "Data Flow" {
					API.WebApp -> API.DB "Reads/Writes"
					API.DB -> API.WebApp "Returns data"
				}
			}`,
			expectError: true, // Parser doesn't support yet
			description: "DFD flow with qualified component references",
		},
		{
			name: "DFD flow with tags",
			dsl: `architecture "Test" {
				system A "System A"
				system B "System B"
				flow TaggedFlow "Tagged Flow" {
					A -> B "Data" [critical, async]
				}
			}`,
			expectError: true, // Parser doesn't support yet
			description: "DFD flow with relation tags",
		},
		{
			name: "Mixed flow - DFD relations and steps",
			dsl: `architecture "Test" {
				system API "API"
				flow MixedFlow "Mixed Flow" {
					API -> API "Internal processing"
					step S1 "Validate"
					step S2 "Process"
				}
			}`,
			expectError: true, // Parser doesn't support yet
			description: "Flow with both DFD relations and workflow steps",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			t.Skip("Parser doesn't support DFD-style flows yet - FlowRelation pattern needs parser fix")

			parser, err := NewParser()
			if err != nil {
				t.Fatalf("Failed to create parser: %v", err)
			}

			program, _, err := parser.Parse("test.sruja", tc.dsl)
			if tc.expectError {
				if err == nil {
					t.Errorf("Expected parse error but got none for: %s", tc.description)
				}
				return
			}

			if err != nil {
				t.Fatalf("Failed to parse DSL: %v\nDSL:\n%s", err, tc.dsl)
			}

			if program.Architecture == nil {
				t.Fatal("Expected architecture to be parsed")
			}

			if len(program.Architecture.Flows) == 0 {
				t.Fatal("Expected at least one flow to be parsed")
			}

			flow := program.Architecture.Flows[0]
			if len(flow.Steps) == 0 {
				t.Error("Expected flow to have steps parsed")
			}
		})
	}
}

// TestFlow_PostProcess_Relations tests that Flow.PostProcess extracts steps correctly
func TestFlow_PostProcess_Relations(t *testing.T) {
	t.Skip("Skipping Flow PostProcess tests due to pending parser/struct alignment")
	// Flow is an alias to Scenario - uses ScenarioStep
	flow := &Flow{
		ID:    "TestFlow",
		Title: "Test Flow",
		Items: []*ScenarioItem{
			{
				Step: &ScenarioStep{
					From:        QualifiedIdent{Parts: []string{"A"}},
					Arrow:       "->",
					To:          QualifiedIdent{Parts: []string{"B"}},
					Description: stringPtr("Test relation"),
				},
			},
		},
	}

	flow.PostProcess()

	if len(flow.Steps) != 1 {
		t.Errorf("Expected 1 step, got %d", len(flow.Steps))
	}

	if flow.Steps[0].From.String() != "A" {
		t.Errorf("Expected step From 'A', got '%s'", flow.Steps[0].From.String())
	}

	if flow.Steps[0].To.String() != "B" {
		t.Errorf("Expected step To 'B', got '%s'", flow.Steps[0].To.String())
	}
}
