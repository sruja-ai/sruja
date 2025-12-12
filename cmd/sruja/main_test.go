package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRun(t *testing.T) {
	// Test version command
	var stdout, stderr bytes.Buffer
	exitCode := Run([]string{"version"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0, got %d", exitCode)
	}
	if !strings.Contains(stdout.String(), "sruja version") {
		t.Error("Expected version output")
	}

	// Test unknown command
	stdout.Reset()
	stderr.Reset()
	exitCode = Run([]string{"unknown"}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for unknown command")
	}
	if !strings.Contains(stderr.String(), "unknown command") {
		t.Errorf("Expected unknown command error, got: %q", stderr.String())
	}
}

func TestParseArchitectureFile_Errors(t *testing.T) {
	// Test non-existent file
	var stderr bytes.Buffer
	_, err := parseArchitectureFile("non_existent.sruja", &stderr)
	if err == nil {
		t.Error("Expected error for non-existent file")
	}
	if !strings.Contains(stderr.String(), "Error reading file") {
		t.Error("Expected error message for reading file")
	}

	// Test invalid syntax
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "invalid.sruja")
	if err := os.WriteFile(file, []byte("invalid syntax"), 0o644); err != nil {
		t.Fatal(err)
	}

	stderr.Reset()
	_, err = parseArchitectureFile(file, &stderr)
	if err == nil {
		t.Error("Expected error for invalid syntax")
	}
    if !strings.Contains(stderr.String(), "Parser Error") && !strings.Contains(stderr.String(), "unexpected token") && !strings.Contains(stderr.String(), "Internal parser panic") && !strings.Contains(stderr.String(), "sub-expression") {
        t.Errorf("Expected parser error message, got:\n%s", stderr.String())
    }
}
