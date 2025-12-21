package main

import (
	"bytes"
	"os"
	"path/filepath"
	"testing"
)

func TestCLI_ErrorHandling(t *testing.T) {
	tmpDir := t.TempDir()
	nonExistentFile := filepath.Join(tmpDir, "non_existent.sruja")

	t.Run("runCompile_NonExistentFile", func(t *testing.T) {
		var stdout, stderr bytes.Buffer
		exitCode := runCompile([]string{nonExistentFile}, &stdout, &stderr)
		if exitCode == 0 {
			t.Error("Expected non-zero exit code")
		}
		if stderr.Len() == 0 {
			t.Error("Expected stderr output")
		}
	})

	t.Run("runLint_NonExistentFile", func(t *testing.T) {
		var stdout, stderr bytes.Buffer
		exitCode := runLint([]string{nonExistentFile}, &stdout, &stderr)
		if exitCode == 0 {
			t.Error("Expected non-zero exit code")
		}
		if stderr.Len() == 0 {
			t.Error("Expected stderr output")
		}
	})

	t.Run("runFmt_NonExistentFile", func(t *testing.T) {
		var stdout, stderr bytes.Buffer
		exitCode := runTree([]string{nonExistentFile}, &stdout, &stderr)
		if exitCode == 0 {
			t.Error("Expected non-zero exit code")
		}
		if stderr.Len() == 0 {
			t.Error("Expected stderr output")
		}
	})

	t.Run("runTree_NonExistentFile", func(t *testing.T) {
		var stdout, stderr bytes.Buffer
		exitCode := runTree([]string{nonExistentFile}, &stdout, &stderr)
		if exitCode == 0 {
			t.Error("Expected non-zero exit code")
		}
		if stderr.Len() == 0 {
			t.Error("Expected stderr output")
		}
	})

	t.Run("runExport_NonExistentFile", func(t *testing.T) {
		var stdout, stderr bytes.Buffer
		exitCode := runExport([]string{"json", nonExistentFile}, &stdout, &stderr)
		if exitCode == 0 {
			t.Error("Expected non-zero exit code")
		}
		if stderr.Len() == 0 {
			t.Error("Expected stderr output")
		}
	})
}

func TestFindSrujaFile(t *testing.T) {
	// Save current working directory
	wd, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	defer os.Chdir(wd)

	// Create temp dir and switch to it
	tmpDir := t.TempDir()
	err = os.Chdir(tmpDir)
	if err != nil {
		t.Fatal(err)
	}

	t.Run("ExplicitPath", func(t *testing.T) {
		path := findSrujaFile("explicit.sruja")
		if path != "explicit.sruja" {
			t.Errorf("Expected 'explicit.sruja', got %q", path)
		}
	})

	t.Run("NoFileInDir", func(t *testing.T) {
		path := findSrujaFile("")
		if path != "" {
			t.Errorf("Expected empty string, got %q", path)
		}
	})

	t.Run("FileInDir", func(t *testing.T) {
		err := os.WriteFile("test.sruja", []byte{}, 0o644)
		if err != nil {
			t.Fatal(err)
		}
		path := findSrujaFile("")
		if path != "test.sruja" {
			t.Errorf("Expected 'test.sruja', got %q", path)
		}
	})
}
