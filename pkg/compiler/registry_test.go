//go:build legacy

// Package compiler_test provides tests for the compiler registry.
package compiler_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestRegistry_NewRegistry(t *testing.T) {
	registry := compiler.NewRegistry()
	if registry == nil {
		t.Fatal("Expected registry to be created, got nil")
	}

	// Check that default compilers are registered
	formats := registry.List()
	if len(formats) < 2 {
		t.Errorf("Expected at least 2 formats (d2, mermaid), got %d", len(formats))
	}

	// Check that d2 and mermaid are registered
	hasD2 := false
	hasMermaid := false
	for _, format := range formats {
		if format == "d2" {
			hasD2 = true
		}
		if format == "mermaid" {
			hasMermaid = true
		}
	}

	if !hasD2 {
		t.Error("Expected 'd2' to be registered")
	}
	if !hasMermaid {
		t.Error("Expected 'mermaid' to be registered")
	}
}

func TestRegistry_Get(t *testing.T) {
	registry := compiler.NewRegistry()

	// Test getting existing compiler
	d2Compiler, err := registry.Get("d2")
	if err != nil {
		t.Fatalf("Failed to get d2 compiler: %v", err)
	}
	if d2Compiler == nil {
		t.Fatal("Expected d2 compiler, got nil")
	}
	if d2Compiler.Name() != "d2" {
		t.Errorf("Expected compiler name 'd2', got '%s'", d2Compiler.Name())
	}

	// Test getting non-existent compiler
	_, err = registry.Get("nonexistent")
	if err == nil {
		t.Error("Expected error when getting non-existent compiler")
	}
}

func TestRegistry_List(t *testing.T) {
	registry := compiler.NewRegistry()
	formats := registry.List()

	if len(formats) == 0 {
		t.Error("Expected at least one format, got none")
	}

	// Check that formats are unique
	seen := make(map[string]bool)
	for _, format := range formats {
		if seen[format] {
			t.Errorf("Duplicate format found: %s", format)
		}
		seen[format] = true
	}
}

func TestRegistry_Register(t *testing.T) {
	registry := compiler.NewRegistry()

	// Create a custom compiler
	customCompiler := &testCompiler{name: "test"}

	// Register it
	registry.Register(customCompiler)

	// Verify it's registered
	compiler, err := registry.Get("test")
	if err != nil {
		t.Fatalf("Failed to get registered compiler: %v", err)
	}
	if compiler.Name() != "test" {
		t.Errorf("Expected compiler name 'test', got '%s'", compiler.Name())
	}

	// Verify it's in the list
	formats := registry.List()
	found := false
	for _, format := range formats {
		if format == "test" {
			found = true
			break
		}
	}
	if !found {
		t.Error("Expected 'test' format to be in list")
	}
}

func TestRegistry_Compile(t *testing.T) {
	dsl := `
architecture "Test" {
    model {
        system API "API Service"
    }
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	registry := compiler.NewRegistry()

	// Test compiling with d2
	output, err := registry.Compile("d2", program)
	if err != nil {
		t.Fatalf("Failed to compile with d2: %v", err)
	}
	if output == "" {
		t.Error("Expected non-empty output from d2 compiler")
	}

	// Test compiling with mermaid
	output, err = registry.Compile("mermaid", program)
	if err != nil {
		t.Fatalf("Failed to compile with mermaid: %v", err)
	}
	if output == "" {
		t.Error("Expected non-empty output from mermaid compiler")
	}

	// Test compiling with non-existent format
	_, err = registry.Compile("nonexistent", program)
	if err == nil {
		t.Error("Expected error when compiling with non-existent format")
	}
}

// testCompiler is a simple test implementation of the Compiler interface
type testCompiler struct {
	name string
}

func (c *testCompiler) Name() string {
	return c.name
}

func (c *testCompiler) Compile(program *language.Program) (string, error) {
	return "test output", nil
}
