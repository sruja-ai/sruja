package main

import (
	"bytes"
	"os"
	"path/filepath"
	"testing"
)

func TestRunInit(t *testing.T) {
	// Create a temporary directory for testing
	tempDir, err := os.MkdirTemp("", "sruja-init-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Change to temp dir
	originalWd, _ := os.Getwd()
	if err := os.Chdir(tempDir); err != nil {
		t.Fatalf("Failed to change dir: %v", err)
	}
	defer os.Chdir(originalWd)

	// Test case 1: Init with project name
	projectName := "test-project"
	var stdout, stderr bytes.Buffer
	result := runInit([]string{projectName}, &stdout, &stderr)
	if result != 0 {
		t.Fatalf("runInit failed with code %d: %s", result, stderr.String())
	}

	// Verify directory exists
	if _, err := os.Stat(projectName); os.IsNotExist(err) {
		t.Errorf("Project directory not created")
	}

	// Verify main.sruja exists
	mainPath := filepath.Join(projectName, "main.sruja")
	if _, err := os.Stat(mainPath); os.IsNotExist(err) {
		t.Errorf("main.sruja not created")
	}

	// Verify README.md exists
	readmePath := filepath.Join(projectName, "README.md")
	if _, err := os.Stat(readmePath); os.IsNotExist(err) {
		t.Errorf("README.md not created")
	}

	// Verify .gitignore exists
	gitignorePath := filepath.Join(projectName, ".gitignore")
	if _, err := os.Stat(gitignorePath); os.IsNotExist(err) {
		t.Errorf(".gitignore not created")
	}

	// Test case 2: Init with default name
	stdout.Reset()
	stderr.Reset()
	result = runInit([]string{}, &stdout, &stderr)
	if result != 0 {
		t.Fatalf("runInit failed with code %d: %s", result, stderr.String())
	}

	defaultName := "my-sruja-project"
	if _, err := os.Stat(defaultName); os.IsNotExist(err) {
		t.Errorf("Default project directory not created")
	}
}

func TestRunInit_Error(t *testing.T) {
	tempDir := t.TempDir()
	originalWd, _ := os.Getwd()
	if err := os.Chdir(tempDir); err != nil {
		t.Fatalf("Failed to change dir: %v", err)
	}
	defer os.Chdir(originalWd)

	// Create a file blocking the directory creation
	blockedName := "blocked-project"
	if err := os.WriteFile(blockedName, []byte("blocker"), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	result := runInit([]string{blockedName}, &stdout, &stderr)
	if result == 0 {
		t.Error("Expected non-zero exit code when project directory is blocked by a file")
	}
}
