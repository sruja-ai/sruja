package language

import (
	"testing"
)

func TestParser_FlexibleRoot(t *testing.T) {
	tests := []struct {
		name  string
		input string
		check func(*testing.T, *Program)
	}{
		{
			name:  "Bare System",
			input: `system API "API Service"`,
			check: func(t *testing.T, p *Program) {
				if len(p.Architecture.Systems) != 1 {
					t.Errorf("Expected 1 system, got %d", len(p.Architecture.Systems))
				}
				if p.Architecture.Systems[0].ID != "API" {
					t.Errorf("Expected system ID 'API', got %q", p.Architecture.Systems[0].ID)
				}
			},
		},
		{
			name:  "Bare Component",
			input: `component Comp "My Component"`,
			check: func(t *testing.T, p *Program) {
				if len(p.Architecture.Components) != 1 {
					t.Errorf("Expected 1 component, got %d", len(p.Architecture.Components))
				}
				if p.Architecture.Components[0].ID != "Comp" {
					t.Errorf("Expected component ID 'Comp', got %q", p.Architecture.Components[0].ID)
				}
			},
		},
		{
			name: "Mixed Bare Items",
			input: `
				system API "API Service"
				component Comp "My Component"
				adr ADR001 "Decision"
			`,
			check: func(t *testing.T, p *Program) {
				if len(p.Architecture.Systems) != 1 {
					t.Errorf("Expected 1 system")
				}
				if len(p.Architecture.Components) != 1 {
					t.Errorf("Expected 1 component")
				}
				if len(p.Architecture.ADRs) != 1 {
					t.Errorf("Expected 1 ADR")
				}
			},
		},
		{
			name: "Architecture Block + Bare Items",
			input: `
				architecture "Main" {
					system API "API Service"
				}
				component Comp "My Component"
			`,
			check: func(t *testing.T, p *Program) {
				if p.Architecture.Name != "Main" {
					t.Errorf("Expected architecture name 'Main', got %q", p.Architecture.Name)
				}
				if len(p.Architecture.Systems) != 1 {
					t.Errorf("Expected 1 system")
				}
				if len(p.Architecture.Components) != 1 {
					t.Errorf("Expected 1 component")
				}
			},
		},
		{
			name:  "Bare Container",
			input: `container Web "Web App"`,
			check: func(t *testing.T, p *Program) {
				if len(p.Architecture.Containers) != 1 {
					t.Errorf("Expected 1 container, got %d", len(p.Architecture.Containers))
				}
				if p.Architecture.Containers[0].ID != "Web" {
					t.Errorf("Expected container ID 'Web', got %q", p.Architecture.Containers[0].ID)
				}
			},
		},
		{
			name:  "Bare DataStore",
			input: `datastore DB "Database"`,
			check: func(t *testing.T, p *Program) {
				if len(p.Architecture.DataStores) != 1 {
					t.Errorf("Expected 1 datastore, got %d", len(p.Architecture.DataStores))
				}
				if p.Architecture.DataStores[0].ID != "DB" {
					t.Errorf("Expected datastore ID 'DB', got %q", p.Architecture.DataStores[0].ID)
				}
			},
		},
		{
			name:  "Bare Queue",
			input: `queue Q "Message Queue"`,
			check: func(t *testing.T, p *Program) {
				if len(p.Architecture.Queues) != 1 {
					t.Errorf("Expected 1 queue, got %d", len(p.Architecture.Queues))
				}
				if p.Architecture.Queues[0].ID != "Q" {
					t.Errorf("Expected queue ID 'Q', got %q", p.Architecture.Queues[0].ID)
				}
			},
		},
		{
			name:  "System Missing Label",
			input: `system MySystem`,
			check: func(t *testing.T, p *Program) {
				if len(p.Architecture.Systems) != 1 {
					t.Errorf("Expected 1 system, got %d", len(p.Architecture.Systems))
				}
				if p.Architecture.Systems[0].ID != "MySystem" {
					t.Errorf("Expected system ID 'MySystem', got %q", p.Architecture.Systems[0].ID)
				}
				if p.Architecture.Systems[0].Label != "" {
					t.Errorf("Expected empty label, got %q", p.Architecture.Systems[0].Label)
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			parser, err := NewParser()
			if err != nil {
				t.Fatalf("Failed to create parser: %v", err)
			}
			program, err := parser.Parse("test.sruja", tt.input)
			if err != nil {
				t.Fatalf("Failed to parse: %v", err)
			}
			tt.check(t, program)
		})
	}
}
