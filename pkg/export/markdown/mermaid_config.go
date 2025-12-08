// pkg/export/markdown/mermaid_config.go
// Package markdown provides Mermaid configuration extraction from DSL.
//
//nolint:gocritic // preferFprint false positive or style choice
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// MermaidConfig holds Mermaid diagram configuration options
type MermaidConfig struct {
	Layout         string
	Theme          string
	Look           string
	Direction      string
	UseFrontmatter bool // Whether to write YAML frontmatter config
}

// extractMermaidConfig extracts Mermaid configuration from architecture style/metadata
//
//nolint:gocyclo,mnd // Configuration extraction involves many checks
func extractMermaidConfig(arch *language.Architecture) MermaidConfig {
	config := MermaidConfig{
		Layout:         "elk",
		Theme:          "default",
		Look:           "",
		Direction:      "LR",
		UseFrontmatter: false, // Default: no frontmatter (Mermaid supports inline config)
	}

	if arch == nil {
		return config
	}

	// Check style block for Mermaid options
	if arch.Style != nil {
		if layout, ok := arch.Style["mermaid_layout"]; ok {
			switch strings.ToLower(layout) {
			case "elk", "dagre":
				config.Layout = strings.ToLower(layout)
			case "lr", "tb", "bt", "rl":
				config.Direction = strings.ToUpper(layout)
			}
		}
		if dir, ok := arch.Style["mermaid_direction"]; ok {
			switch strings.ToLower(dir) {
			case "lr", "tb", "bt", "rl":
				config.Direction = strings.ToUpper(dir)
			}
		}
		if theme, ok := arch.Style["mermaid_theme"]; ok {
			config.Theme = theme
		}
		if look, ok := arch.Style["mermaid_look"]; ok {
			config.Look = look
		}
		if fm, ok := arch.Style["mermaid_frontmatter"]; ok {
			config.UseFrontmatter = strings.EqualFold(fm, "true")
		}
	}

	// Check metadata for Mermaid options (alternative)
	for _, meta := range arch.Metadata {
		if meta.Value == nil {
			continue
		}
		val := *meta.Value
		switch meta.Key {
		case "mermaid_layout":
			switch strings.ToLower(val) {
			case "elk", "dagre":
				config.Layout = strings.ToLower(val)
			case "lr", "tb", "bt", "rl":
				config.Direction = strings.ToUpper(val)
			}
		case "mermaid_direction":
			switch strings.ToLower(val) {
			case "lr", "tb", "bt", "rl":
				config.Direction = strings.ToUpper(val)
			}
		case "mermaid_theme":
			config.Theme = val
		case "mermaid_look":
			config.Look = val
		case "mermaid_frontmatter":
			config.UseFrontmatter = strings.EqualFold(val, "true")
		}
	}
	// Check frontmatter from properties
	if props := arch.Properties; props != nil {
		if frontmatter, ok := props["mermaid_frontmatter"]; ok {
			config.UseFrontmatter = strings.EqualFold(frontmatter, "true")
		}
	}

	return config
}

// writeMermaidConfig writes Mermaid configuration directives to the diagram
// Uses YAML frontmatter if UseFrontmatter is true, otherwise uses inline config
func writeMermaidConfig(sb *strings.Builder, config MermaidConfig) {
	if config.UseFrontmatter {
		writeMermaidConfigFrontmatter(sb, config)
	} else {
		writeMermaidConfigInline(sb, config)
	}
}

// writeMermaidConfigFrontmatter writes configuration as YAML frontmatter
func writeMermaidConfigFrontmatter(sb *strings.Builder, config MermaidConfig) {
	sb.WriteString("---\n")
	sb.WriteString("config:\n")
	if config.Layout != "" {
		sb.WriteString(fmt.Sprintf("  layout: %s\n", config.Layout))
	}
	if config.Theme != "" && config.Theme != "default" {
		sb.WriteString(fmt.Sprintf("  theme: %s\n", config.Theme))
	}
	if config.Direction != "" {
		sb.WriteString(fmt.Sprintf("  direction: %s\n", strings.ToLower(config.Direction)))
	}
	if config.Look != "" {
		sb.WriteString(fmt.Sprintf("  look: %s\n", config.Look))
	}
	sb.WriteString("---\n")
}

// writeMermaidConfigInline writes configuration as inline Mermaid directives
// Note: For inline mode, direction is already handled in graphDirection().
// Theme and look customization require frontmatter or external config.
func writeMermaidConfigInline(_ *strings.Builder, _ MermaidConfig) {
	// For inline config without frontmatter, we rely on:
	// - Direction: handled by graphDirection() in the graph declaration
	// - Theme/Look: can be configured via CSS or external Mermaid config
	// No inline directives written to keep compatibility with all renderers
}
