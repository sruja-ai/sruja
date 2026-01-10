package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestScenarioValidationRule_Reachability(t *testing.T) {
	dsl := `
		SystemA = system "System A"
		SystemB = system "System B"

		scenario HappyPath "Happy Path" {
			step SystemA -> SystemB "Requests data"
			step SystemA -> MissingSystem "Should fail"
		}
	`
	parser, err := language.NewParser()
	require.NoError(t, err)

	prog, diags, err := parser.Parse("test.sruja", dsl)
	require.NoError(t, err)
	require.Empty(t, diags)

	rule := &ScenarioValidationRule{}
	results := rule.Validate(prog)

	// Should have 1 diagnostic for MissingSystem
	assert.Len(t, results, 1)
	assert.Contains(t, results[0].Message, "MissingSystem")
	assert.Contains(t, results[0].Message, "not found")
}

func TestScenarioValidationRule_PolicyViolation(t *testing.T) {
	dsl := `
		Gateway = system "API Gateway" {
			tags ["external"]
		}
		CustomerDB = database "Customer DB" {
			tags ["database"]
		}

		scenario IllegalAccess "Direct Access" {
			step Gateway -> CustomerDB "Illegal direct access"
		}
	`
	parser, err := language.NewParser()
	require.NoError(t, err)

	prog, diags, err := parser.Parse("test.sruja", dsl)
	require.NoError(t, err)
	require.Empty(t, diags)

	rule := &ScenarioValidationRule{}
	results := rule.Validate(prog)

	// Should have 1 diagnostic for Policy Violation
	assert.Len(t, results, 1)
	assert.Contains(t, results[0].Message, "Security Policy Violation")
	assert.Contains(t, results[0].Message, "External node 'Gateway' cannot talk directly to database 'CustomerDB'")
}

func TestScenarioValidationRule_ComplexTags(t *testing.T) {
	parser, err := language.NewParser()
	require.NoError(t, err)

	rule := &ScenarioValidationRule{}

	// Test external to database policy via Assignment TagRef
	// Correct syntax: TagRef after Title
	dsl1 := `
		Ext = system "External" #external
		DB = database "DB" #database
		scenario S "S" { step Ext -> DB }
	`
	prog1, _, _ := parser.Parse("test1.sruja", dsl1)
	results1 := rule.Validate(prog1)
	assert.Len(t, results1, 1)

	// Test metadata based tagging (supports both 'tag' and 'tags' keys)
	dsl2 := `
		Ext = system "External" { metadata { tag "external" } }
		DB = database "DB" { metadata { tags "database" } }
		scenario S "S" { step Ext -> DB }
	`
	prog2, _, _ := parser.Parse("test2.sruja", dsl2)
	results2 := rule.Validate(prog2)
	assert.Len(t, results2, 1)

	// Test multiple tags in body
	dsl3 := `
		Ext = system "External" { tags ["external", "other"] }
		DB = database "DB" { tags ["database", "secure"] }
		scenario S "S" { step Ext -> DB }
	`
	prog3, _, _ := parser.Parse("test3.sruja", dsl3)
	results3 := rule.Validate(prog3)
	assert.Len(t, results3, 1)
}

func TestScenarioValidationRule_DeepFQN(t *testing.T) {
	dsl := `
		S = system "S" {
			C = container "C" {
				Comp = component "Comp"
			}
		}
		scenario Scen "Scen" {
			step S.C.Comp -> S.C
		}
	`
	parser, err := language.NewParser()
	require.NoError(t, err)
	prog, _, _ := parser.Parse("test.sruja", dsl)
	rule := &ScenarioValidationRule{}
	results := rule.Validate(prog)
	assert.Len(t, results, 0)
}

func TestScenarioValidationRule_InlineSteps(t *testing.T) {
	dsl := `
		SystemA = system "System A" {
			step SystemA -> SystemB "Nested step"
		}
		SystemB = system "System B"
	`
	parser, err := language.NewParser()
	require.NoError(t, err)

	prog, _, err := parser.Parse("test.sruja", dsl)
	require.NoError(t, err)

	rule := &ScenarioValidationRule{}
	results := rule.Validate(prog)

	// Should be valid (SystemB exists)
	assert.Len(t, results, 0)
}
