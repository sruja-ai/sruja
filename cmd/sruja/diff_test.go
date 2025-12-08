package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRunDiff(t *testing.T) {
	tmpDir := t.TempDir()
	file1 := filepath.Join(tmpDir, "v1.sruja")
	file2 := filepath.Join(tmpDir, "v2.sruja")

	if err := os.WriteFile(file1, []byte(`architecture "Test" { system S1 "System 1" }`), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(file2, []byte(`architecture "Test" { system S1 "System 1" { container C1 "Container 1" } }`), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	// Test diff
	exitCode := runDiff([]string{file1, file2}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0, got %d. Stderr: %s", exitCode, stderr.String())
	}

	output := stdout.String()
	// Should show modified system S1 and added container C1
	if !strings.Contains(output, "Modified Systems") {
		t.Error("Expected output to contain 'Modified Systems'")
	}
	if !strings.Contains(output, "S1") {
		t.Error("Expected output to contain 'S1'")
	}
	if !strings.Contains(output, "container C1") {
		t.Error("Expected output to contain 'container C1'")
	}
}

func TestRunDiff_JSON(t *testing.T) {
	tmpDir := t.TempDir()
	file1 := filepath.Join(tmpDir, "v1.sruja")
	file2 := filepath.Join(tmpDir, "v2.sruja")

	if err := os.WriteFile(file1, []byte(`architecture "Test" { system S1 "System 1" }`), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(file2, []byte(`architecture "Test" { system S2 "System 2" }`), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	exitCode := runDiff([]string{"--json", file1, file2}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0, got %d. Stderr: %s", exitCode, stderr.String())
	}

	output := stdout.String()
	if !strings.Contains(output, "\"added_systems\": [") {
		t.Error("Expected JSON output with added_systems")
	}
	if !strings.Contains(output, "\"S2\"") {
		t.Error("Expected S2 in output")
	}
	if !strings.Contains(output, "\"removed_systems\": [") {
		t.Error("Expected JSON output with removed_systems")
	}
	if !strings.Contains(output, "\"S1\"") {
		t.Error("Expected S1 in output")
	}
}

func TestRunDiff_Errors(t *testing.T) {
	var stdout, stderr bytes.Buffer

	// Missing args
	if exitCode := runDiff([]string{}, &stdout, &stderr); exitCode == 0 {
		t.Error("Expected non-zero exit code for missing args")
	}

	// Non-existent file
	stderr.Reset()
	if exitCode := runDiff([]string{"nonexistent", "nonexistent"}, &stdout, &stderr); exitCode == 0 {
		t.Error("Expected non-zero exit code for non-existent file")
	}
}

func TestRunDiff_Complex(t *testing.T) {
	tmpDir := t.TempDir()
	file1 := filepath.Join(tmpDir, "v1.sruja")
	file2 := filepath.Join(tmpDir, "v2.sruja")

	v1 := `architecture "Test" {
		system S1 "S1" {
			container C1 "C1" {
				component Comp1 "Comp1"
			}
			container C2 "C2"
		}
	}`
	v2 := `architecture "Test" {
		system S1 "S1" {
			container C1 "C1" {
				component Comp2 "Comp2"
			}
			container C3 "C3"
		}
	}`

	if err := os.WriteFile(file1, []byte(v1), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(file2, []byte(v2), 0o644); err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	runDiff([]string{file1, file2}, &stdout, &stderr)
	output := stdout.String()

	if !strings.Contains(output, "- container C2") {
		t.Error("Expected removed container C2")
	}
	if !strings.Contains(output, "+ container C3") {
		t.Error("Expected added container C3")
	}
	if !strings.Contains(output, "- component C1.Comp1") {
		t.Error("Expected removed component Comp1")
	}
	if !strings.Contains(output, "+ component C1.Comp2") {
		t.Error("Expected added component Comp2")
	}
}
