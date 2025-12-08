package markdown

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExtractMermaidConfig(t *testing.T) {
	tests := []struct {
		name     string
		arch     *language.Architecture
		expected MermaidConfig
	}{
		{
			name: "Nil Architecture",
			arch: nil,
			expected: MermaidConfig{
				Layout:         "elk",
				Theme:          "default",
				Direction:      "LR",
				UseFrontmatter: false,
			},
		},
		{
			name: "Style Block Config",
			arch: &language.Architecture{
				Style: map[string]string{
					"mermaid_layout":      "dagre",
					"mermaid_direction":   "TB",
					"mermaid_theme":       "forest",
					"mermaid_look":        "hand-drawn",
					"mermaid_frontmatter": "true",
				},
			},
			expected: MermaidConfig{
				Layout:         "dagre",
				Theme:          "forest",
				Direction:      "TB",
				Look:           "hand-drawn",
				UseFrontmatter: true,
			},
		},
		{
			name: "Metadata Config",
			arch: &language.Architecture{
				Metadata: []*language.MetaEntry{
					{Key: "mermaid_layout", Value: strPtr("elk")},
					{Key: "mermaid_direction", Value: strPtr("RL")},
					{Key: "mermaid_theme", Value: strPtr("dark")},
					{Key: "mermaid_look", Value: strPtr("classic")},
					{Key: "mermaid_frontmatter", Value: strPtr("true")},
				},
			},
			expected: MermaidConfig{
				Layout:         "elk",
				Theme:          "dark",
				Direction:      "RL",
				Look:           "classic",
				UseFrontmatter: true,
			},
		},
		{
			name: "Mixed Config (Metadata Priority)",
			arch: &language.Architecture{
				Style: map[string]string{
					"mermaid_layout": "dagre",
				},
				Metadata: []*language.MetaEntry{
					{Key: "mermaid_layout", Value: strPtr("elk")},
				},
			},
			expected: MermaidConfig{
				Layout:         "elk",
				Theme:          "default",
				Direction:      "LR",
				UseFrontmatter: false,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			config := extractMermaidConfig(tt.arch)
			if config.Layout != tt.expected.Layout {
				t.Errorf("Layout: got %s, want %s", config.Layout, tt.expected.Layout)
			}
			if config.Theme != tt.expected.Theme {
				t.Errorf("Theme: got %s, want %s", config.Theme, tt.expected.Theme)
			}
			if config.Direction != tt.expected.Direction {
				t.Errorf("Direction: got %s, want %s", config.Direction, tt.expected.Direction)
			}
			if config.Look != tt.expected.Look {
				t.Errorf("Look: got %s, want %s", config.Look, tt.expected.Look)
			}
			if config.UseFrontmatter != tt.expected.UseFrontmatter {
				t.Errorf("UseFrontmatter: got %v, want %v", config.UseFrontmatter, tt.expected.UseFrontmatter)
			}
		})
	}
}

func TestWriteMermaidConfig(t *testing.T) {
	tests := []struct {
		name     string
		config   MermaidConfig
		contains []string
	}{
		{
			name: "Inline (Default)",
			config: MermaidConfig{
				UseFrontmatter: false,
			},
			contains: []string{}, // Should be empty
		},
		{
			name: "Frontmatter Full",
			config: MermaidConfig{
				UseFrontmatter: true,
				Layout:         "elk",
				Theme:          "forest",
				Direction:      "TB",
				Look:           "hand-drawn",
			},
			contains: []string{
				"---",
				"config:",
				"layout: elk",
				"theme: forest",
				"direction: tb",
				"look: hand-drawn",
			},
		},
		{
			name: "Frontmatter Minimal",
			config: MermaidConfig{
				UseFrontmatter: true,
				Theme:          "default", // Should be skipped
			},
			contains: []string{
				"---",
				"config:",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var sb strings.Builder
			writeMermaidConfig(&sb, tt.config)
			output := sb.String()

			for _, s := range tt.contains {
				if !strings.Contains(output, s) {
					t.Errorf("Output missing %q. Got:\n%s", s, output)
				}
			}
		})
	}
}

func TestWriteMermaidConfigInline(t *testing.T) {
	var sb strings.Builder
	config := MermaidConfig{
		UseFrontmatter: false,
		Layout:         "elk",
	}
	writeMermaidConfigInline(&sb, config)
	if sb.String() != "" {
		t.Errorf("Expected empty inline config, got: %s", sb.String())
	}
}

func strPtr(s string) *string {
	return &s
}
