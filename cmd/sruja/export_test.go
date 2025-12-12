package main

import (
	"bytes"
	"encoding/json"
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

	// Test Markdown export (disabled)
	stdout.Reset()
	stderr.Reset()
	exitCode = runExport([]string{"markdown", file}, &stdout, &stderr)
	if exitCode == 0 {
		t.Errorf("Expected markdown export to be disabled, but got exit code 0")
	}
	if !strings.Contains(stderr.String(), "temporarily disabled") {
		t.Errorf("Expected disabled message, got: %s", stderr.String())
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

func TestRunExport_Mermaid(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "mermaid.sruja")
	err := os.WriteFile(file, []byte(`architecture "A" {
        system S "Sys" { container C "Cont" }
    }`), 0o644)
	if err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	// Mermaid export is disabled - should return error
	exitCode := runExport([]string{"mermaid", file}, &stdout, &stderr)
	if exitCode == 0 {
		t.Fatalf("expected mermaid export to be disabled, but got exit code 0")
	}
	if !strings.Contains(stderr.String(), "temporarily disabled") {
		t.Fatalf("expected disabled message, got: %s", stderr.String())
	}
}

func TestRunExport_JSONExtendedViews(t *testing.T) {
	tmpDir := t.TempDir()
	file := filepath.Join(tmpDir, "ext.sruja")
	err := os.WriteFile(file, []byte(`architecture "A" {
        system S "Sys" {
            container C "Cont"
        }
    }`), 0o644)
	if err != nil {
		t.Fatal(err)
	}

	var stdout, stderr bytes.Buffer
	exitCode := runExport([]string{"-extended", "json", file}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit=%d stderr=%s", exitCode, stderr.String())
	}

	var result map[string]interface{}
	if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
		t.Fatalf("json unmarshal: %v", err)
	}
	views, ok := result["views"].(map[string]interface{})
	if !ok {
		t.Fatalf("expected views present in extended output")
	}
	l2 := views["L2"].(map[string]interface{})
	if _, ok := l2["S"]; !ok {
		t.Fatalf("expected L2 key 'S'")
	}
	l3 := views["L3"].(map[string]interface{})
	if _, ok := l3["S.C"]; !ok {
		t.Fatalf("expected L3 key 'S.C'")
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

// SVG export removed; views tests removed accordingly.
