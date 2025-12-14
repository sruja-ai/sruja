package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestToSlug(t *testing.T) {
	tests := []struct {
		in  string
		out string
	}{
		{"Hello World", "hello-world"},
		{"System_Design 101", "system-design-101"},
		{"  Advanced__Topics  ", "advanced--topics"},
		{"Intro!", "intro"},
		{"UPPER lower 123", "upper-lower-123"},
	}
	for _, tt := range tests {
		got := toSlug(tt.in)
		if got != tt.out {
			t.Errorf("toSlug(%q) = %q; want %q", tt.in, got, tt.out)
		}
	}
}

func TestCreateBasicFile(t *testing.T) {
	dir := t.TempDir()
	out := filepath.Join(dir, "lesson.md")
	meta := &ContentMeta{Title: "My Title", Summary: "Summary", Weight: 3, Date: "2025-01-02"}
	createBasicFile(out, meta)
	b, err := os.ReadFile(out)
	if err != nil {
		t.Fatalf("read file: %v", err)
	}
	s := string(b)
	if !strings.Contains(s, `title: "My Title"`) {
		t.Errorf("missing title")
	}
	if !strings.Contains(s, `summary: "Summary"`) {
		t.Errorf("missing summary")
	}
	if !strings.Contains(s, "weight: 3") {
		t.Errorf("missing weight")
	}
	if !strings.Contains(s, `pubDate: "2025-01-02"`) {
		t.Errorf("missing pubDate")
	}
	if !strings.Contains(s, "# My Title") {
		t.Errorf("missing header")
	}
}

func TestCreateFromTemplateFallback(t *testing.T) {
	dir := t.TempDir()
	out := filepath.Join(dir, "doc.md")
	meta := &ContentMeta{Title: "Doc Title"}
	createFromTemplate("missing-template.md", out, meta)
	b, err := os.ReadFile(out)
	if err != nil {
		t.Fatalf("read file: %v", err)
	}
	if !strings.Contains(string(b), `title: "Doc Title"`) {
		t.Errorf("basic file not created")
	}
}
