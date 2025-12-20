package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRunTree(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")
	content := `model {
		system S "System" {
			container C "Container" {
				component Comp "Component"
			}
		}
	}`
	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	// Test valid tree
	exitCode := runTree([]string{file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0, got %d. Stderr: %s", exitCode, stderr.String())
	}
	output := stdout.String()
	if !strings.Contains(output, "System (S)") {
		t.Error("Output missing system")
	}
	if !strings.Contains(output, "Container (C)") {
		t.Error("Output missing container")
	}
	if !strings.Contains(output, "Component (Comp)") {
		t.Error("Output missing component")
	}

	// Test missing file argument
	stdout.Reset()
	stderr.Reset()
	exitCode = runTree([]string{}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for missing argument")
	}
	if !strings.Contains(stderr.String(), "Usage:") {
		t.Errorf("Expected usage message, got: %s", stderr.String())
	}
}

func TestRunTree_Errors(t *testing.T) {
	// Test invalid syntax
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "invalid.sruja")
	if err := os.WriteFile(file, []byte("invalid syntax"), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	exitCode := runTree([]string{file}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for invalid syntax")
	}
	if !strings.Contains(stderr.String(), "Parser Error") {
		t.Errorf("Expected parser error, got: %s", stderr.String())
	}
}
