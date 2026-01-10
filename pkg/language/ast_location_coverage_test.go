package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/stretchr/testify/assert"
)

func TestLocation_Coverage(t *testing.T) {
	// DeploymentNode
	assert.NotNil(t, (&language.DeploymentNode{}).Location())
	assert.NotNil(t, (&language.InfrastructureNode{}).Location())
	assert.NotNil(t, (&language.ContainerInstance{}).Location())

	// Elements
	assert.NotNil(t, (&language.System{}).Location())
	assert.NotNil(t, (&language.Container{}).Location())
	assert.NotNil(t, (&language.Component{}).Location())
	assert.NotNil(t, (&language.DataStore{}).Location())
	assert.NotNil(t, (&language.Queue{}).Location())
	assert.NotNil(t, (&language.Person{}).Location())

	// Core
	assert.NotNil(t, (&language.MetaEntry{}).Location())
	assert.NotNil(t, (&language.MetadataBlock{}).Location())
	assert.NotNil(t, (&language.StyleDecl{}).Location())
	assert.NotNil(t, (&language.StyleBlock{}).Location())
	assert.NotNil(t, (&language.OverviewBlock{}).Location())
	assert.NotNil(t, (&language.ScaleBlock{}).Location())
	assert.NotNil(t, (&language.SLOBlock{}).Location())
	assert.NotNil(t, (&language.ConstraintEntry{}).Location())
	assert.NotNil(t, (&language.ConstraintsBlock{}).Location())
	assert.NotNil(t, (&language.ConventionEntry{}).Location())
	assert.NotNil(t, (&language.ConventionsBlock{}).Location())

	// Model
	assert.NotNil(t, (&language.ImportStatement{}).Location())
	assert.NotNil(t, (&language.Requirement{}).Location())
	assert.NotNil(t, (&language.ADR{}).Location())
}
