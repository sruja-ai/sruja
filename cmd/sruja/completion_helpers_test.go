package main

import (
	"os"
	"testing"

	"github.com/spf13/cobra"
)

func TestCompleteSrujaFiles(t *testing.T) {
	tmpDir := t.TempDir()
	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	if err := os.Chdir(tmpDir); err != nil {
		t.Fatal(err)
	}

	// Create test files
	files := []string{"test1.sruja", "test2.sruja", "other.txt"}
	for _, f := range files {
		if err := os.WriteFile(f, []byte(""), 0o644); err != nil {
			t.Fatal(err)
		}
	}

	cmd := &cobra.Command{}

	// Test with no args (should complete)
	completions, directive := completeSrujaFiles(cmd, []string{}, "")
	if directive != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("Expected NoFileComp directive, got %v", directive)
	}
	if len(completions) != 2 {
		t.Errorf("Expected 2 completions, got %d", len(completions))
	}

	// Test with args (should not complete)
	completions, directive = completeSrujaFiles(cmd, []string{"arg1"}, "")
	if directive != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("Expected NoFileComp directive, got %v", directive)
	}
	if completions != nil {
		t.Error("Expected nil completions when args present")
	}

	// Test with prefix
	completions, _ = completeSrujaFiles(cmd, []string{}, "test1")
	if len(completions) != 1 || completions[0] != "test1.sruja" {
		t.Errorf("Expected test1.sruja, got %v", completions)
	}
}

// Additional test coverage for edge cases
func TestCompleteSrujaFiles_NoMatches(t *testing.T) {
	tmpDir := t.TempDir()
	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	if err := os.Chdir(tmpDir); err != nil {
		t.Fatal(err)
	}

	// Create only .sruja files
	files := []string{"example.sruja", "sample.sruja"}
	for _, f := range files {
		if err := os.WriteFile(f, []byte(""), 0o644); err != nil {
			t.Fatal(err)
		}
	}

	cmd := &cobra.Command{}

	// Test with a prefix that doesn't match anything
	// Should fallback to all .sruja files
	completions, directive := completeSrujaFiles(cmd, []string{}, "nomatch")
	if directive != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("Expected NoFileComp directive, got %v", directive)
	}
	// Should return all .sruja files as fallback
	if len(completions) != 2 {
		t.Errorf("Expected 2 completions (fallback), got %d: %v", len(completions), completions)
	}
}

func TestCompleteSrujaFiles_EmptyDirectory(t *testing.T) {
	tmpDir := t.TempDir()
	originalDir, _ := os.Getwd()
	defer os.Chdir(originalDir)
	if err := os.Chdir(tmpDir); err != nil {
		t.Fatal(err)
	}

	cmd := &cobra.Command{}

	// Test with empty directory
	completions, directive := completeSrujaFiles(cmd, []string{}, "")
	if directive != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("Expected NoFileComp directive, got %v", directive)
	}
	if len(completions) != 0 {
		t.Errorf("Expected 0 completions in empty directory, got %d", len(completions))
	}
}
