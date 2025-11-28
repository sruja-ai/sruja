// pkg/language/parser_top_level_test.go
package language

import (
	"testing"
)

func TestParser_TopLevelSystem(t *testing.T) {
	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	dsl := `system API "API Service" {
  container WebApp "Web Application"
}`

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse: %v", err)
	}

	if program.Architecture == nil {
		t.Fatal("Expected Architecture to be created")
	}

	if len(program.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(program.Architecture.Systems))
	}

	if program.Architecture.Systems[0].ID != "API" {
		t.Errorf("Expected system ID 'API', got '%s'", program.Architecture.Systems[0].ID)
	}

	if len(program.Architecture.Systems[0].Containers) != 1 {
		t.Fatalf("Expected 1 container, got %d", len(program.Architecture.Systems[0].Containers))
	}
}

func TestParser_TopLevelContainer(t *testing.T) {
	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	dsl := `container WebApp "Web Application" {
  component Frontend "Frontend"
}`

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse: %v", err)
	}

	if len(program.Architecture.Containers) != 1 {
		t.Fatalf("Expected 1 container, got %d", len(program.Architecture.Containers))
	}

	if program.Architecture.Containers[0].ID != "WebApp" {
		t.Errorf("Expected container ID 'WebApp', got '%s'", program.Architecture.Containers[0].ID)
	}
}

func TestParser_TopLevelComponent(t *testing.T) {
	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	dsl := `component AuthService "Authentication Service" {}`

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse: %v", err)
	}

	if len(program.Architecture.Components) != 1 {
		t.Fatalf("Expected 1 component, got %d", len(program.Architecture.Components))
	}

	if program.Architecture.Components[0].ID != "AuthService" {
		t.Errorf("Expected component ID 'AuthService', got '%s'", program.Architecture.Components[0].ID)
	}
}

func TestParser_ArchitectureBlockStillWorks(t *testing.T) {
	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	dsl := `architecture "My System" {
  system API "API Service"
}`

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse: %v", err)
	}

	if program.Architecture.Name != "My System" {
		t.Errorf("Expected architecture name 'My System', got '%s'", program.Architecture.Name)
	}

	if len(program.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(program.Architecture.Systems))
	}
}
