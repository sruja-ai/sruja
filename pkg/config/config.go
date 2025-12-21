// pkg/config/config.go
package config

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// Config represents the Sruja configuration file structure.
type Config struct {
	Diagrams   *DiagramsConfig   `json:"diagrams,omitempty"`
	Plugins    []string          `json:"plugins,omitempty"`
	Validation *ValidationConfig `json:"validation,omitempty"`
	LSP        *LSPConfig        `json:"lsp,omitempty"`
}

// DiagramsConfig configures diagram generation.
type DiagramsConfig struct {
	Theme         string   `json:"theme,omitempty"`
	ShowMetadata  []string `json:"showMetadata,omitempty"`
	DefaultFormat string   `json:"defaultFormat,omitempty"`
	Layout        string   `json:"layout,omitempty"`
}

// ValidationConfig configures validation rules.
type ValidationConfig struct {
	Strict bool     `json:"strict,omitempty"`
	Rules  []string `json:"rules,omitempty"`
}

// LSPConfig configures LSP behavior.
type LSPConfig struct {
	MetadataSuggestions bool `json:"metadataSuggestions,omitempty"`
	QuickActions        bool `json:"quickActions,omitempty"`
}

// LoadConfig loads configuration from a file or returns default config.
//
//nolint:gocyclo // Config loading is complex
func LoadConfig(configPath string) (*Config, error) {
	// Try to find config file
	if configPath == "" {
		// Look for sruja.config.json in current directory and parent directories
		dir, err := os.Getwd()
		if err != nil {
			return DefaultConfig(), nil
		}

		for {
			candidate := filepath.Join(dir, "sruja.config.json")
			if fi, err := os.Lstat(candidate); err == nil {
				if fi.Mode().IsRegular() && (fi.Mode()&os.ModeSymlink) == 0 {
					configPath = candidate
					break
				}
			}

			parent := filepath.Dir(dir)
			if parent == dir {
				break // Reached root
			}
			dir = parent
		}
	}

	if configPath == "" {
		return DefaultConfig(), nil
	}

	// sanitize and validate the resolved path
	cleanPath := filepath.Clean(configPath)
	// Prevent directory traversal attacks: ensure path doesn't contain ".." after cleaning
	if strings.Contains(cleanPath, "..") {
		return nil, fmt.Errorf("invalid config file path: %s", cleanPath)
	}
	fi, statErr := os.Lstat(cleanPath)
	if statErr != nil {
		if os.IsNotExist(statErr) {
			return DefaultConfig(), nil
		}
		return nil, fmt.Errorf("error accessing config file: %w", statErr)
	}
	if !fi.Mode().IsRegular() || (fi.Mode()&os.ModeSymlink) != 0 {
		return nil, fmt.Errorf("invalid config file type: %s", cleanPath)
	}

	data, err := os.ReadFile(cleanPath)
	if err != nil {
		if os.IsNotExist(err) {
			return DefaultConfig(), nil
		}
		return nil, fmt.Errorf("error reading config file: %w", err)
	}

	var config Config
	if err := json.Unmarshal(data, &config); err != nil {
		return nil, fmt.Errorf("error parsing config file: %w", err)
	}

	return &config, nil
}

// SaveConfig saves configuration to a file.
func SaveConfig(config *Config, configPath string) error {
	if configPath == "" {
		configPath = "sruja.config.json"
	}

	data, err := json.MarshalIndent(config, "", "  ")
	if err != nil {
		return fmt.Errorf("error serializing config: %w", err)
	}

	// sanitize target path
	target := filepath.Clean(configPath)

	// ensure directory exists
	dir := filepath.Dir(target)
	if mkErr := os.MkdirAll(dir, 0o700); mkErr != nil {
		return fmt.Errorf("error ensuring config directory: %w", mkErr)
	}

	// atomic write: write to temp file then rename
	tmp := target + ".tmp"
	if wErr := os.WriteFile(tmp, data, 0o600); wErr != nil {
		return fmt.Errorf("error writing temp config file: %w", wErr)
	}
	if rErr := os.Rename(tmp, target); rErr != nil {
		return fmt.Errorf("error finalizing config file: %w", rErr)
	}

	return nil
}

// DefaultConfig returns the default configuration.
func DefaultConfig() *Config {
	return &Config{
		Diagrams: &DiagramsConfig{
			Theme:         "neutral-default",
			ShowMetadata:  []string{"team", "criticality"},
			DefaultFormat: "d2",
			Layout:        "dagre",
		},
		Plugins: []string{},
		Validation: &ValidationConfig{
			Strict: false,
			Rules:  []string{"unique-ids", "valid-references", "cycle-detection"},
		},
		LSP: &LSPConfig{
			MetadataSuggestions: true,
			QuickActions:        true,
		},
	}
}

// Merge merges another config into this config, with this config taking precedence.
func (c *Config) Merge(other *Config) {
	if other == nil {
		return
	}

	if other.Diagrams != nil {
		if c.Diagrams == nil {
			c.Diagrams = &DiagramsConfig{}
		}
		if other.Diagrams.Theme != "" {
			c.Diagrams.Theme = other.Diagrams.Theme
		}
		if len(other.Diagrams.ShowMetadata) > 0 {
			c.Diagrams.ShowMetadata = other.Diagrams.ShowMetadata
		}
		if other.Diagrams.DefaultFormat != "" {
			c.Diagrams.DefaultFormat = other.Diagrams.DefaultFormat
		}
		if other.Diagrams.Layout != "" {
			c.Diagrams.Layout = other.Diagrams.Layout
		}
	}

	if other.Validation != nil {
		if c.Validation == nil {
			c.Validation = &ValidationConfig{}
		}
		c.Validation.Strict = other.Validation.Strict
		if len(other.Validation.Rules) > 0 {
			c.Validation.Rules = other.Validation.Rules
		}
	}

	if other.LSP != nil {
		if c.LSP == nil {
			c.LSP = &LSPConfig{}
		}
		c.LSP.MetadataSuggestions = other.LSP.MetadataSuggestions
		c.LSP.QuickActions = other.LSP.QuickActions
	}

	if len(other.Plugins) > 0 {
		c.Plugins = other.Plugins
	}
}
