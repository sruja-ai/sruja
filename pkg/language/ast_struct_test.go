//go:build legacy

// pkg/language/ast_struct_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestAST_StructureParsing(t *testing.T) {
	dsl := `
architecture "Test" {
  system App "Application" {
    container API "HTTP API" {
      tags ["http", "public"]
      component Checkout "Checkout"
    }
    datastore DB "PostgreSQL"
    queue MQ "Events"
  }
  App -> DB reads "Reads data"
  API -> MQ publishes "Publishes events"
  requirement R1 functional "Persist data"
  adr ADR001 "Use PostgreSQL"
}`

	p, err := language.NewParser()
	if err != nil {
		t.Fatalf("parser: %v", err)
	}
	prog, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}

	if prog.Architecture == nil {
		t.Fatalf("architecture nil")
	}

	// Import feature removed - no longer checking imports

	// Systems
	if len(prog.Architecture.Systems) != 1 {
		t.Fatalf("expected 1 system, got %d", len(prog.Architecture.Systems))
	}

	sys := prog.Architecture.Systems[0]
	if sys.ID != "App" {
		t.Fatalf("expected system ID 'App', got '%s'", sys.ID)
	}

	// System items: container, datastore, queue
	hasContainer := false
	hasDataStore := false
	hasQueue := false
	for _, item := range sys.Items {
		if item.Container != nil {
			hasContainer = true
		}
		if item.DataStore != nil {
			hasDataStore = true
		}
		if item.Queue != nil {
			hasQueue = true
		}
	}
	if !hasContainer {
		t.Fatalf("expected container in system items")
	}
	if !hasDataStore {
		t.Fatalf("expected datastore in system items")
	}
	if !hasQueue {
		t.Fatalf("expected queue in system items")
	}

	// Container items: tags and component
	cont := sys.Containers[0]
	foundContTags := false
	foundComponent := false
	for _, item := range cont.Items {
		if item.Tags != nil && len(item.Tags) == 2 {
			foundContTags = true
		}
		if item.Component != nil && item.Component.ID == "Checkout" {
			foundComponent = true
		}
	}
	if !foundContTags {
		t.Fatalf("expected container tags")
	}
	if !foundComponent {
		t.Fatalf("expected component in container")
	}

	// Relation verbs
	if len(prog.Architecture.Relations) != 2 {
		t.Fatalf("expected 2 relations, got %d", len(prog.Architecture.Relations))
	}
	rel1 := prog.Architecture.Relations[0]
	rel2 := prog.Architecture.Relations[1]
	if rel1.Verb == nil || *rel1.Verb != "reads" {
		t.Fatalf("expected first relation verb 'reads', got '%v'", rel1.Verb)
	}
	if rel2.Verb == nil || *rel2.Verb != "publishes" {
		t.Fatalf("expected second relation verb 'publishes', got '%v'", rel2.Verb)
	}

	// Requirements
	if len(prog.Architecture.Requirements) != 1 {
		t.Fatalf("expected 1 requirement, got %d", len(prog.Architecture.Requirements))
	}

	// ADRs
	if len(prog.Architecture.ADRs) != 1 {
		t.Fatalf("expected 1 adr, got %d", len(prog.Architecture.ADRs))
	}
}

func TestAST_Getters(t *testing.T) {
	dsl := `
architecture "Test" {
  system API "API" {
    container Web "Web" {
      component Cart "Cart"
    }
  }
}`
	p, _ := language.NewParser()
	prog, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	sys := prog.Architecture.Systems[0]
	cont := sys.Containers[0]
	comp := cont.Components[0]
	if sys.GetID() != "API" || sys.GetLabel() != "API" {
		t.Fatalf("system getters failed")
	}
	if cont.GetID() != "Web" || cont.GetLabel() != "Web" {
		t.Fatalf("container getters failed")
	}
	if comp.GetID() != "Cart" || comp.GetLabel() != "Cart" {
		t.Fatalf("component getters failed")
	}
}
