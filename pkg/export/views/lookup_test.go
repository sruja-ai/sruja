package views

import (
	"testing"
)

func TestElementLookup_ResolveFQN(t *testing.T) {
	dsl := `
		System = system "My System" {
			API = container "API"
		}
	`
	prog := parseDSL(t, dsl)
	lookup := BuildElementLookup(prog)

	tests := []struct {
		shortID   string
		contextID string
		expected  string
	}{
		{"System", "", "System"},
		{"API", "System", "System.API"},
		{"API", "", "System.API"}, // Auto-resolve if unique
		{"Unknown", "", "Unknown"},
	}

	for _, tt := range tests {
		got := lookup.ResolveFQN(tt.shortID, tt.contextID)
		if got != tt.expected {
			t.Errorf("ResolveFQN(%s, %s) = %s, want %s", tt.shortID, tt.contextID, got, tt.expected)
		}
	}
}

func TestElementLookup_GetRoot(t *testing.T) {
	dsl := `
		System = system "My System" {
			API = container "API" {
				Auth = component "Auth"
			}
		}
	`
	prog := parseDSL(t, dsl)
	lookup := BuildElementLookup(prog)

	root, ok := lookup.GetRoot("System.API.Auth")
	if !ok || root != "System" {
		t.Errorf("GetRoot(System.API.Auth) = %s, %v; want System, true", root, ok)
	}

	root, ok = lookup.GetRoot("System")
	if !ok || root != "System" {
		t.Errorf("GetRoot(System) = %s, %v; want System, true", root, ok)
	}
}

func TestElementLookup_GetContainer(t *testing.T) {
	dsl := `
		System = system "My System" {
			API = container "API" {
				Auth = component "Auth"
			}
			DB = database "DB"
		}
	`
	prog := parseDSL(t, dsl)
	lookup := BuildElementLookup(prog)

	cont := lookup.GetContainer("System.API.Auth")
	if cont != "System.API" {
		t.Errorf("GetContainer(System.API.Auth) = %s; want System.API", cont)
	}

	cont = lookup.GetContainer("System.API")
	if cont != "System.API" {
		t.Errorf("GetContainer(System.API) = %s; want System.API", cont)
	}

	// Verify Datastore returns itself
	cont = lookup.GetContainer("System.DB")
	if cont != "System.DB" {
		t.Errorf("GetContainer(System.DB) = %s; want System.DB", cont)
	}

	cont = lookup.GetContainer("System")
	if cont != "" {
		t.Errorf("GetContainer(System) = %s; want empty", cont)
	}
}
