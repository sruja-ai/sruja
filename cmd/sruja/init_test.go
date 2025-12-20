package main

import (
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
	err = runInit(nil, []string{projectName})
	if err != nil {
		t.Fatalf("runInit failed: %v", err)
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
	err = runInit(nil, []string{})
	if err != nil {
		t.Fatalf("runInit failed: %v", err)
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

	err := runInit(nil, []string{blockedName})
	if err == nil {
		t.Error("Expected error when project directory is blocked by a file")
	}
}
