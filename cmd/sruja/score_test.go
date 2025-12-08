package main

import (
	"bytes"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestScoreCommand(t *testing.T) {
	// Create a temporary directory for test files
	tempDir := t.TempDir()

	// Create a valid Sruja file with some violations to test scoring
	validFile := filepath.Join(tempDir, "score_test.sruja")
	content := `
architecture "ScoreTest" {
	system Sys {
		container API {
			technology "Go"
			description "API Service"
		}
		// Missing description (violation)
		container Worker {
			technology "Python"
		}
	}
	// Orphan element (violation)
	person User
}
`
	err := os.WriteFile(validFile, []byte(content), 0o644)
	require.NoError(t, err)

	// Capture stdout
	r, w, _ := os.Pipe()

	// Run the score command
	// We need to simulate command execution. Since runScore is private and part of main package,
	// we can't call it directly if we are in main_test package unless we export it or use main package.
	// Assuming this test is in package main.

	// Setup root command and args
	rootCmd.SetArgs([]string{"score", validFile})
	rootCmd.SetOut(w)
	rootCmd.SetErr(w)

	// Execute
	err = rootCmd.Execute()

	// Restore stdout
	w.Close()

	// Read output
	var buf bytes.Buffer
	_, _ = buf.ReadFrom(r)
	output := buf.String()

	// Verify execution
	assert.NoError(t, err)

	// Verify output contains score and violations
	assert.Contains(t, output, "Architecture Score:")
	assert.Contains(t, output, "Deductions:")
	assert.Contains(t, output, "Missing Description") // Worker
	assert.Contains(t, output, "Orphan Element")      // User

	// Check that score is not 100
	assert.NotContains(t, output, "Score: 100/100")
}

func TestScoreCommand_FileNotFound(t *testing.T) {
	// Capture stdout/stderr
	oldStderr := os.Stderr
	_, w, _ := os.Pipe()
	os.Stderr = w

	rootCmd.SetArgs([]string{"score", "non_existent.sruja"})
	err := rootCmd.Execute()

	w.Close()
	os.Stderr = oldStderr

	assert.Error(t, err)
}
