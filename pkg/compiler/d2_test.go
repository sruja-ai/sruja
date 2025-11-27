//go:build legacy

// Package compiler_test provides tests for the D2 compiler.
package compiler_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestD2Compiler_Name(t *testing.T) {
	c := compiler.NewD2Compiler()
	if c.Name() != "d2" {
		t.Errorf("Expected compiler name 'd2', got '%s'", c.Name())
	}
}

func TestD2Compiler_SimpleSystem(t *testing.T) {
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

	c := compiler.NewD2Compiler()
	output, err := c.Compile(program)
	if err != nil {
		t.Fatalf("Failed to compile: %v", err)
	}

	// Check that output contains the system
	if !strings.Contains(output, "API") {
		t.Errorf("Expected output to contain 'API', got: %s", output)
	}
	if !strings.Contains(output, "API Service") {
		t.Errorf("Expected output to contain 'API Service', got: %s", output)
	}
}

func TestD2Compiler_SystemWithContainer(t *testing.T) {
	dsl := `
architecture "Test" {
	model {
		system API "API Service" {
			container WebApp "Web Application" {
				technology "React"
			}
		}
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

	c := compiler.NewD2Compiler()
	output, err := c.Compile(program)
	if err != nil {
		t.Fatalf("Failed to compile: %v", err)
	}

	// Check that output contains both system and container
	if !strings.Contains(output, "API") {
		t.Errorf("Expected output to contain 'API', got: %s", output)
	}
	if !strings.Contains(output, "WebApp") {
		t.Errorf("Expected output to contain 'WebApp', got: %s", output)
	}
}

func TestD2Compiler_Relations(t *testing.T) {
	dsl := `
architecture "Test" {
	model {
		system API "API Service"
		system DB "Database"
		API -> DB "Reads/Writes"
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

	c := compiler.NewD2Compiler()
	output, err := c.Compile(program)
	if err != nil {
		t.Fatalf("Failed to compile: %v", err)
	}

	// Check that output contains the relation
	if !strings.Contains(output, "API -> DB") {
		t.Errorf("Expected output to contain 'API -> DB', got: %s", output)
	}
	if !strings.Contains(output, "Reads/Writes") {
		t.Errorf("Expected output to contain 'Reads/Writes', got: %s", output)
	}
}

func TestD2Compiler_WithTheme(t *testing.T) {
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

	c := compiler.NewD2CompilerWithOptions(compiler.D2Options{
		Theme: "gruvbox-dark",
	})
	output, err := c.Compile(program)
	if err != nil {
		t.Fatalf("Failed to compile: %v", err)
	}

	// Check that theme is set (if D2 compiler supports it in output)
	if !strings.Contains(output, "API") {
		t.Errorf("Expected output to contain 'API', got: %s", output)
	}
}

func TestD2Compiler_ComplexArchitecture(t *testing.T) {
	dsl := `
architecture "Test" {
	model {
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

	c := compiler.NewD2Compiler()
	output, err := c.Compile(program)
	if err != nil {
		t.Fatalf("Failed to compile: %v", err)
	}

	// Check that all elements are present
	expectedElements := []string{"User", "API", "WebApp", "Database"}
	for _, elem := range expectedElements {
		if !strings.Contains(output, elem) {
			t.Errorf("Expected output to contain '%s', got: %s", elem, output)
		}
	}

	// Check that relations are present
	if !strings.Contains(output, "User -> API") {
		t.Errorf("Expected output to contain 'User -> API', got: %s", output)
	}
}

func TestD2Compiler_DataStoreAndQueue(t *testing.T) {
	dsl := `
architecture "Test" {
	model {
		system API "API Service" {
			datastore DB "PostgreSQL"
			queue Queue "Message Queue"
		}
		API -> DB "Reads/Writes"
		API -> Queue "Publishes"
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

	c := compiler.NewD2Compiler()
	output, err := c.Compile(program)
	if err != nil {
		t.Fatalf("Failed to compile: %v", err)
	}

	// Check that all element types are present
	if !strings.Contains(output, "API") {
		t.Errorf("Expected output to contain 'API'")
	}
	if !strings.Contains(output, "DB") {
		t.Errorf("Expected output to contain 'DB'")
	}
	if !strings.Contains(output, "Queue") {
		t.Errorf("Expected output to contain 'Queue'")
	}
}
