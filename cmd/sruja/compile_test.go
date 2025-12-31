package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRunCompile(t *testing.T) {
	tmpDir := t.TempDir()
	validFile := filepath.Join(tmpDir, "valid.sruja")
	invalidFile := filepath.Join(tmpDir, "invalid.sruja")

	validContent := `system = kind "System"
		S1 = system "System 1"
		S2 = system "System 2"
		S1 -> S2 "uses"`
	invalidContent := `system = kind "System"
		S1 = system "System 1"
		S1 = system "Duplicate System"`

	if err := os.WriteFile(validFile, []byte(validContent), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(invalidFile, []byte(invalidContent), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	// Test valid file
	exitCode := runCompile([]string{validFile}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for valid file, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), "Compilation successful") {
		t.Errorf("Expected success message, got: %s", stdout.String())
	}

	// Test invalid file
	stdout.Reset()
	stderr.Reset()
	exitCode = runCompile([]string{invalidFile}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for invalid file")
	}
	if !strings.Contains(stderr.String(), "Duplicate identifier") {
		t.Errorf("Expected 'Duplicate identifier' error, got: %s", stderr.String())
	}

	// Test missing file argument
	stdout.Reset()
	stderr.Reset()
	exitCode = runCompile([]string{}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for missing argument")
	}
	if !strings.Contains(stderr.String(), "Usage:") {
		t.Errorf("Expected usage message, got: %s", stderr.String())
	}
}

func TestRunCompile_Errors(t *testing.T) {
	var stdout, stderr bytes.Buffer

	// Test non-existent file
	if exitCode := runCompile([]string{"nonexistent.sruja"}, &stdout, &stderr); exitCode == 0 {
		t.Error("Expected non-zero exit code for non-existent file")
	}
	if !strings.Contains(stderr.String(), "Error accessing path") {
		t.Error("Expected error accessing path")
	}

	// Test invalid syntax
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "invalid_syntax.sruja")
	if err := os.WriteFile(file, []byte("invalid syntax"), 0o644); err != nil {
		t.Fatal(err)
	}

	stderr.Reset()
	if exitCode := runCompile([]string{file}, &stdout, &stderr); exitCode == 0 {
		t.Error("Expected non-zero exit code for invalid syntax")
	}
	if !strings.Contains(stderr.String(), "Parser Error") {
		t.Error("Expected parser error")
	}
}

func TestRunCompile_Directory(t *testing.T) {
	tmpDir := t.TempDir()
	mainFile := filepath.Join(tmpDir, "main.sruja")
	otherFile := filepath.Join(tmpDir, "other.sruja")

	if err := os.WriteFile(mainFile, []byte(`system=kind "System"
S1 = system "Sys 1"`), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(otherFile, []byte(`S2 = system "Sys 2"`), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	exitCode := runCompile([]string{tmpDir}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for directory compilation, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), "Compilation successful") {
		t.Error("Expected success message")
	}
}
