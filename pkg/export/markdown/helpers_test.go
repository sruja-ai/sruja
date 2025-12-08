package markdown

import "testing"

func TestEscapeMermaidLabel(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"Normal Label", "Normal Label"},
		{"Label with \"quotes\"", "Label with \\\"quotes\\\""},
		{"Label with <brackets>", "Label with <brackets>"},
	}

	for _, tt := range tests {
		result := escapeMermaidLabel(tt.input)
		if result != tt.expected {
			t.Errorf("escapeMermaidLabel(%q) = %q, want %q", tt.input, result, tt.expected)
		}
	}
}

func TestEscapeMarkdown(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"Normal Text", "Normal Text"},
		{"Text with | pipe", "Text with | pipe"},
		{"Text with * star", "Text with \\* star"},
		{"Text with [ brackets ]", "Text with \\[ brackets \\]"},
	}

	for _, tt := range tests {
		result := escapeMarkdown(tt.input)
		if result != tt.expected {
			t.Errorf("escapeMarkdown(%q) = %q, want %q", tt.input, result, tt.expected)
		}
	}
}

func TestGenerateAnchor(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"Heading One", "heading-one"},
		{"Heading (Two)", "heading-two"},
		{"Heading.Three", "headingthree"},
	}

	for _, tt := range tests {
		result := generateAnchor(tt.input)
		if result != tt.expected {
			t.Errorf("generateAnchor(%q) = %q, want %q", tt.input, result, tt.expected)
		}
	}
}
