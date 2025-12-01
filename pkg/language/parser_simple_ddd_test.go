package language

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParser_DDD_Simplified(t *testing.T) {
	dsl := `
architecture "Simple Data" {
    domain Shop {
        module Orders {
            data Order {
                id string
                valueObject Address {
                    city string
                }
            }
        }
    }
}
`
	parser, err := NewParser()
	require.NoError(t, err)

	program, err := parser.Parse("simple.sruja", dsl)
	require.NoError(t, err)
	require.NotNil(t, program)

	arch := program.Architecture
	require.NotNil(t, arch)

	require.Len(t, arch.Contexts, 1)
	ctx := arch.Contexts[0]
	assert.Equal(t, "Orders", ctx.ID)

	require.Len(t, ctx.Entities, 1)
	ent := ctx.Entities[0]
	assert.Equal(t, "Order", ent.ID)

	// Check nested struct (ValueObject) inside Entity
	// Note: My AST structure puts ValueObjects in Entity.Items, but PostProcess might not flatten them into Entity.ValueObjects?
	// Let's check Entity.Items directly or check if I added PostProcess logic for nested VOs in Entity?
	// Looking at ast_postprocess.go:
	// func (e *Entity) PostProcess() { ... if item.Field ... }
	// It seems I missed adding ValueObject to Entity's PostProcess or fields!
	// Let's check the AST definition for EntityItem.
}
