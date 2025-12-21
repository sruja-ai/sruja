package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRunFmt(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")

	// Unformatted content
	content := `model{system S "S"}`
	if err := os.WriteFile(file, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	exitCode := runFmt([]string{file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0, got %d. Stderr: %s", exitCode, stderr.String())
	}

	expected := `model {
  S = system "S"
}
`
	if stdout.String() != expected {
		t.Errorf("Expected formatted output:\n%q\nGot:\n%q", expected, stdout.String())
	}

	// Test missing file argument
	stdout.Reset()
	stderr.Reset()
	exitCode = runFmt([]string{}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for missing argument")
	}
	if !strings.Contains(stderr.String(), "Usage:") {
		t.Errorf("Expected usage message, got: %s", stderr.String())
	}
}

func TestRunFmt_Errors(t *testing.T) {
	var stdout, stderr bytes.Buffer

	// Non-existent file
	if exitCode := runFmt([]string{"nonexistent.sruja"}, &stdout, &stderr); exitCode == 0 {
		t.Error("Expected non-zero exit code for non-existent file")
	}
	if !strings.Contains(stderr.String(), "Error accessing path") {
		t.Error("Expected error accessing path")
	}

	// Invalid syntax
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "invalid.sruja")
	if err := os.WriteFile(file, []byte("invalid syntax"), 0o644); err != nil {
		t.Fatal(err)
	}

	stderr.Reset()
	if exitCode := runFmt([]string{file}, &stdout, &stderr); exitCode == 0 {
		t.Error("Expected non-zero exit code for invalid syntax")
	}
	if !strings.Contains(stderr.String(), "Parser Error") {
		t.Error("Expected parser error")
	}
}

func TestRunFmt_FlagsError(t *testing.T) {
	var stdout, stderr bytes.Buffer
	exitCode := runFmt([]string{"--invalid-flag", "file.sruja"}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected failure for invalid flag")
	}
	if !strings.Contains(stderr.String(), "Error parsing fmt flags") {
		t.Errorf("Expected flag parse error, got: %s", stderr.String())
	}
}
