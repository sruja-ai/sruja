package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRunList(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")
	content := `architecture "Test" {
		system S1 "System 1" {
			container C1 "Container 1" {
				component Comp1 "Component 1"
			}
		}
	}`
	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	// Test list systems
	exitCode := runList([]string{"--file", file, "systems"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), "S1") {
		t.Errorf("Expected output to contain 'S1', got '%s'", stdout.String())
	}

	// Test list containers (JSON)
	stdout.Reset()
	stderr.Reset()
	exitCode = runList([]string{"--file", file, "--json", "containers"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"id": "C1"`) {
		t.Errorf("Expected JSON output to contain 'C1', got '%s'", stdout.String())
	}

	// Test invalid type
	stdout.Reset()
	stderr.Reset()
	exitCode = runList([]string{"invalid", "--file", file}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for invalid type")
	}
}
