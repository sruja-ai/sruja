package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRunLint(t *testing.T) {
	tmpDir := t.TempDir()
	validFile := filepath.Join(tmpDir, "valid.sruja")
	invalidFile := filepath.Join(tmpDir, "invalid.sruja")

	validContent := `architecture "Valid" {
		system S1 "System 1"
		system S2 "System 2"
		S1 -> S2 "uses"
	}`
	invalidContent := `architecture "Invalid" {
		system S1 "System 1"
		system S1 "Duplicate System"
	}`

	if err := os.WriteFile(validFile, []byte(validContent), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(invalidFile, []byte(invalidContent), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	// Test valid file
	exitCode := runLint([]string{validFile}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for valid file, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), "No linting errors found") {
		t.Errorf("Expected success message, got: %s", stdout.String())
	}

	// Test invalid file
	stdout.Reset()
	stderr.Reset()
	exitCode = runLint([]string{invalidFile}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for invalid file")
	}
	if !strings.Contains(stderr.String(), "Duplicate identifier") {
		t.Errorf("Expected 'Duplicate identifier' error, got: %s", stderr.String())
	}

	// Test missing file argument
	stdout.Reset()
	stderr.Reset()
	exitCode = runLint([]string{}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for missing argument")
	}
	if !strings.Contains(stderr.String(), "Usage:") {
		t.Errorf("Expected usage message, got: %s", stderr.String())
	}
}
