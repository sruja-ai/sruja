package language

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParser_Library_Keyword(t *testing.T) {
	input := `
library Compliance "Standard Compliance Policies" {
  description "A collection of standard compliance policies."
  
  policy GDPR_Encryption "GDPR Data Encryption" {
    category "security"
    enforcement "required"
  }
}
`
	parser, err := NewParser()
	require.NoError(t, err)

	program, _, err := parser.Parse("compliance.sruja", input)
	require.NoError(t, err)
	require.NotNil(t, program)
	require.NotNil(t, program.Architecture)
	require.Len(t, program.Architecture.Libraries, 1)

	lib := program.Architecture.Libraries[0]
	assert.Equal(t, "Compliance", lib.ID)
	assert.Equal(t, "Standard Compliance Policies", lib.Label)
	assert.Len(t, lib.Policies, 1)
	assert.Equal(t, "GDPR_Encryption", lib.Policies[0].ID)
}
