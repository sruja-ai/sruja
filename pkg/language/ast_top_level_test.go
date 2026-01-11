package language

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestProgram_PostProcess_TopLevelItems(t *testing.T) {
	prog := &Program{
		Items: []TopLevelItem{
			{Scenario: &Scenario{ID: "S1"}},
			{Flow: &Flow{ID: "F1"}},
			{Requirement: &Requirement{ID: "R1"}},
			{ADR: &ADR{ID: "A1"}},
			{Policy: &Policy{ID: "P1"}},
			{Deployment: &DeploymentNode{ID: "D1"}},
			{Constraints: &ConstraintsBlock{Entries: []*ConstraintEntry{{Key: "C1"}}}},
			{Conventions: &ConventionsBlock{Entries: []*ConventionEntry{{Key: "V1"}}}},
			{Extend: &ExtendElement{ID: QualifiedIdent{Parts: []string{"E1"}}}},
		},
	}

	prog.PostProcess()

	assert.NotNil(t, prog.Model)
	// 5 behavioral + 1 deployment + 1 constraints + 1 conventions + 1 extend = 9 items
	assert.Len(t, prog.Model.Items, 9)

	var scenarioFound, flowFound, reqFound, adrFound, policyFound bool
	for _, item := range prog.Model.Items {
		if item.Scenario != nil {
			scenarioFound = true
		}
		if item.Flow != nil {
			flowFound = true
		}
		if item.Requirement != nil {
			reqFound = true
		}
		if item.ADR != nil {
			adrFound = true
		}
		if item.Policy != nil {
			policyFound = true
		}
	}

	assert.True(t, scenarioFound)
	assert.True(t, flowFound)
	assert.True(t, reqFound)
	assert.True(t, adrFound)
	assert.True(t, policyFound)
}

func TestModelItem_PostProcess_Internal(_ *testing.T) {
	items := []ModelItem{
		{Scenario: &Scenario{ID: "S1"}},
		{Flow: &Flow{ID: "F1"}},
		{Requirement: &Requirement{ID: "R1"}},
		{ADR: &ADR{ID: "A1"}},
		{Policy: &Policy{ID: "P1"}},
		{DeploymentNode: &DeploymentNode{ID: "D1"}},
	}

	for _, item := range items {
		item.PostProcess()
	}
}
