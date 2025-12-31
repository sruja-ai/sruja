package main

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// comprehensiveDSL returns a comprehensive DSL for integration testing
func comprehensiveDSL() string {
	return `Person = kind "Person"
		System = kind "System"
		Container = kind "Container"
		Component = kind "Component"
		Datastore = kind "Datastore"
		Queue = kind "Queue"
		Adr = kind "ADR"
		Requirement = kind "Requirement"
		Scenario = kind "Scenario"
		overview {
			version "1.0.0"
			owner "Tech Team"
		}

		Customer = Person "Customer" {
			description "End user"
		}

		Backend = System "Backend System" {
			description "Core backend services"
			
			API = Container "REST API" {
				technology "Go"
				tags ["api", "rest"]
				version "2.0.0"
				
				Auth = Component "Authentication" {
					technology "JWT"
					description "Handles authentication"
				}
				
				Orders = Component "Order Management" {
					technology "Go"
				}
				
				Cache = Datastore "Redis Cache"
				Events = Queue "Event Queue"
				
				Auth -> Orders "forwards to"
				Auth -> Cache "caches tokens"
				Orders -> Events "publishes events"
			}
			
			Worker = Container "Background Worker" {
				technology "Python"
			}
			
			DB = Datastore "PostgreSQL"
			JobQueue = Queue "Job Queue"
			
			API -> DB "reads/writes"
			API -> JobQueue "publishes"
			Worker -> DB "reads/writes"
			Worker -> JobQueue "consumes"
		}

		Frontend = System "Frontend" {
			Web = Container "Web App" {
				technology "React"
			}
		}

		Customer -> Frontend "uses"
		Frontend -> Backend "calls" "HTTPS"
		Frontend.Web -> Backend.API "calls API"

		FR1 = Requirement "User authentication required" { tags ["functional"] }
		NFR1 = Requirement "99.9% uptime" { tags ["non_functional"] }

		ADR1 = Adr "Use microservices" {
			status "Accepted"
			context "Need to scale"
			decision "Microservices"
			consequences "Complexity"
		}

		Login = Scenario "User Login" {
			// description "User logs in"
			Customer -> Frontend "Opens app"
			Frontend -> Backend "Authenticates"
		}`
}

// TestCLI_ExportJSON tests the CLI export JSON command with comprehensive DSL
func TestCLI_ExportJSON(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "comprehensive.sruja")

	// Write comprehensive DSL
	if err := os.WriteFile(file, []byte(comprehensiveDSL()), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	// Run export json command
	exitCode := runExport([]string{"json", file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("Export failed with exit code %d. Stderr: %s", exitCode, stderr.String())
	}

	// Parse and validate JSON output
	var result map[string]interface{}
	if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
		t.Fatalf("Failed to parse JSON output: %v", err)
	}

	// Validate structure
	metadata := result["_metadata"].(map[string]interface{})
	if metadata["name"] != "Model" {
		t.Errorf("Expected name 'Model', got %v", metadata["name"])
	}

	// Validate persons (found in elements)
	elements := result["elements"].(map[string]interface{})
	personCount := 0
	for _, e := range elements {
		elem := e.(map[string]interface{})
		if elem["kind"] == "person" || elem["kind"] == "Person" {
			personCount++
		}
	}
	if personCount != 1 {
		t.Errorf("Expected 1 person, got %d", personCount)
	}

	// Validate systems
	systemCount := 0
	for _, e := range elements {
		elem := e.(map[string]interface{})
		if elem["kind"] == "system" || elem["kind"] == "System" {
			systemCount++
		}
	}
	if systemCount != 2 {
		t.Errorf("Expected 2 systems, got %d", systemCount)
	}

	// Validate specific elements exist with FQNs
	expectedFQNs := []string{
		"Backend",
		"Backend.API",
		"Backend.API.Auth",
		"Backend.API.Orders",
		"Backend.Worker",
		"Backend.DB",
		"Backend.JobQueue",
		"Frontend",
		"Frontend.Web",
		"Customer",
	}

	for _, fqn := range expectedFQNs {
		if _, ok := elements[fqn]; !ok {
			t.Errorf("Expected element with FQN '%s' not found", fqn)
		}
	}

	// Validate components are present
	auth, ok := elements["Backend.API.Auth"].(map[string]interface{})
	if !ok || (auth["kind"] != "component" && auth["kind"] != "Component") {
		t.Errorf("Backend.API.Auth is not a component or not found")
	}
	if auth["parent"] != "Backend.API" {
		t.Errorf("Expected parent 'Backend.API' for Auth, got '%v'", auth["parent"])
	}

	// Validate relations
	relations := result["relations"].([]interface{})
	if len(relations) != 14 {
		t.Errorf("Expected 14 relations, got %d", len(relations))
	}

	// Validate Sruja extensions
	srujaExt := result["sruja"].(map[string]interface{})

	// Validate requirements
	requirements := srujaExt["requirements"].([]interface{})
	if len(requirements) != 2 {
		t.Errorf("Expected 2 requirements, got %d", len(requirements))
	}

	// Validate ADRs
	adrs := srujaExt["adrs"].([]interface{})
	if len(adrs) != 1 {
		t.Errorf("Expected 1 ADR, got %d", len(adrs))
	}

	// Validate scenarios
	scenarios := srujaExt["scenarios"].([]interface{})
	if len(scenarios) != 1 {
		t.Errorf("Expected 1 scenario, got %d", len(scenarios))
	}
}

// TestCLI_ExportMarkdown tests that markdown export produces valid output
func TestCLI_ExportMarkdown(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "comprehensive.sruja")

	if err := os.WriteFile(file, []byte(comprehensiveDSL()), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	// Markdown export is now enabled - should return success
	exitCode := runExport([]string{"markdown", file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("Expected markdown export to succeed, but got exit code %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), "# Architecture") {
		t.Errorf("Expected markdown title, got: %s", stdout.String())
	}
	if !strings.Contains(stdout.String(), "```mermaid") {
		t.Errorf("Expected embedded mermaid diagram")
	}
}

// TestCLI_ExportHTML tests HTML export with comprehensive DSL
func TestCLI_ExportHTML(t *testing.T) {
	t.Skip("HTML export removed from CLI")
}

// TestCLI_CompileComprehensive tests compile with comprehensive DSL
func TestCLI_CompileComprehensive(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "comprehensive.sruja")

	if err := os.WriteFile(file, []byte(comprehensiveDSL()), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	exitCode := runCompile([]string{file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("Compile failed. Stderr: %s", stderr.String())
	}

	if !strings.Contains(stdout.String(), "Compilation successful") {
		t.Error("Expected successful compilation")
	}
}

// TestCLI_InvalidDSL tests that CLI returns non-zero for invalid syntax
func TestCLI_InvalidDSL(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "invalid.sruja")

	if err := os.WriteFile(file, []byte("invalid { !!! }"), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	exitCode := runCompile([]string{file}, &stdout, &stderr)

	if exitCode == 0 {
		t.Error("Expected non-zero exit code for invalid DSL, but got 0")
	}
	if stderr.Len() == 0 {
		t.Error("Expected error messages in stderr, but got none")
	}
}

// TestCLI_MissingFile tests that CLI returns non-zero for missing files
func TestCLI_MissingFile(t *testing.T) {
	var stdout, stderr bytes.Buffer
	exitCode := runCompile([]string{"non_existent_file.sruja"}, &stdout, &stderr)

	if exitCode == 0 {
		t.Error("Expected non-zero exit code for missing file, but got 0")
	}
	if !strings.Contains(stderr.String(), "no such file or directory") && !strings.Contains(stderr.String(), "failed to read file") {
		t.Errorf("Expected file error in stderr, got: %s", stderr.String())
	}
}
