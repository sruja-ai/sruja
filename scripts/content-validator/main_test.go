// scripts/content-validator/main_test.go
package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestValidateFrontmatter_Valid(t *testing.T) {
	tmpDir := t.TempDir()
	testFile := filepath.Join(tmpDir, "test.md")
	content := `---
title: "Test"
description: "Test description"
---
Content here
`
	if err := os.WriteFile(testFile, []byte(content), 0o644); err != nil {
		t.Fatalf("Failed to write test file: %v", err)
	}

	errors := validateFrontmatter(testFile)
	if len(errors) > 0 {
		t.Errorf("Expected no errors for valid frontmatter, got %d: %v", len(errors), errors)
	}
}

func TestValidateFrontmatter_Invalid(t *testing.T) {
	tmpDir := t.TempDir()
	testFile := filepath.Join(tmpDir, "test.md")
	content := `No frontmatter here
Just content
`
	if err := os.WriteFile(testFile, []byte(content), 0o644); err != nil {
		t.Fatalf("Failed to write test file: %v", err)
	}

	errors := validateFrontmatter(testFile)
	if len(errors) == 0 {
		t.Error("Expected errors for invalid frontmatter, got none")
	}
}

func TestValidateFrontmatter_MissingTitle(t *testing.T) {
	tmpDir := t.TempDir()
	testFile := filepath.Join(tmpDir, "test.md")
	content := `---
author: "Me"
---
Content here
`
	if err := os.WriteFile(testFile, []byte(content), 0o644); err != nil {
		t.Fatalf("Failed to write test file: %v", err)
	}

	errors := validateFrontmatter(testFile)
	if len(errors) == 0 {
		t.Error("Expected errors for missing title, got none")
	}
}

func TestValidateFrontmatter_InvalidFormat(t *testing.T) {
	tmpDir := t.TempDir()
	testFile := filepath.Join(tmpDir, "test.md")
	content := `---
title: "Test"
` // Missing closing delimiter
	if err := os.WriteFile(testFile, []byte(content), 0o644); err != nil {
		t.Fatalf("Failed to write test file: %v", err)
	}

	errors := validateFrontmatter(testFile)
	if len(errors) == 0 {
		t.Error("Expected errors for invalid format, got none")
	}
}

func TestValidateDocs_Empty(_ *testing.T) {
	// Should not panic if docs dir doesn't exist
	errors := validateDocs()
	_ = errors
}

func TestValidateFrontmatter_NonExistentFile(t *testing.T) {
	errors := validateFrontmatter("/nonexistent/file.md")
	if len(errors) == 0 {
		t.Error("Expected errors for non-existent file, got none")
	}
}
