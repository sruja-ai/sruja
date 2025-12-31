package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParser_SpecificationItems(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		wantErr  bool
		validate func(t *testing.T, prog *language.Program)
	}{
		{
			name: "Simple specification with elements",
			input: `
				person = kind "Person"
				system = kind "System"
				container = kind "Container"
				component = kind "Component"
			`,
			wantErr: false,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Specification)
				assert.Len(t, prog.Specification.Items, 4)
			},
		},
		{
			name: "Specification with element body",
			input: `
				microservice = kind "Microservice" {
					technology "Go"
				}
			`,
			wantErr: false,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Specification)
				assert.Len(t, prog.Specification.Items, 1)
				elem := prog.Specification.Items[0].Element
				require.NotNil(t, elem)
				assert.Equal(t, "microservice", elem.Name)
				require.NotNil(t, elem.Title)
				assert.Equal(t, "Microservice", *elem.Title)
			},
		},
		{
			name: "Specification with tag",
			input: `
				deprecated = tag "Deprecated"
				critical = tag "Critical"
			`,
			wantErr: false,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Specification)
				assert.Len(t, prog.Specification.Items, 2)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p, err := language.NewParser()
			require.NoError(t, err)

			prog, diags, err := p.Parse("test.sruja", tt.input)
			if tt.wantErr {
				assert.Error(t, err)
				return
			}
			require.NoError(t, err)
			assert.Empty(t, diags)
			if tt.validate != nil {
				tt.validate(t, prog)
			}
		})
	}
}

func TestParser_ModelItems(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		wantErr  bool
		validate func(t *testing.T, prog *language.Program)
	}{
		{
			name: "Model with person",
			input: `
				customer = person "Customer"
			`,
			wantErr: false,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Model)
				assert.Len(t, prog.Model.Items, 1)
				elem := prog.Model.Items[0].ElementDef
				require.NotNil(t, elem)
				assert.Equal(t, "customer", elem.GetID())
				assert.Equal(t, "person", elem.GetKind())
			},
		},
		{
			name: "Model with system and nested containers",
			input: `
				backend = system "Backend" {
					api = container "REST API"
					db = database "PostgreSQL"
				}
			`,
			wantErr: false,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Model)
				assert.Len(t, prog.Model.Items, 1)
				elem := prog.Model.Items[0].ElementDef
				require.NotNil(t, elem)
				assert.Equal(t, "backend", elem.GetID())
				body := elem.GetBody()
				require.NotNil(t, body)
				assert.Len(t, body.Items, 2)
			},
		},
		{
			name: "Model with relation",
			input: `
				user = person "User"
				api = system "API"
				user -> api "uses"
			`,
			wantErr: false,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Model)
				// 2 elements + 1 relation
				elemCount := 0
				relCount := 0
				for _, item := range prog.Model.Items {
					if item.ElementDef != nil {
						elemCount++
					}
					if item.Relation != nil {
						relCount++
					}
				}
				assert.Equal(t, 2, elemCount)
				assert.Equal(t, 1, relCount)
			},
		},
		{
			name: "Model with description in body",
			input: `
				api = system "API" {
					description "Main API service"
				}
			`,
			wantErr: false,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Model)
				elem := prog.Model.Items[0].ElementDef
				require.NotNil(t, elem)
				body := elem.GetBody()
				require.NotNil(t, body)
				found := false
				for _, item := range body.Items {
					if item.Description != nil {
						assert.Equal(t, "Main API service", *item.Description)
						found = true
						break
					}
				}
				assert.True(t, found, "Description not found in body items")
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p, err := language.NewParser()
			require.NoError(t, err)

			prog, diags, err := p.Parse("test.sruja", tt.input)
			if tt.wantErr {
				assert.Error(t, err)
				return
			}
			require.NoError(t, err)
			assert.Empty(t, diags)
			if tt.validate != nil {
				tt.validate(t, prog)
			}
		})
	}
}

func TestParser_ViewItems(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		wantErr  bool
		validate func(t *testing.T, prog *language.Program)
	}{
		{
			name: "Simple view",
			input: `
				view index {
					title "Overview"
					include *
				}
			`,
			wantErr: false,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Views)
				assert.Len(t, prog.Views.Items, 1)
				view := prog.Views.Items[0].View
				require.NotNil(t, view)
				if view.Name != nil {
					assert.Equal(t, "index", *view.Name)
				}
				// Title can be in ViewDef or ViewBody
				title := ""
				if view.Title != nil {
					title = *view.Title
				} else if view.Body != nil {
					for _, item := range view.Body.Items {
						if item.Title != nil {
							title = *item.Title
							break
						}
					}
				}
				assert.Equal(t, "Overview", title)
			},
		},
		{
			name: "View with of clause",
			input: `
				view backend of Backend {
					include *
				}
			`,
			wantErr: false,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Views)
				assert.Len(t, prog.Views.Items, 1)
				view := prog.Views.Items[0].View
				require.NotNil(t, view)
				require.NotNil(t, view.Of)
			},
		},
		{
			name: "Multiple views",
			input: `
				view landscape {
					include *
				}
				view containers {
					include *
				}
			`,
			wantErr: false,
			validate: func(t *testing.T, prog *language.Program) {
				require.NotNil(t, prog.Views)
				assert.Len(t, prog.Views.Items, 2)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p, err := language.NewParser()
			require.NoError(t, err)

			prog, diags, err := p.Parse("test.sruja", tt.input)
			if tt.wantErr {
				assert.Error(t, err)
				return
			}
			require.NoError(t, err)
			assert.Empty(t, diags)
			if tt.validate != nil {
				tt.validate(t, prog)
			}
		})
	}
}

func TestParser_FullFile(t *testing.T) {
	input := `
	person = kind "Person"
	system = kind "System"
	container = kind "Container"
	database = kind "Database"

	customer = person "Customer" {
		description "End user of the platform"
	}

	ecommerce = system "E-Commerce Platform" {
		webApp = container "Web Application"
		api = container "REST API"
		db = database "PostgreSQL"
	}

	customer -> ecommerce.webApp "browses products"

	view landscape {
		title "System Landscape"
		include *
	}
`

	p, err := language.NewParser()
	require.NoError(t, err)

	prog, diags, err := p.Parse("test.sruja", input)
	require.NoError(t, err)
	assert.Empty(t, diags)

	// Verify all blocks were parsed
	assert.NotNil(t, prog.Specification)
	assert.NotNil(t, prog.Model)
	assert.NotNil(t, prog.Views)

	// Verify specification
	assert.Len(t, prog.Specification.Items, 4)

	// Verify model has elements
	assert.GreaterOrEqual(t, len(prog.Model.Items), 2) // At least customer and ecommerce

	// Verify views
	assert.Len(t, prog.Views.Items, 1)
}
