package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestCobraIntegration(t *testing.T) {
	// Helper to reset command state
	resetCmd := func() {
		rootCmd.SetArgs(nil)
		rootCmd.SetOut(nil)
		rootCmd.SetErr(nil)
	}

	t.Run("Version", func(t *testing.T) {
		resetCmd()
		var stdout bytes.Buffer
		rootCmd.SetOut(&stdout)
		rootCmd.SetArgs([]string{"version"})

		err := rootCmd.Execute()
		if err != nil {
			t.Fatalf("Execute failed: %v", err)
		}

		if !strings.Contains(stdout.String(), "sruja version") {
			t.Errorf("Expected version output, got: %s", stdout.String())
		}
	})

	t.Run("Compile", func(t *testing.T) {
		resetCmd()
		tmpDir := t.TempDir()
		file := filepath.Join(tmpDir, "test.sruja")
		err := os.WriteFile(file, []byte(`architecture "Test" {}`), 0o644)
		if err != nil {
			t.Fatal(err)
		}

		var stdout, stderr bytes.Buffer
		rootCmd.SetOut(&stdout)
		rootCmd.SetErr(&stderr)
		rootCmd.SetArgs([]string{"compile", file})

		err = rootCmd.Execute()
		if err != nil {
			t.Fatalf("Execute failed: %v", err)
		}

		if !strings.Contains(stdout.String(), "Compilation successful") {
			t.Errorf("Expected success message, got: %s", stdout.String())
		}
	})

	t.Run("Lint", func(t *testing.T) {
		resetCmd()
		tmpDir := t.TempDir()
		file := filepath.Join(tmpDir, "test.sruja")
		err := os.WriteFile(file, []byte(`architecture "Test" {}`), 0o644)
		if err != nil {
			t.Fatal(err)
		}

		var stdout, stderr bytes.Buffer
		rootCmd.SetOut(&stdout)
		rootCmd.SetErr(&stderr)
		rootCmd.SetArgs([]string{"lint", file})

		err = rootCmd.Execute()
		if err != nil {
			t.Fatalf("Execute failed: %v", err)
		}

		if !strings.Contains(stdout.String(), "No linting errors found") {
			t.Errorf("Expected success message, got: %s", stdout.String())
		}
	})

	t.Run("Export", func(t *testing.T) {
		resetCmd()
		tmpDir := t.TempDir()
		file := filepath.Join(tmpDir, "test.sruja")
		err := os.WriteFile(file, []byte(`architecture "Test" {}`), 0o644)
		if err != nil {
			t.Fatal(err)
		}

		var stdout, stderr bytes.Buffer
		rootCmd.SetOut(&stdout)
		rootCmd.SetErr(&stderr)
		rootCmd.SetArgs([]string{"export", "json", file})

		err = rootCmd.Execute()
		if err != nil {
			t.Fatalf("Execute failed: %v", err)
		}

		if !strings.Contains(stdout.String(), `"architecture":`) {
			t.Errorf("Expected JSON output, got: %s", stdout.String())
		}
	})

	t.Run("UnknownCommand", func(t *testing.T) {
		resetCmd()
		var stderr bytes.Buffer
		rootCmd.SetErr(&stderr)
		rootCmd.SetArgs([]string{"unknown"})

		// Should return error
		err := rootCmd.Execute()
		if err == nil {
			t.Error("Expected error for unknown command")
		}
	})
}
