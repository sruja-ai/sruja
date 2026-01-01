//nolint:goconst // Test data repetition
package main

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

func TestDiffJSON(t *testing.T) {
	tmpDir := t.TempDir()
	file1 := filepath.Join(tmpDir, "v1.sruja")
	file2 := filepath.Join(tmpDir, "v2.sruja")

	content1 := `system = kind "System"
Sys1 = system "System 1"`

	content2 := `system = kind "System"
Sys1 = system "System 1"
Sys2 = system "System 2"`

	if err := os.WriteFile(file1, []byte(content1), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(file2, []byte(content2), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	exitCode := runDiff([]string{"--json", file1, file2}, &stdout, &stderr)

	if exitCode != 0 {
		t.Logf("Stderr: %s", stderr.String())
		t.Fatalf("diff command failed with exit code %d", exitCode)
	}

	output := stdout.String()

	// Verify JSON structure
	var result map[string]interface{}
	if err := json.Unmarshal([]byte(output), &result); err != nil {
		t.Fatalf("Output is not valid JSON: %v. Output: %s", err, output)
	}
}

func TestDiffModified(t *testing.T) {
	tmpDir := t.TempDir()
	file1 := filepath.Join(tmpDir, "v1.sruja")
	file2 := filepath.Join(tmpDir, "v2.sruja")

	content1 := `system = kind "System"
Sys1 = system "System One"`

	content2 := `system = kind "System"
Sys1 = system "System 1 Updated"`

	if err := os.WriteFile(file1, []byte(content1), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(file2, []byte(content2), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	exitCode := runDiff([]string{file1, file2}, &stdout, &stderr)

	if exitCode != 0 {
		t.Logf("Stderr: %s", stderr.String())
		t.Fatalf("diff command failed with exit code %d", exitCode)
	}

	output := stdout.String()
	if output == "" {
		t.Error("Expected diff output, got empty string")
	}
}

func TestDiffRemoved(t *testing.T) {
	tmpDir := t.TempDir()
	file1 := filepath.Join(tmpDir, "v1.sruja")
	file2 := filepath.Join(tmpDir, "v2.sruja")

	content1 := `system = kind "System"
Sys1 = system "System 1"
Sys2 = system "System 2"`

	content2 := `system = kind "System"
Sys1 = system "System 1"`

	if err := os.WriteFile(file1, []byte(content1), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(file2, []byte(content2), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	exitCode := runDiff([]string{file1, file2}, &stdout, &stderr)

	if exitCode != 0 {
		t.Logf("Stderr: %s", stderr.String())
		t.Fatalf("diff command failed with exit code %d", exitCode)
	}

	output := stdout.String()
	if output == "" {
		t.Error("Expected diff output, got empty string")
	}
}
