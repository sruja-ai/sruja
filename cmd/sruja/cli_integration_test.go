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
	return `architecture "IntegrationTest" {
		metadata {
			version "1.0.0"
			owner "Tech Team"
		}



		person Customer "Customer" {
			description "End user"
		}

		system Backend "Backend System" {
			description "Core backend services"
			
			container API "REST API" {
				technology "Go"
				tags ["api", "rest"]
				version "2.0.0"
				
				component Auth "Authentication" {
					technology "JWT"
					description "Handles authentication"
				}
				
				component Router "Request Router" {
					technology "Chi"
				}
				
				datastore Cache "Redis Cache"
				queue Events "Event Queue"
				
				Auth -> Router "forwards to"
				Auth -> Cache "caches tokens"
				Router -> Events "publishes events"
			}
			
			container Worker "Background Worker" {
				technology "Python"
			}
			
			datastore DB "PostgreSQL"
			queue JobQueue "Job Queue"
			
			API -> DB "reads/writes"
			API -> JobQueue "publishes"
			Worker -> DB "reads/writes"
			Worker -> JobQueue "consumes"
		}

		system Frontend "Frontend" {
			container Web "Web App" {
				technology "React"
			}
		}

		Customer -> Frontend "uses"
		Frontend -> Backend "calls" "HTTPS"
		Frontend.Web -> Backend.API "calls API"

		requirement FR1 functional "User authentication required"
		requirement NFR1 non_functional "99.9% uptime"

		adr ADR1 "Use microservices" {
			status "Accepted"
			context "Need to scale"
			decision "Microservices"
			consequences "Complexity"
		}

		scenario Login "User Login" "User logs in" {
			Customer -> Frontend "Opens app"
			Frontend -> Backend "Authenticates"
		}
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
	metadata := result["metadata"].(map[string]interface{})
	if metadata["name"] != "IntegrationTest" {
		t.Errorf("Expected name 'IntegrationTest', got %v", metadata["name"])
	}

	arch := result["architecture"].(map[string]interface{})

	// Validate persons
	persons := arch["persons"].([]interface{})
	if len(persons) != 1 {
		t.Errorf("Expected 1 person, got %d", len(persons))
	}

	// Validate systems
	systems := arch["systems"].([]interface{})
	if len(systems) != 2 {
		t.Errorf("Expected 2 systems, got %d", len(systems))
	}

	// Validate Backend system has containers
	backend := systems[0].(map[string]interface{})
	containers := backend["containers"].([]interface{})
	if len(containers) != 2 {
		t.Errorf("Expected 2 containers, got %d", len(containers))
	}

	// Validate API container has components
	api := containers[0].(map[string]interface{})
	components := api["components"].([]interface{})
	if len(components) != 2 {
		t.Errorf("Expected 2 components, got %d", len(components))
	}

	// Validate relations
	relations := arch["relations"].([]interface{})
	if len(relations) != 4 {
		t.Errorf("Expected 4 relations, got %d", len(relations))
	}

	// Validate requirements
	requirements := arch["requirements"].([]interface{})
	if len(requirements) != 2 {
		t.Errorf("Expected 2 requirements, got %d", len(requirements))
	}

	// Validate ADRs
	adrs := arch["adrs"].([]interface{})
	if len(adrs) != 1 {
		t.Errorf("Expected 1 ADR, got %d", len(adrs))
	}

	// Validate scenarios
	scenarios := arch["scenarios"].([]interface{})
	if len(scenarios) != 1 {
		t.Errorf("Expected 1 scenario, got %d", len(scenarios))
	}
}

// TestCLI_ExportMarkdown tests that markdown export is disabled
func TestCLI_ExportMarkdown(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "comprehensive.sruja")

	if err := os.WriteFile(file, []byte(comprehensiveDSL()), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	// Markdown export is disabled - should return error
	exitCode := runExport([]string{"markdown", file}, &stdout, &stderr)
	if exitCode == 0 {
		t.Fatalf("Expected markdown export to be disabled, but got exit code 0")
	}
	if !strings.Contains(stderr.String(), "temporarily disabled") {
		t.Errorf("Expected disabled message, got: %s", stderr.String())
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
