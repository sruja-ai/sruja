package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRunExplain(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")
	content := `system = kind "System"
		S1 = system "System 1" {
			description "A test system"
		}`
	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	oldArgs := os.Args
	defer func() { os.Args = oldArgs }()

	var stdout, stderr bytes.Buffer

	// Test explain system
	exitCode := runExplain([]string{"--file", file, "S1"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), "System 1") {
		t.Errorf("Expected output to contain 'System 1', got '%s'", stdout.String())
	}
	if !strings.Contains(stdout.String(), "A test system") {
		t.Errorf("Expected output to contain 'A test system', got '%s'", stdout.String())
	}

	// Test explain JSON
	stdout.Reset()
	stderr.Reset()
	exitCode = runExplain([]string{"--file", file, "--json", "S1"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"id": "S1"`) {
		t.Errorf("Expected JSON output to contain 'S1', got '%s'", stdout.String())
	}

	// Test element not found
	stdout.Reset()
	stderr.Reset()
	exitCode = runExplain([]string{"--file", file, "Unknown"}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for unknown element")
	}
}

func TestRunExplain_Errors(t *testing.T) {
	var stdout, stderr bytes.Buffer

	// Test missing arguments
	if exitCode := runExplain([]string{}, &stdout, &stderr); exitCode == 0 {
		t.Error("Expected non-zero exit code for missing arguments")
	}
	if !strings.Contains(stderr.String(), "Usage:") {
		t.Error("Expected usage message")
	}

	// Test non-existent file
	stderr.Reset()
	if exitCode := runExplain([]string{"--file", "nonexistent.sruja", "S1"}, &stdout, &stderr); exitCode == 0 {
		t.Error("Expected non-zero exit code for non-existent file")
	}
}
