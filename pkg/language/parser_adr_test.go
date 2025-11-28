package language

import (
	"testing"
)

func TestParser_ADR(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected *ADR
	}{
		{
			name:  "Simple ADR",
			input: `adr ADR001 "Simple Decision"`,
			expected: &ADR{
				ID:    "ADR001",
				Title: stringPtr("Simple Decision"),
			},
		},
		{
			name: "Full ADR",
			input: `adr ADR002 "Complex Decision" {
				status "Accepted"
				context "We need X"
				decision "We chose Y"
				consequences "Z happened"
			}`,
			expected: &ADR{
				ID:    "ADR002",
				Title: stringPtr("Complex Decision"),
				Body: &ADRBody{
					Status:       stringPtr("Accepted"),
					Context:      stringPtr("We need X"),
					Decision:     stringPtr("We chose Y"),
					Consequences: stringPtr("Z happened"),
				},
			},
		},
		{
			name:  "Reference ADR",
			input: `adr ADR003`,
			expected: &ADR{
				ID:    "ADR003",
				Title: nil,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Wrap in architecture for parsing
			dsl := "architecture \"Test\" {\n" + tt.input + "\n}"
			parser, err := NewParser()
			if err != nil {
				t.Fatalf("Failed to create parser: %v", err)
			}

			program, err := parser.Parse("test.sruja", dsl)
			if err != nil {
				t.Fatalf("Failed to parse DSL: %v", err)
			}

			// Find the ADR in items (since PostProcess might move them to .ADRs, but we check raw items first or check .ADRs if PostProcess ran)
			// The parser populates Items.
			// Let's check Items first.

			var foundADR *ADR
			for _, item := range program.Architecture.Items {
				if item.ADR != nil {
					foundADR = item.ADR
					break
				}
			}

			if foundADR == nil {
				t.Fatalf("Expected ADR in items, got nil")
			}

			if foundADR.ID != tt.expected.ID {
				t.Errorf("Expected ID %q, got %q", tt.expected.ID, foundADR.ID)
			}
			if tt.expected.Title == nil {
				if foundADR.Title != nil {
					t.Errorf("Expected nil Title, got %q", *foundADR.Title)
				}
			} else {
				if foundADR.Title == nil {
					t.Errorf("Expected Title %q, got nil", *tt.expected.Title)
				} else if *foundADR.Title != *tt.expected.Title {
					t.Errorf("Expected Title %q, got %q", *tt.expected.Title, *foundADR.Title)
				}
			}

			if tt.expected.Body == nil {
				if foundADR.Body != nil {
					t.Errorf("Expected nil Body, got %v", foundADR.Body)
				}
			} else {
				if foundADR.Body == nil {
					t.Fatalf("Expected Body, got nil")
				}
				if *foundADR.Body.Status != *tt.expected.Body.Status {
					t.Errorf("Expected Status %q, got %q", *tt.expected.Body.Status, *foundADR.Body.Status)
				}
				if *foundADR.Body.Context != *tt.expected.Body.Context {
					t.Errorf("Expected Context %q, got %q", *tt.expected.Body.Context, *foundADR.Body.Context)
				}
				if *foundADR.Body.Decision != *tt.expected.Body.Decision {
					t.Errorf("Expected Decision %q, got %q", *tt.expected.Body.Decision, *foundADR.Body.Decision)
				}
				if *foundADR.Body.Consequences != *tt.expected.Body.Consequences {
					t.Errorf("Expected Consequences %q, got %q", *tt.expected.Body.Consequences, *foundADR.Body.Consequences)
				}
			}
		})
	}
}

func stringPtr(s string) *string {
	return &s
}
