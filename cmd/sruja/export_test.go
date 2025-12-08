package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRunExport(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "test.sruja")
	err := os.WriteFile(file, []byte(`architecture "Test" {
		system Sys "System" {
			container Cont "Container"
		}
	}`), 0o644)
	if err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	// Test JSON export
	exitCode := runExport([]string{"json", file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for JSON export, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), `"name": "Test"`) {
		t.Errorf("Expected JSON output, got: %s", stdout.String())
	}

	// Test Markdown export
	stdout.Reset()
	stderr.Reset()
	exitCode = runExport([]string{"markdown", file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for Markdown export, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), "# Test") {
		t.Errorf("Expected Markdown output, got: %s", stdout.String())
	}

	// Test HTML export
	stdout.Reset()
	stderr.Reset()
	exitCode = runExport([]string{"html", file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for HTML export, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), "<!DOCTYPE html>") {
		t.Errorf("Expected HTML output, got: %s", stdout.String())
	}

	// Test SVG export
	stdout.Reset()
	stderr.Reset()
	exitCode = runExport([]string{"svg", file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for SVG export, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), "<svg") {
		t.Errorf("Expected SVG output, got: %s", stdout.String())
	}

	// Test SVG container view export
	stdout.Reset()
	stderr.Reset()
	exitCode = runExport([]string{"-view", "container:Sys", "svg", file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for SVG container view export, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), "<svg") {
		t.Errorf("Expected SVG output for container view, got: %s", stdout.String())
	}

	// Test HTML export with flags
	stdout.Reset()
	stderr.Reset()
	exitCode = runExport([]string{"-local", "-single-file", "-out", tmpDir, "html", file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for HTML export with flags, got %d. Stderr: %s", exitCode, stderr.String())
	}

	// Test SVG component view export
	stdout.Reset()
	stderr.Reset()
	// Note: Component view falls back to container view for now as per implementation
	exitCode = runExport([]string{"-view", "component:Sys/Cont", "svg", file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0 for SVG component view export, got %d. Stderr: %s", exitCode, stderr.String())
	}
	if !strings.Contains(stdout.String(), "<svg") {
		t.Errorf("Expected SVG output for component view, got: %s", stdout.String())
	}

	// Test unsupported format
	stdout.Reset()
	stderr.Reset()
	exitCode = runExport([]string{"unsupported", file}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for unsupported format")
	}
	if !strings.Contains(stderr.String(), "Unsupported export format") {
		t.Errorf("Expected unsupported format error, got: %s", stderr.String())
	}

	// Test missing arguments
	stdout.Reset()
	stderr.Reset()
	exitCode = runExport([]string{"json"}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for missing arguments")
	}
	if !strings.Contains(stderr.String(), "Usage:") {
		t.Errorf("Expected usage message, got: %s", stderr.String())
	}
}

func TestRunExport_Errors(t *testing.T) {
	var stdout, stderr bytes.Buffer

	// Test non-existent file
	if exitCode := runExport([]string{"json", "nonexistent.sruja"}, &stdout, &stderr); exitCode == 0 {
		t.Error("Expected non-zero exit code for non-existent file")
	}
	if !strings.Contains(stderr.String(), "Error reading file") {
		t.Error("Expected error reading file")
	}

	// Test invalid syntax
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "invalid.sruja")
	if err := os.WriteFile(file, []byte("invalid syntax"), 0o644); err != nil {
		t.Fatal(err)
	}

	stderr.Reset()
	if exitCode := runExport([]string{"json", file}, &stdout, &stderr); exitCode == 0 {
		t.Error("Expected non-zero exit code for invalid syntax")
	}
	if !strings.Contains(stderr.String(), "Parser Error") {
		t.Error("Expected parser error")
	}
}

func TestRunExport_SVG_Views(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "views.sruja")
	err := os.WriteFile(file, []byte(`architecture "Views" {
		system Sys "System" {
			container Cont "Container"
		}
		deployment Prod "Prod" {
			node Server "Server" {
				containerInstance Cont
			}
		}
	}`), 0o644)
	if err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer

	tests := []struct {
		view     string
		wantOut  string
		wantErr  string
		exitCode int
	}{
		{"all", "<svg", "", 0},
		{"deployment", "<svg", "", 0},
		{"deployment:Prod", "<svg", "", 0},
		{"c4:l1", "<svg", "", 0},
		{"c4:l2:Sys", "<svg", "", 0},
		{"c4:l3:Sys/Cont", "<svg", "", 0},
		{"container:Unknown", "", "System not found", 1},
		{"component:Sys/Unknown", "", "Component view not found", 1},
		{"deployment:Unknown", "", "Deployment node not found", 1},
	}

	for _, tt := range tests {
		stdout.Reset()
		stderr.Reset()
		exitCode := runExport([]string{"-view", tt.view, "svg", file}, &stdout, &stderr)
		if exitCode != tt.exitCode {
			t.Errorf("view %s: expected exit code %d, got %d. Stderr: %s", tt.view, tt.exitCode, exitCode, stderr.String())
		}
		if tt.wantOut != "" && !strings.Contains(stdout.String(), tt.wantOut) {
			t.Errorf("view %s: expected output containing %q, got %q", tt.view, tt.wantOut, stdout.String())
		}
		if tt.wantErr != "" && !strings.Contains(stderr.String(), tt.wantErr) {
			t.Errorf("view %s: expected error containing %q, got %q", tt.view, tt.wantErr, stderr.String())
		}
	}
}
