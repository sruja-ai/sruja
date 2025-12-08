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
	jsonFile := filepath.Join(tmpDir, "test.json")
	jsonContent := `{"metadata": {"name": "Test"}, "architecture": {"systems": [{"id": "S", "label": "System"}]}}`
	if err := os.WriteFile(jsonFile, []byte(jsonContent), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	// Test JSON import
	exitCode := runImport([]string{"json", jsonFile}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for JSON import, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), `architecture "Test"`) {
		t.Errorf("Expected DSL output, got: %s", stdout.String())
	}

	// Test unsupported format
	stdout.Reset()
	stderr.Reset()
	exitCode = runImport([]string{"unsupported", jsonFile}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for unsupported format")
	}
	if !strings.Contains(stderr.String(), "Unsupported import format") {
		t.Errorf("Expected unsupported format error, got: %s", stderr.String())
	}

	// Test missing arguments
	stdout.Reset()
	stderr.Reset()
	exitCode = runImport([]string{"json"}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for missing arguments")
	}
	if !strings.Contains(stderr.String(), "Usage:") {
		t.Errorf("Expected usage message, got: %s", stderr.String())
	}
}

func TestRunImport_Errors(t *testing.T) {
	var stdout, stderr bytes.Buffer

	// Test non-existent file
	t.Run("NonExistentInputFile", func(t *testing.T) {
		stderr.Reset()
		if exitCode := runImport([]string{"json", "nonexistent.json"}, &stdout, &stderr); exitCode == 0 {
			t.Error("Expected non-zero exit code for non-existent file")
		}
		if !strings.Contains(stderr.String(), "Error reading file") {
			t.Error("Expected error reading file")
		}
	})

	// Test invalid output directory
	t.Run("InvalidOutputDir", func(t *testing.T) {
		tmpDir := t.TempDir()
		jsonFile := filepath.Join(tmpDir, "test.json")
		jsonContent := `{"metadata": {"name": "Test"}, "architecture": {"systems": [{"id": "S", "label": "System"}]}}`
		if err := os.WriteFile(jsonFile, []byte(jsonContent), 0o644); err != nil {
			t.Fatal(err)
		}

		// Create a file where we want a directory to be
		invalidDir := filepath.Join(tmpDir, "file_as_dir")
		if err := os.WriteFile(invalidDir, []byte(""), 0o644); err != nil {
			t.Fatal(err)
		}

		stderr.Reset()
		if exitCode := runImport([]string{"--out", invalidDir, "json", jsonFile}, &stdout, &stderr); exitCode == 0 {
			t.Error("Expected non-zero exit code for invalid output dir")
		}
		if !strings.Contains(stderr.String(), "Error creating output directory") {
			t.Errorf("Expected error creating output directory, got: %s", stderr.String())
		}
		if !strings.Contains(stderr.String(), "Error creating output directory") {
			t.Errorf("Expected error creating output directory, got: %s", stderr.String())
		}
	})

	// Test invalid JSON content
	t.Run("InvalidJSON", func(t *testing.T) {
		tmpDir := t.TempDir()
		jsonFile := filepath.Join(tmpDir, "invalid.json")
		if err := os.WriteFile(jsonFile, []byte("{invalid json"), 0o644); err != nil {
			t.Fatal(err)
		}

		stderr.Reset()
		if exitCode := runImport([]string{"json", jsonFile}, &stdout, &stderr); exitCode == 0 {
			t.Error("Expected non-zero exit code for invalid JSON")
		}
		if !strings.Contains(stderr.String(), "Import Error") {
			t.Errorf("Expected Import Error, got: %s", stderr.String())
		}
	})
}
