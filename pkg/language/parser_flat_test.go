package language_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_FlatSyntax(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		validate func(t *testing.T, prog *language.Program)
	}{
		{
			name: "Top-level specification items",
			input: `
				person = kind "Person"
				system = kind "System"
				critical = tag "Critical"
			`,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Specification)
				assert.Len(t, prog.Specification.Items, 3)

				// Check items
				assert.Equal(t, "person", prog.Specification.Items[0].Element.Name)
				assert.Equal(t, "system", prog.Specification.Items[1].Element.Name)
				assert.Equal(t, "critical", prog.Specification.Items[2].Tag.Name)
			},
		},

		{
			name: "Top-level views",
			input: `
				view index {
					title "Index"
					include *
				}
			`,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Views)
				assert.Len(t, prog.Views.Items, 1)
				assert.NotNil(t, prog.Views.Items[0].View)
				assert.Equal(t, "index", *prog.Views.Items[0].View.Name)
			},
		},

		{
			name: "Top-level governance elements",
			input: `
				REQ001 = requirement functional "Must be fast"
				ADR001 = adr "Choice"
			`,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Model)
				assert.Len(t, prog.Model.Items, 2)
				assert.Equal(t, "REQ001", prog.Model.Items[0].ElementDef.GetID())
				assert.Equal(t, "ADR001", prog.Model.Items[1].ElementDef.GetID())
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p, err := language.NewParser()
			require.NoError(t, err)

			prog, diags, err := p.Parse("test.sruja", tt.input)
			require.NoError(t, err)
			assert.Empty(t, diags)

			tt.validate(t, prog)
		})
	}
}
