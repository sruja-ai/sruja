package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestRunDiff(t *testing.T) {
	tmpDir := t.TempDir()
	file1 := filepath.Join(tmpDir, "v1.sruja")
	file2 := filepath.Join(tmpDir, "v2.sruja")

	// Use LikeC4 syntax
	v1 := `model {
		S1 = system "System 1"
	}`
	v2 := `model {
		S1 = system "System 1" {
			C1 = container "Container 1"
		}
	}`

	if err := os.WriteFile(file1, []byte(v1), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(file2, []byte(v2), 0o644); err != nil {
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

	// Use LikeC4 syntax
	v1 := `model {
		S1 = system "System 1"
	}`
	v2 := `model {
		S2 = system "System 2"
	}`

	if err := os.WriteFile(file1, []byte(v1), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(file2, []byte(v2), 0o644); err != nil {
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

	// Use LikeC4 syntax
	v1 := `model {
		S1 = system "S1" {
			C1 = container "C1" {
				Comp1 = component "Comp1"
			}
			C2 = container "C2"
		}
	}`
	v2 := `model {
		S1 = system "S1" {
			C1 = container "C1" {
				Comp2 = component "Comp2"
			}
			C3 = container "C3"
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

func TestComputeDiff_LikeC4Syntax(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	v1 := `model {
		Backend = system "Backend"
		Frontend = system "Frontend"
	}`
	v2 := `model {
		Backend = system "Backend" {
			API = container "API"
		}
		Frontend = system "Frontend"
		Database = system "Database"
	}`

	program1, _, err := parser.Parse("v1.sruja", v1)
	if err != nil {
		t.Fatalf("Failed to parse v1: %v", err)
	}

	program2, _, err := parser.Parse("v2.sruja", v2)
	if err != nil {
		t.Fatalf("Failed to parse v2: %v", err)
	}

	diff := computeDiff(program1, program2, "v1.sruja", "v2.sruja")

	// Check added systems
	if len(diff.AddedSystems) != 1 || diff.AddedSystems[0] != "Database" {
		t.Errorf("Expected 1 added system 'Database', got %v", diff.AddedSystems)
	}

	// Check modified systems (Backend has new container)
	if len(diff.ModifiedSystems) != 1 || diff.ModifiedSystems[0] != "Backend" {
		t.Errorf("Expected 1 modified system 'Backend', got %v", diff.ModifiedSystems)
	}

	// Check added containers
	if len(diff.AddedContainers["Backend"]) != 1 || diff.AddedContainers["Backend"][0] != "API" {
		t.Errorf("Expected added container 'API' in Backend, got %v", diff.AddedContainers["Backend"])
	}
}

func TestComputeDiff_EmptyModels(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	empty := `model {}`
	program1, _, err := parser.Parse("empty.sruja", empty)
	if err != nil {
		t.Fatalf("Failed to parse: %v", err)
	}

	diff := computeDiff(program1, program1, "empty.sruja", "empty.sruja")

	if len(diff.AddedSystems) != 0 {
		t.Errorf("Expected no added systems, got %v", diff.AddedSystems)
	}
	if len(diff.RemovedSystems) != 0 {
		t.Errorf("Expected no removed systems, got %v", diff.RemovedSystems)
	}
	if len(diff.ModifiedSystems) != 0 {
		t.Errorf("Expected no modified systems, got %v", diff.ModifiedSystems)
	}
}

func TestComputeDiff_NilPrograms(t *testing.T) {
	diff := computeDiff(nil, nil, "file1", "file2")

	if len(diff.AddedSystems) != 0 {
		t.Errorf("Expected no added systems, got %v", diff.AddedSystems)
	}
	if len(diff.RemovedSystems) != 0 {
		t.Errorf("Expected no removed systems, got %v", diff.RemovedSystems)
	}
}
