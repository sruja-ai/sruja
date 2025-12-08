package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRunChangeCreate(t *testing.T) {
	// Setup temp dir and switch to it
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer os.Chdir(oldWd)
	if err := os.Chdir(tmpDir); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	// Test create change
	exitCode := runChangeCreate("test-change", "REQ-1", "Alice", []string{"Bob"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0, got %d. Stderr: %s", exitCode, stderr.String())
	}

	// Verify file created
	expectedFile := filepath.Join("changes", "001-test-change.sruja")
	if _, err := os.Stat(expectedFile); os.IsNotExist(err) {
		t.Errorf("Expected change file %s to exist", expectedFile)
	}

	// Test validate change
	stdout.Reset()
	stderr.Reset()
	exitCode = runChangeValidate(expectedFile, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for validate, got %d. Stderr: %s", exitCode, stderr.String())
	}

	// Test validate non-existent file
	stdout.Reset()
	stderr.Reset()
	exitCode = runChangeValidate("non-existent.sruja", &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for non-existent file")
	}
	if !strings.Contains(stderr.String(), "Error reading change file") {
		t.Errorf("Expected read error, got: %s", stderr.String())
	}

	// Test validate invalid file
	invalidFile := filepath.Join(tmpDir, "invalid.sruja")
	if err := os.WriteFile(invalidFile, []byte("invalid content"), 0o644); err != nil {
		t.Fatal(err)
	}
	stdout.Reset()
	stderr.Reset()
	exitCode = runChangeValidate(invalidFile, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for invalid file")
	}
	if !strings.Contains(stderr.String(), "Parse error") {
		t.Errorf("Expected parse error, got: %s", stderr.String())
	}
}

func TestFindNextChangeNumber(t *testing.T) {
	tmpDir := t.TempDir()

	// Test empty dir
	if num := findNextChangeNumber(tmpDir); num != 1 {
		t.Errorf("Expected 1 for empty dir, got %d", num)
	}

	// Create some files
	createFile := func(name string) {
		if err := os.WriteFile(filepath.Join(tmpDir, name), []byte(""), 0o644); err != nil {
			t.Fatal(err)
		}
	}

	createFile("001-first.sruja")
	if num := findNextChangeNumber(tmpDir); num != 2 {
		t.Errorf("Expected 2, got %d", num)
	}

	createFile("002-second.sruja")
	if num := findNextChangeNumber(tmpDir); num != 3 {
		t.Errorf("Expected 3, got %d", num)
	}

	// Test with non-sruja files and dirs
	createFile("other.txt")
	if err := os.Mkdir(filepath.Join(tmpDir, "subdir"), 0o755); err != nil {
		t.Fatal(err)
	}
	if num := findNextChangeNumber(tmpDir); num != 3 {
		t.Errorf("Expected 3 ignoring other files, got %d", num)
	}

	// Test with invalid number format
	createFile("abc-invalid.sruja")
	if num := findNextChangeNumber(tmpDir); num != 3 {
		t.Errorf("Expected 3 ignoring invalid format, got %d", num)
	}

	// Test with gap
	createFile("005-gap.sruja")
	if num := findNextChangeNumber(tmpDir); num != 6 {
		t.Errorf("Expected 6 after gap, got %d", num)
	}
}
