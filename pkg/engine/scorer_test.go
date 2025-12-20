package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestScorer_CalculateScore(t *testing.T) {
	tests := []struct {
		name          string
		dsl           string
		expectedScore int
		expectedGrade string
		expectedRules []string // Rules expected to be violated
	}{
		{
			name: "Perfect Score",
			dsl: `
model {
	Web = container "Web App" {
		description "A web application"
		technology "React"
	}
	API = container "API Service" {
		description "An API service"
		technology "Go"
	}
	Web -> API
}`,
			expectedScore: 100,
			expectedGrade: "A",
			expectedRules: []string{},
		},
		{
			name: "Missing Description",
			dsl: `
model {
	Web = container "Web App" {
		technology "React"
	}
}`,
			expectedScore: 93, // -2 for missing description, -5 for orphan
			expectedGrade: "A",
			expectedRules: []string{"Missing Description", "Orphan Element"},
		},
		{
			name: "Orphan Element",
			dsl: `
model {
	Web = container "Web App" {
		description "Web App"
	}
}`,
			expectedScore: 95, // -5 for orphan
			expectedGrade: "A",
			expectedRules: []string{"Orphan Element"},
		},
		{
			name: "Layer Violation",
			dsl: `
model {
	Web = container "Web App" {
		description "Web App"
		metadata { layer "web" }
	}
	DB = container "Database" {
		description "Database"
		metadata { layer "data" }
	}
	// Violation: Data -> Web
	DB -> Web
}`,
			expectedScore: 90, // -10 for layer violation
			expectedGrade: "A",
			expectedRules: []string{"Layer Violation"},
		},
		{
			name: "Cycle Detection",
			dsl: `
model {
	container A "Service A" { description "A" }
	B = container "Service B" { description "B" }
	A -> B
	B -> A
}`,
			expectedScore: 80, // -20 for cycle
			expectedGrade: "B",
			expectedRules: []string{"Circular Dependency"},
		},
		{
			name: "Invalid Reference",
			dsl: `
model {
	container A "Service A" { description "A" }
	A -> B // B is undefined
}`,
			expectedScore: 90, // -10 for invalid ref
			expectedGrade: "A",
			expectedRules: []string{"Invalid Reference"},
		},
		{
			name: "Multiple Violations",
			dsl: `
model {
	// Orphan (-5) + Missing Description (-2)
	Orphan = container "Orphan" 
}`,
			expectedScore: 93, // 100 - 5 - 2
			expectedGrade: "A",
			expectedRules: []string{"Orphan Element", "Missing Description"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p, err := language.NewParser()
			assert.NoError(t, err)

			program, _, err := p.Parse("test.sruja", tt.dsl)
			require.NoError(t, err)
			require.NotNil(t, program)

			// For InvalidRef, diags might contain the error, but Scorer re-runs validation.
			// Actually Scorer runs validation internally.

			scorer := NewScorer()
			card := scorer.CalculateScore(program)

			assert.Equal(t, tt.expectedScore, card.Score, "Score mismatch")
			assert.Equal(t, tt.expectedGrade, card.Grade, "Grade mismatch")

			// Check if expected rules are present in deductions
			for _, expectedRule := range tt.expectedRules {
				found := false
				for _, d := range card.Deductions {
					if d.Rule == expectedRule {
						found = true
						break
					}
				}
				assert.True(t, found, "Expected rule violation '%s' not found", expectedRule)
			}
		})
	}
}
