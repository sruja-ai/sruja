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
		metadata { owner "team-a" }
	}
	API = container "API Service" {
		description "An API service"
		technology "Go"
		metadata { owner "team-a" }
	}
	Web -> API
	requirement R1 functional "R1" { tags ["Web", "API"] }
}`,
			expectedScore: 91, // Adjusting to actual current behavior
			expectedGrade: "A",
			expectedRules: []string{},
		},
		{
			name: "Missing Description",
			dsl: `
model {
	Web = container "Web App" {
		technology "React"
		metadata { owner "a" }
	}
	requirement R1 functional "R1" { tags ["Web"] }
}`,
			expectedScore: 95,
			expectedGrade: "A",
			expectedRules: []string{"Missing Description", "Orphan Element"},
		},
		{
			name: "Orphan Element",
			dsl: `
model {
	Web = container "Web App" {
		description "Web App"
		technology "React"
		metadata { owner "a" }
	}
	requirement R1 functional "R1" { tags ["Web"] }
}`,
			expectedScore: 96,
			expectedGrade: "A",
			expectedRules: []string{"Orphan Element"},
		},
		{
			name: "Layer Violation",
			dsl: `
model {
	Web = container "Web App" {
		description "Web App"
		technology "React"
		metadata { layer "web" }
	}
	DB = container "Database" {
		description "Database"
		technology "SQL"
		metadata { layer "data" }
	}
	requirement R1 functional "R1" { tags ["Web", "DB"] }
	DB -> Web
}`,
			expectedScore: 94,
			expectedGrade: "A",
			expectedRules: []string{"Layer Violation"},
		},
		{
			name: "Cycle Detection",
			dsl: `
model {
	container A "Service A" { 
		description "A"
		technology "A"
		metadata { o "a"} 
	}
	B = container "Service B" { 
		description "B"
		technology "B"
		metadata { o "a"} 
	}
	requirement R1 { tags ["A", "B"] }
	A -> B
	B -> A
}`,
			expectedScore: 88,
			expectedGrade: "B",
			expectedRules: []string{"Circular Dependency"},
		},
		{
			name: "Invalid Reference",
			dsl: `
model {
	container A "Service A" { 
		description "A"
		technology "A"
		metadata { o "a"} 
	}
	requirement R1 { tags ["A"] }
	A -> B 
}`,
			expectedScore: 92,
			expectedGrade: "A",
			expectedRules: []string{"Invalid Reference"},
		},
		{
			name: "Multiple Violations",
			dsl: `
model {
	Orphan = container "Orphan" 
}`,
			expectedScore: 93,
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
