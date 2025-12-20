package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRunImport(t *testing.T) {
	tmpDir := t.TempDir()
	jsonFile := filepath.Join(tmpDir, "simple.json")

	err := os.WriteFile(jsonFile, []byte(`{
		"id": "sys1",
		"label": "System 1"
	}`), 0644)
	if err != nil {
		t.Fatal(err)
	}

	stdout := &bytes.Buffer{}
	stderr := &bytes.Buffer{}

	// Test basic import json
	args := []string{"json", jsonFile}
	exitCode := runImport(args, stdout, stderr)
	if exitCode != 0 {
		t.Fatalf("runImport failed: %s", stderr.String())
	}

	if !bytes.Contains(stdout.Bytes(), []byte("system sys1")) {
		t.Errorf("Expected stdout to contain 'system sys1', got %s", stdout.String())
	}

	// Test unsupported format
	stderr.Reset()
	exitCode = runImport([]string{"xml", jsonFile}, stdout, stderr)
	if exitCode != 1 {
		t.Errorf("Expected exit code 1 for unsupported format, got %d", exitCode)
	}
}

func TestRunImport_Errors(t *testing.T) {
	var stdout, stderr bytes.Buffer

	// Test missing args
	exitCode := runImport([]string{}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected failure for missing args")
	}
	if !strings.Contains(stderr.String(), "Usage:") {
		t.Error("Expected usage message")
	}

	// Test invalid flag
	stderr.Reset()
	exitCode = runImport([]string{"--invalid-flag", "json", "file.json"}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected failure for invalid flag")
	}
	if !strings.Contains(stderr.String(), "Error parsing import flags") {
		t.Error("Expected flag parse error")
	}

	// Test file not found
	stderr.Reset()
	exitCode = runImport([]string{"json", "nonexistent.json"}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected failure for non-existent file")
	}
	if !strings.Contains(stderr.String(), "Error accessing path") {
		t.Errorf("Expected 'Error accessing path', got: %s", stderr.String())
	}
}
