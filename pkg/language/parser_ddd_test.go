package language

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParser_DDD(t *testing.T) {
	dsl := `
architecture "DDD Shop" {
    domain ECommerce "E-Commerce" {
        description "Online shopping domain"
        
        context OrderManagement "Order Management" {
            description "Handles order processing"
            
            aggregate Order {
                entity OrderLineItem {
                    name string
                    quantity int
                }
                
                valueObject ShippingAddress {
                    street string
                    city string
                }
            }
            
            event OrderCreated {
                description "Emitted when a new order is placed"
                orderId string
            }
        }
    }
}
`
	parser, err := NewParser()
	require.NoError(t, err)

	program, err := parser.Parse("ddd.sruja", dsl)
	require.NoError(t, err)
	require.NotNil(t, program)

	arch := program.Architecture
	require.NotNil(t, arch)
	assert.Equal(t, "DDD Shop", arch.Name)

	// Check Contexts (collected from Domain)
	require.Len(t, arch.Contexts, 1)
	ctx := arch.Contexts[0]
	assert.Equal(t, "OrderManagement", ctx.ID)

	// Check Aggregates
	require.Len(t, ctx.Aggregates, 1)
	agg := ctx.Aggregates[0]
	assert.Equal(t, "Order", agg.ID)

	// Check Entities inside Aggregate
	require.Len(t, agg.Entities, 1)
	ent := agg.Entities[0]
	assert.Equal(t, "OrderLineItem", ent.ID)
	require.Len(t, ent.Fields, 2)
	assert.Equal(t, "name", ent.Fields[0].Name)
	assert.Equal(t, "string", ent.Fields[0].Type)

	// Check ValueObjects inside Aggregate
	require.Len(t, agg.ValueObjects, 1)
	vo := agg.ValueObjects[0]
	assert.Equal(t, "ShippingAddress", vo.ID)
	require.Len(t, vo.Fields, 2)

	// Check Events inside Context
	require.Len(t, ctx.Events, 1)
	evt := ctx.Events[0]
	assert.Equal(t, "OrderCreated", evt.ID)
	require.Len(t, evt.Fields, 1)
}
