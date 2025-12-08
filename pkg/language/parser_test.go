//go:build legacy

// pkg/language/parser_test.go
// Package language_test provides tests for the language parser.
// This file contains core parser tests for basic architecture and element parsing.
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_EmptyArchitecture(t *testing.T) {
	dsl := `architecture "Test" { }`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	_, _, err = parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse empty architecture: %v", err)
	}
}

func TestParser_BasicSystem(t *testing.T) {
	dsl := `
architecture "Test" {
	system API "API Service" {}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse valid DSL: %v", err)
	}

	if program.Architecture == nil {
		t.Fatal("Expected architecture to be parsed, got nil")
	}

	if len(program.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(program.Architecture.Systems))
	}

	sys := program.Architecture.Systems[0]
	if sys.ID != "API" {
		t.Errorf("Expected system ID 'API', got '%s'", sys.ID)
	}
	if sys.Label != "API Service" {
		t.Errorf("Expected system label 'API Service', got '%s'", sys.Label)
	}
}

func TestParser_SystemWithDescription(t *testing.T) {
	dsl := `
architecture "Test" {
	system API "API Service" "Handles all API requests" {}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL with description: %v", err)
	}

	if len(program.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(program.Architecture.Systems))
	}

	sys := program.Architecture.Systems[0]
	if sys.Description == nil {
		t.Fatal("Expected description to be present")
	}
	if *sys.Description != "Handles all API requests" {
		t.Errorf("Expected description 'Handles all API requests', got '%s'", *sys.Description)
	}
}

func TestParser_SystemWithContainer(t *testing.T) {
	dsl := `
architecture "Test" {
	system API "API Service" {
		container WebApp "Web Application"
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse system with container: %v", err)
	}

	if len(program.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(program.Architecture.Systems))
	}

	sys := program.Architecture.Systems[0]
	if len(sys.Items) != 1 {
		t.Fatalf("Expected 1 item, got %d", len(sys.Items))
	}

	if sys.Items[0].Container == nil {
		t.Fatal("Expected container in system items")
	}

	container := sys.Items[0].Container
	if container.ID != "WebApp" {
		t.Errorf("Expected container ID 'WebApp', got '%s'", container.ID)
	}
	if container.Label != "Web Application" {
		t.Errorf("Expected container label 'Web Application', got '%s'", container.Label)
	}
}

func TestParser_ContainerWithTechnology(t *testing.T) {
	dsl := `
architecture "Test" {
	system API "API Service" {
		container WebApp "Web Application" {
			technology "React"
		}
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse container with technology: %v", err)
	}

	sys := program.Architecture.Systems[0]
	container := sys.Items[0].Container

	// Technology should be in the container items
	if len(container.Items) != 1 {
		t.Fatalf("Expected 1 item in container, got %d", len(container.Items))
	}

	if container.Items[0].Technology == nil {
		t.Fatal("Expected technology in container items")
	}

	if *container.Items[0].Technology != "React" {
		t.Errorf("Expected technology 'React', got '%s'", *container.Items[0].Technology)
	}
}

func TestParser_Relation(t *testing.T) {
	dsl := `
architecture "Test" {
	system User "End User"
	system API "API Service"
	User -> API "Uses"
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse relation: %v", err)
	}

	if len(program.Architecture.Systems) != 2 {
		t.Fatalf("Expected 2 systems, got %d", len(program.Architecture.Systems))
	}

	if len(program.Architecture.Relations) != 1 {
		t.Fatalf("Expected 1 relation, got %d", len(program.Architecture.Relations))
	}

	rel := program.Architecture.Relations[0]
	if rel.From != "User" {
		t.Errorf("Expected relation from 'User', got '%s'", rel.From)
	}
	if rel.To != "API" {
		t.Errorf("Expected relation to 'API', got '%s'", rel.To)
	}
	if rel.Label == nil {
		t.Fatal("Expected relation label to be present")
	}
	if *rel.Label != "Uses" {
		t.Errorf("Expected relation label 'Uses', got '%s'", *rel.Label)
	}
}

func TestParser_ComplexArchitecture(t *testing.T) {
	dsl := `
architecture "Test" {
	system User "End User"
	system API "API Service" {
		container WebApp "Web Application" {
			technology "React"
		}
		container Database "PostgreSQL Database" {
			technology "PostgreSQL 14"
		}
	}
	User -> API "Uses"
	API -> Database "Reads/Writes"
	requirement R1 functional "Must handle 10k concurrent users"
	requirement R2 performance "Response time < 200ms"
	adr ADR001 "Use microservices architecture"
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse complex architecture: %v", err)
	}

	// Verify systems
	if len(program.Architecture.Systems) != 2 {
		t.Errorf("Expected 2 systems, got %d", len(program.Architecture.Systems))
	}

	// Verify relations
	if len(program.Architecture.Relations) != 2 {
		t.Errorf("Expected 2 relations, got %d", len(program.Architecture.Relations))
	}

	// Verify requirements
	if len(program.Architecture.Requirements) != 2 {
		t.Errorf("Expected 2 requirements, got %d", len(program.Architecture.Requirements))
	}

	// Verify ADRs
	if len(program.Architecture.ADRs) != 1 {
		t.Errorf("Expected 1 ADR, got %d", len(program.Architecture.ADRs))
	}
}
