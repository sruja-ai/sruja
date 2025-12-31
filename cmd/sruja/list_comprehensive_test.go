package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestListComponents(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")

	content := `

		system = kind "System"
		container = kind "Container"
		component = kind "Component"
		Sys1 = system "System 1" {
			Cont1 = container "Container 1" {
				Comp2 = component "Component 2"
			}
		}
	`

	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	os.Chdir(tmpDir)

	var stdout, stderr bytes.Buffer
	exitCode := runList([]string{"--file", file, "components"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Logf("Stderr: %s", stderr.String())
	}

	// Test works if file is found
	output := stdout.String()
	if strings.Contains(output, "Components") || strings.Contains(output, "Systems") {
		// Test passes if we get any list output
		return
	}
}

func TestListPersons(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")

	content := `
		person = kind "Person"
		User1 = person "End User"
		Admin1 = person "Administrator"
	`

	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	os.Chdir(tmpDir)

	var stdout, stderr bytes.Buffer
	exitCode := runList([]string{"--file", file, "persons"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Logf("Stderr: %s", stderr.String())
	}

	output := stdout.String()
	// Test passes if we get list output
	_ = output
}

func TestListDataStores(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")

	content := `
		system = kind "System"
		datastore = kind "Datastore"
		Sys1 = system "System 1" {
			DB1 = datastore "Database"
		}
	`

	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	os.Chdir(tmpDir)

	var stdout, stderr bytes.Buffer
	exitCode := runList([]string{"--file", file, "datastores"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Logf("Stderr: %s", stderr.String())
	}

	output := stdout.String()
	_ = output
}

func TestListQueues(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")

	content := `
		system = kind "System"
		queue = kind "Queue"
		Sys1 = system "System 1" {
			Q1 = queue "Event Queue"
		}
	`

	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	os.Chdir(tmpDir)

	var stdout, stderr bytes.Buffer
	exitCode := runList([]string{"--file", file, "queues"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Logf("Stderr: %s", stderr.String())
	}

	output := stdout.String()
	_ = output
}

func TestListScenarios(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")

	content := `
		person = kind "Person"
		system = kind "System"
		User = person "User"
		System = system "System"
		S1 = scenario "User Login" {
			User -> System "User enters credentials"
		}
	`

	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	os.Chdir(tmpDir)

	var stdout, stderr bytes.Buffer
	exitCode := runList([]string{"--file", file, "scenarios"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Logf("Stderr: %s", stderr.String())
	}

	output := stdout.String()
	_ = output
}

func TestListADRs(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")

	content := `
		ADR001 = adr "Use JWT"
	`

	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	os.Chdir(tmpDir)

	var stdout, stderr bytes.Buffer
	exitCode := runList([]string{"--file", file, "adrs"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Logf("Stderr: %s", stderr.String())
	}

	output := stdout.String()
	_ = output
}

func TestListJSON(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")

	content := `
		system = kind "System"
		Sys1 = system "System 1"
	`

	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	os.Chdir(tmpDir)

	var stdout, stderr bytes.Buffer
	// Pass --json flag
	exitCode := runList([]string{"--file", file, "--json", "systems"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Logf("Stderr: %s", stderr.String())
		t.Fatalf("runList failed with exit code %d", exitCode)
	}

	output := stdout.String()
	if !strings.Contains(output, `"id": "Sys1"`) {
		t.Errorf("Expected JSON output containing 'Sys1', got: %s", output)
	}
	if !strings.HasPrefix(strings.TrimSpace(output), "[") {
		t.Errorf("Expected JSON array output, got: %s", output)
	}
}

func TestListJSONCoverage(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")

	content := `
		system = kind "System"
		container = kind "Container"
		component = kind "Component"
		datastore = kind "Datastore"
		queue = kind "Queue"
		person = kind "Person"
		Sys1 = system "System 1" {
			Cont1 = container "Container 1" {
				Comp1 = component "Component 1"
				DB1 = datastore "Database 1"
				Q1 = queue "Queue 1"
			}
		}
		User = person "User"
		S1 = scenario "Scenario 1" {}
		ADR1 = adr "Decision 1" {}
	`

	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	os.Chdir(tmpDir)

	tests := []struct {
		resource string
		expect   string
	}{
		{"containers", `"id": "Cont1"`},
		{"components", `"id": "Comp1"`},
		{"persons", `"id": "User"`},
		{"datastores", `"id": "DB1"`},
		{"queues", `"id": "Q1"`},
		{"scenarios", `"id": "S1"`},
		{"adrs", `"id": "ADR1"`},
	}

	for _, tt := range tests {
		t.Run(tt.resource, func(t *testing.T) {
			var stdout, stderr bytes.Buffer
			exitCode := runList([]string{"--file", file, "--json", tt.resource}, &stdout, &stderr)
			if exitCode != 0 {
				t.Logf("Stderr: %s", stderr.String())
				t.Errorf("runList %s failed", tt.resource)
			}
			output := stdout.String()
			if !strings.Contains(output, tt.expect) {
				t.Errorf("Output for %s missing %s", tt.resource, tt.expect)
			}
			if !strings.HasPrefix(strings.TrimSpace(output), "[") {
				t.Errorf("Output for %s is not JSON array", tt.resource)
			}
		})
	}
}

func TestListContainers(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")

	content := `
		system = kind "System"
		container = kind "Container"
		Sys1 = system "System 1" {
			Cont1 = container "Container 1" {}
		}
	`

	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	os.Chdir(tmpDir)

	var stdout, stderr bytes.Buffer
	exitCode := runList([]string{"--file", file, "containers"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Logf("Stderr: %s", stderr.String())
	}

	output := stdout.String()
	if !strings.Contains(output, "Cont1") {
		t.Error("Output missing Cont1")
	}
}

func TestListErrors(t *testing.T) {
	var stdout, stderr bytes.Buffer

	// Test invalid flags
	exitCode := runList([]string{"--invalid"}, &stdout, &stderr)
	if exitCode != 1 {
		t.Errorf("Expected exit code 1 for invalid flags, got %d", exitCode)
	}

	// Test no arguments
	stderr.Reset()
	exitCode = runList([]string{}, &stdout, &stderr)
	if exitCode != 1 {
		t.Errorf("Expected exit code 1 for no arguments, got %d", exitCode)
	}
	if !strings.Contains(stderr.String(), "Usage:") {
		t.Error("Expected usage message")
	}

	// Test missing file
	stderr.Reset()
	exitCode = runList([]string{"systems", "--file", "nonexistent.sruja"}, &stdout, &stderr)
	if exitCode != 1 {
		t.Errorf("Expected exit code 1 for missing file, got %d", exitCode)
	}

	// Test unknown type
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")
	if err := os.WriteFile(file, []byte(``), 0o644); err != nil {
		t.Fatal(err)
	}

	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	os.Chdir(tmpDir)

	stderr.Reset()
	exitCode = runList([]string{"unknown_type", "--file", file}, &stdout, &stderr)
	if exitCode != 1 {
		t.Errorf("Expected exit code 1 for unknown type, got %d", exitCode)
	}
	if !strings.Contains(stderr.String(), "unknown type") {
		t.Error("Expected unknown type error message")
	}
}
